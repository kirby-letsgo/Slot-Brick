//! A manually opened, passive FTP server for the player's files on the card.
//! No shell commands, executable folders, anonymous login, or always-on service.
use ring::rand::{SecureRandom, SystemRandom};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, UNIX_EPOCH};

pub const PORT: u16 = 2121;
pub const USER: &str = "slot";
const ROOTS: &[&str] = &[
    "Games",
    "Saves",
    "States",
    "Labels",
    "BIOS",
    "Overlays",
    "Wallpapers",
];
const CLIENTS: usize = 4;
const COMMAND_MAX: usize = 4096;
const IO_TICK: Duration = Duration::from_millis(200);
const DATA_IDLE: Duration = Duration::from_secs(30);
const CONTROL_IDLE: Duration = Duration::from_secs(300);
const MAX_FILE: u64 = 0xffff_ffff; // FAT32's file limit, without buffering the file in RAM.
static SERIAL: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    pub clients: usize,
    pub uploaded: u64,
    pub downloaded: u64,
    pub activity: String,
}
struct Shared {
    stop: AtomicBool,
    changed: AtomicBool,
    clients: AtomicUsize,
    uploaded: AtomicU64,
    downloaded: AtomicU64,
    activity: Mutex<String>,
    // Serializes filesystem mutations; readers never see an upload until its final rename.
    mutation: Mutex<()>,
}
impl Shared {
    fn active(&self) -> io::Result<()> {
        if self.stop.load(Ordering::Relaxed) {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Transfer stopped",
            ))
        } else {
            Ok(())
        }
    }
    fn activity(&self, text: String) {
        *self.activity.lock().unwrap_or_else(|e| e.into_inner()) = text;
    }
}

pub struct Server {
    address: SocketAddr,
    password: String,
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
}
impl Server {
    pub fn start(root: &Path, address: SocketAddr) -> io::Result<Self> {
        let root = root.canonicalize()?;
        let listener = TcpListener::bind(address)?;
        let address = listener.local_addr()?;
        listener.set_nonblocking(true)?;
        let mut random = [0; 8];
        SystemRandom::new()
            .fill(&mut random)
            .map_err(|_| io::Error::other("Cannot generate FTP password"))?;
        let password = format!("{:08}", u64::from_le_bytes(random) % 100_000_000);
        let shared = Arc::new(Shared {
            stop: AtomicBool::new(false),
            changed: AtomicBool::new(false),
            clients: AtomicUsize::new(0),
            uploaded: AtomicU64::new(0),
            downloaded: AtomicU64::new(0),
            activity: Mutex::new("Waiting for a connection".into()),
            mutation: Mutex::new(()),
        });
        let (state, secret) = (shared.clone(), password.clone());
        let worker = thread::Builder::new()
            .name("slot-ftp".into())
            .spawn(move || {
                let mut clients: Vec<JoinHandle<()>> = Vec::new();
                while !state.stop.load(Ordering::Relaxed) {
                    let mut at = 0;
                    while at < clients.len() {
                        if clients[at].is_finished() {
                            let _ = clients.swap_remove(at).join();
                        } else {
                            at += 1;
                        }
                    }
                    match listener.accept() {
                        Ok((mut socket, peer)) => {
                            if clients.len() >= CLIENTS {
                                let _ = socket.set_write_timeout(Some(IO_TICK));
                                let _ = socket.write_all(b"421 Too many connections\r\n");
                                continue;
                            }
                            let (state, root, secret) =
                                (state.clone(), root.clone(), secret.clone());
                            if let Ok(handle) = thread::Builder::new()
                                .name("slot-ftp-client".into())
                                .spawn(move || {
                                    state.clients.fetch_add(1, Ordering::Relaxed);
                                    let _ = Client::new(socket, peer, root, secret, state.clone())
                                        .and_then(|mut c| c.run());
                                    state.clients.fetch_sub(1, Ordering::Relaxed);
                                })
                            {
                                clients.push(handle);
                            }
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(40))
                        }
                        Err(_) => break,
                    }
                }
                state.stop.store(true, Ordering::Relaxed);
                for client in clients {
                    let _ = client.join();
                }
            })?;
        Ok(Self {
            address,
            password,
            shared,
            worker: Some(worker),
        })
    }
    pub fn address(&self) -> SocketAddr {
        self.address
    }
    pub fn password(&self) -> &str {
        &self.password
    }
    pub fn progress(&self) -> Progress {
        Progress {
            clients: self.shared.clients.load(Ordering::Relaxed),
            uploaded: self.shared.uploaded.load(Ordering::Relaxed),
            downloaded: self.shared.downloaded.load(Ordering::Relaxed),
            activity: self
                .shared
                .activity
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
        }
    }
    /// Stop listeners and clients before returning; interrupted uploads discard their temporary file.
    pub fn stop(&mut self) -> bool {
        self.shared.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        self.shared.changed.load(Ordering::Relaxed)
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Bind only the Wi-Fi interface, even if another network interface exists.
pub fn wifi_address() -> io::Result<Ipv4Addr> {
    #[cfg(feature = "sim")]
    {
        return Ok(Ipv4Addr::LOCALHOST);
    }
    #[cfg(not(feature = "sim"))]
    unsafe {
        let mut first: *mut libc::ifaddrs = std::ptr::null_mut();
        if libc::getifaddrs(&mut first) != 0 {
            return Err(io::Error::last_os_error());
        }
        let mut current = first;
        let mut found = None;
        while !current.is_null() {
            let entry = &*current;
            if !entry.ifa_addr.is_null()
                && (*entry.ifa_addr).sa_family as i32 == libc::AF_INET
                && std::ffi::CStr::from_ptr(entry.ifa_name).to_bytes() == b"wlan0"
            {
                let addr = &*(entry.ifa_addr as *const libc::sockaddr_in);
                let ip = Ipv4Addr::from(addr.sin_addr.s_addr.to_ne_bytes());
                if !ip.is_unspecified() && !ip.is_loopback() {
                    found = Some(ip);
                    break;
                }
            }
            current = entry.ifa_next;
        }
        libc::freeifaddrs(first);
        found.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "Connect Wi-Fi first, then open File Transfer",
            )
        })
    }
}

struct Client {
    socket: TcpStream,
    peer: SocketAddr,
    root: PathBuf,
    cwd: Vec<String>,
    password: String,
    user_ok: bool,
    authenticated: bool,
    failures: u8,
    passive: Option<TcpListener>,
    rename: Option<PathBuf>,
    shared: Arc<Shared>,
    pending: Vec<u8>,
}
impl Client {
    fn new(
        socket: TcpStream,
        peer: SocketAddr,
        root: PathBuf,
        password: String,
        shared: Arc<Shared>,
    ) -> io::Result<Self> {
        socket.set_read_timeout(Some(IO_TICK))?;
        socket.set_write_timeout(Some(IO_TICK))?;
        Ok(Self {
            socket,
            peer,
            root,
            cwd: Vec::new(),
            password,
            user_ok: false,
            authenticated: false,
            failures: 0,
            passive: None,
            rename: None,
            shared,
            pending: Vec::new(),
        })
    }
    fn reply(&mut self, code: u16, text: &str) -> io::Result<()> {
        self.socket
            .write_all(format!("{code} {text}\r\n").as_bytes())
    }
    fn command(&mut self) -> io::Result<Option<String>> {
        let mut last = Instant::now();
        loop {
            self.shared.active()?;
            if let Some(end) = self.pending.iter().position(|b| *b == b'\n') {
                if end >= COMMAND_MAX {
                    return Err(io::Error::other("Command too long"));
                }
                let line: Vec<_> = self.pending.drain(..=end).collect();
                return String::from_utf8(line)
                    .map(|s| Some(s.trim_end_matches(['\r', '\n']).into()))
                    .map_err(|_| io::Error::other("Use UTF-8 names"));
            }
            if self.pending.len() >= COMMAND_MAX {
                return Err(io::Error::other("Command too long"));
            }
            let mut buf = [0; 512];
            match self.socket.read(&mut buf) {
                Ok(0) => return Ok(None),
                Ok(n) => {
                    self.pending.extend_from_slice(&buf[..n]);
                    last = Instant::now();
                }
                Err(e) if timed_out(&e) && last.elapsed() < CONTROL_IDLE => continue,
                Err(e) => return Err(e),
            }
        }
    }
    fn run(&mut self) -> io::Result<()> {
        self.reply(
            220,
            "Slot file transfer. Log in with the password on your Brick.",
        )?;
        while let Some(line) = self.command()? {
            let (command, arg) = line.split_once(' ').unwrap_or((&line, ""));
            let command = command.to_ascii_uppercase();
            if command != "RNTO" && command != "RNFR" {
                self.rename = None;
            }
            match command.as_str() {
                "QUIT" => { self.reply(221, "Goodbye")?; break; }
                "USER" => { self.user_ok = arg == USER; self.authenticated = false; self.passive = None; self.reply(331, "Password required")?; }
                "PASS" => {
                    self.authenticated = self.user_ok && arg == self.password;
                    if self.authenticated { self.reply(230, "Logged in")?; }
                    else { self.failures += 1; thread::sleep(IO_TICK); self.reply(530, "Login incorrect")?; if self.failures >= 3 { break; } }
                }
                "FEAT" => self.socket.write_all(b"211-Features\r\n UTF8\r\n EPSV\r\n PASV\r\n SIZE\r\n MDTM\r\n MLST type*;size*;modify*;\r\n211 End\r\n")?,
                "SYST" => self.reply(215, "UNIX Type: L8")?,
                "NOOP" => self.reply(200, "OK")?,
                _ if !self.authenticated => self.reply(530, "Log in first")?,
                "OPTS" if arg.eq_ignore_ascii_case("UTF8 ON") || arg.to_ascii_uppercase().starts_with("MLST ") => self.reply(200, "OK")?,
                "TYPE" if matches!(arg, "I" | "A") => self.reply(200, "Type set")?,
                "PWD" | "XPWD" => self.reply(257, &format!("\"/{}\"", self.cwd.join("/")))?,
                "CWD" | "XCWD" | "CDUP" => {
                    let arg = if command == "CDUP" { ".." } else { arg };
                    match self.resolve(arg, false) {
                        Ok((parts, path)) if path.is_dir() => { self.cwd = parts; self.reply(250, "Directory changed")?; }
                        _ => self.reply(550, "Directory unavailable")?,
                    }
                }
                "PASV" | "EPSV" => self.passive(command == "EPSV")?,
                "LIST" | "NLST" | "MLSD" => {
                    let arg = strip_list_options(arg);
                    let result = self.listing(arg, &command);
                    match result {
                        Ok(bytes) => {
                            if let Some(mut data) = self.data()? {
                                let result = write_data(&mut data, &bytes, &self.shared);
                                drop(data); self.transfer_reply(result.map(|_| ()))?;
                            }
                        }
                        Err(_) => self.reply(550, "Directory unavailable")?,
                    }
                }
                "SIZE" | "MDTM" | "MLST" => {
                    match self.resolve(arg, false).and_then(|(_, p)| fs::metadata(&p).map(|m| (p, m))) {
                        Ok((path, meta)) => {
                            if command == "MLST" {
                                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("/");
                                self.socket.write_all(format!("250-Listing\r\n {}\r\n250 End\r\n", facts(name, &meta)).as_bytes())?;
                            } else if meta.is_file() { self.reply(213, &if command == "SIZE" { meta.len().to_string() } else { modified(&meta) })?; }
                            else { self.reply(550, "Not a file")?; }
                        }
                        Err(_) => self.reply(550, "File unavailable")?,
                    }
                }
                "RETR" => self.download(arg)?,
                "STOR" => self.upload(arg)?,
                "MKD" | "XMKD" => {
                    let result = self.resolve(arg, true).and_then(|(parts, path)| {
                        if parts.len() < 2 { return Err(io::Error::other("Protected directory")); }
                        let _guard = self.shared.mutation.lock().unwrap_or_else(|e| e.into_inner());
                        fs::create_dir(path)
                    });
                    if result.is_ok() { self.shared.changed.store(true, Ordering::Relaxed); self.reply(257, "Directory created")?; }
                    else { self.reply(550, "Cannot create directory")?; }
                }
                "DELE" | "RMD" | "XRMD" => {
                    let result = self.resolve(arg, false).and_then(|(parts, path)| {
                        if parts.len() < 2 { return Err(io::Error::other("Protected directory")); }
                        let _guard = self.shared.mutation.lock().unwrap_or_else(|e| e.into_inner());
                        if command == "DELE" { fs::remove_file(path) } else { fs::remove_dir(path) }
                    });
                    if result.is_ok() { self.shared.changed.store(true, Ordering::Relaxed); self.reply(250, "Removed")?; }
                    else { self.reply(550, "Cannot remove item")?; }
                }
                "RNFR" => {
                    self.rename = self.resolve(arg, false).ok().filter(|(parts, _)| parts.len() >= 2).map(|(_, p)| p);
                    if self.rename.is_some() { self.reply(350, "Send RNTO")?; } else { self.reply(550, "Cannot rename item")?; }
                }
                "RNTO" => {
                    let result = self.rename.take().ok_or_else(|| io::Error::other("Send RNFR first")).and_then(|from| {
                        let (parts, to) = self.resolve(arg, true)?;
                        if parts.len() < 2 { return Err(io::Error::other("Protected directory")); }
                        let _guard = self.shared.mutation.lock().unwrap_or_else(|e| e.into_inner());
                        fs::rename(from, to)
                    });
                    if result.is_ok() { self.shared.changed.store(true, Ordering::Relaxed); self.reply(250, "Renamed")?; }
                    else { self.reply(550, "Cannot rename item")?; }
                }
                "ALLO" => self.reply(202, "Allocation not required")?,
                "ABOR" => { self.passive = None; self.reply(226, "No transfer pending")?; }
                _ => self.reply(502, "Command unsupported; use passive FTP and full-file transfers")?,
            }
        }
        Ok(())
    }
    fn resolve(&self, arg: &str, new: bool) -> io::Result<(Vec<String>, PathBuf)> {
        resolve(&self.root, &self.cwd, arg, new)
    }
    fn passive(&mut self, extended: bool) -> io::Result<()> {
        self.passive = None;
        let ip = self.socket.local_addr()?.ip();
        let listener = TcpListener::bind((ip, 0))?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        self.passive = Some(listener);
        if extended {
            self.reply(229, &format!("Entering Extended Passive Mode (|||{port}|)"))
        } else if let IpAddr::V4(ip) = ip {
            let b = ip.octets();
            self.reply(
                227,
                &format!(
                    "Entering Passive Mode ({},{},{},{},{},{})",
                    b[0],
                    b[1],
                    b[2],
                    b[3],
                    port / 256,
                    port % 256
                ),
            )
        } else {
            self.reply(522, "Use EPSV")
        }
    }
    fn data(&mut self) -> io::Result<Option<TcpStream>> {
        let Some(listener) = self.passive.take() else {
            self.reply(425, "Use PASV or EPSV first")?;
            return Ok(None);
        };
        self.reply(150, "Opening data connection")?;
        let started = Instant::now();
        while started.elapsed() < DATA_IDLE {
            self.shared.active()?;
            match listener.accept() {
                Ok((socket, peer)) if peer.ip() == self.peer.ip() => {
                    socket.set_read_timeout(Some(IO_TICK))?;
                    socket.set_write_timeout(Some(IO_TICK))?;
                    return Ok(Some(socket));
                }
                Ok(_) => {} // Never send a player's files to a different peer.
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20))
                }
                Err(_) => break,
            }
        }
        self.reply(425, "Data connection timed out")?;
        Ok(None)
    }
    fn listing(&self, arg: &str, mode: &str) -> io::Result<Vec<u8>> {
        let (parts, path) = self.resolve(arg, false)?;
        let mut rows = Vec::new();
        if path.is_dir() {
            for entry in fs::read_dir(&path)? {
                let entry = entry?;
                let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                    continue;
                };
                if !valid_name(&name) || (parts.is_empty() && !ROOTS.contains(&name.as_str())) {
                    continue;
                }
                let meta = fs::symlink_metadata(entry.path())?;
                if meta.file_type().is_symlink() || (!meta.is_file() && !meta.is_dir()) {
                    continue;
                }
                rows.push((name, meta));
                if rows.len() > 50_000 {
                    return Err(io::Error::other("Directory too large"));
                }
            }
        } else {
            rows.push((
                parts.last().cloned().unwrap_or_default(),
                fs::metadata(path)?,
            ));
        }
        rows.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        let mut out = String::new();
        for (name, meta) in rows {
            let line = match mode {
                "NLST" => name,
                "MLSD" => facts(&name, &meta),
                _ => format!(
                    "{} 1 slot slot {} {} {}",
                    if meta.is_dir() {
                        "drwxr-xr-x"
                    } else {
                        "-rw-r--r--"
                    },
                    meta.len(),
                    list_date(&meta),
                    name
                ),
            };
            out.push_str(&line);
            out.push_str("\r\n");
        }
        Ok(out.into_bytes())
    }
    fn download(&mut self, arg: &str) -> io::Result<()> {
        let file = self.resolve(arg, false).and_then(|(_, p)| {
            if !p.is_file() {
                return Err(io::Error::other("Not a file"));
            }
            File::open(p)
        });
        let Ok(mut file) = file else {
            return self.reply(550, "File unavailable");
        };
        let Some(mut data) = self.data()? else {
            return Ok(());
        };
        self.shared
            .activity(format!("Sending {}", clean_activity(arg)));
        let result = (|| {
            let mut buffer = [0; 64 * 1024];
            loop {
                self.shared.active()?;
                let n = file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                write_data(&mut data, &buffer[..n], &self.shared)?;
                self.shared
                    .downloaded
                    .fetch_add(n as u64, Ordering::Relaxed);
            }
            Ok(())
        })();
        drop(data);
        self.transfer_reply(result)
    }
    fn upload(&mut self, arg: &str) -> io::Result<()> {
        let target = self.resolve(arg, true).and_then(|(parts, path)| {
            if parts.len() < 2 || path.is_dir() {
                return Err(io::Error::other("Choose a file inside a folder"));
            }
            Ok(path)
        });
        let Ok(target) = target else {
            return self.reply(550, "Destination unavailable");
        };
        let tmp = target.with_file_name(format!(
            ".slot-upload-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        let Ok(mut file) = OpenOptions::new().write(true).create_new(true).open(&tmp) else {
            return self.reply(550, "Cannot write to card");
        };
        let cleanup = Temporary(tmp);
        let Some(mut data) = self.data()? else {
            return Ok(());
        };
        self.shared
            .activity(format!("Receiving {}", clean_activity(arg)));
        let result = (|| {
            let mut total = 0u64;
            let mut buffer = [0; 64 * 1024];
            let mut last = Instant::now();
            loop {
                self.shared.active()?;
                match data.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        total += n as u64;
                        if total > MAX_FILE {
                            return Err(io::Error::other("File exceeds card limit"));
                        }
                        file.write_all(&buffer[..n])?;
                        self.shared.uploaded.fetch_add(n as u64, Ordering::Relaxed);
                        last = Instant::now();
                    }
                    Err(e) if timed_out(&e) && last.elapsed() < DATA_IDLE => continue,
                    Err(e) => return Err(e),
                }
            }
            file.sync_all()?;
            self.shared.active()?;
            let _guard = self
                .shared
                .mutation
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            // Resolve again: another authenticated client may have moved a parent directory.
            let (_, destination) = self.resolve(arg, true)?;
            fs::rename(&cleanup.0, &destination)?;
            self.shared.changed.store(true, Ordering::Relaxed);
            if let Some(parent) = destination.parent() {
                if let Ok(dir) = File::open(parent) {
                    let _ = dir.sync_all();
                }
            }
            Ok(())
        })();
        drop(file);
        drop(data);
        drop(cleanup);
        self.transfer_reply(result)
    }
    fn transfer_reply(&mut self, result: io::Result<()>) -> io::Result<()> {
        match result {
            Ok(()) => {
                self.shared.activity("Transfer complete".into());
                self.reply(226, "Transfer complete")
            }
            Err(_) => {
                self.shared
                    .activity("Transfer interrupted; retry the file".into());
                self.reply(426, "Transfer failed; check free space and retry")
            }
        }
    }
}
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn timed_out(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
    )
}
fn write_data(socket: &mut TcpStream, mut bytes: &[u8], shared: &Shared) -> io::Result<()> {
    let mut last = Instant::now();
    while !bytes.is_empty() {
        shared.active()?;
        match socket.write(bytes) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "Connection closed",
                ))
            }
            Ok(n) => {
                bytes = &bytes[n..];
                last = Instant::now();
            }
            Err(e) if timed_out(&e) && last.elapsed() < DATA_IDLE => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && !name.starts_with('.')
        && !name.ends_with([' ', '.'])
        && !name
            .chars()
            .any(|c| c.is_control() || "\\/:*?\"<>|".contains(c))
}
fn resolve(
    root: &Path,
    cwd: &[String],
    arg: &str,
    new: bool,
) -> io::Result<(Vec<String>, PathBuf)> {
    let denied = || io::Error::new(io::ErrorKind::PermissionDenied, "Outside player folders");
    let mut parts = if arg.starts_with('/') {
        Vec::new()
    } else {
        cwd.to_vec()
    };
    for part in arg.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(denied());
                }
            }
            name if valid_name(name) => parts.push(name.into()),
            _ => return Err(denied()),
        }
    }
    if parts.first().is_some_and(|p| !ROOTS.contains(&p.as_str())) {
        return Err(denied());
    }
    let mut path = root.to_path_buf();
    for (i, part) in parts.iter().enumerate() {
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(m) if m.file_type().is_symlink() || (!m.is_file() && !m.is_dir()) => {
                return Err(denied())
            }
            Ok(m) if i + 1 < parts.len() && !m.is_dir() => return Err(denied()),
            Ok(_) => {}
            Err(e) if new && i + 1 == parts.len() && e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok((parts, path))
}
fn strip_list_options(mut arg: &str) -> &str {
    while let Some((option, rest)) = arg.split_once(' ') {
        if !matches!(option, "-a" | "-l" | "-la" | "-al") {
            break;
        }
        arg = rest;
    }
    if matches!(arg, "-a" | "-l" | "-la" | "-al") {
        ""
    } else {
        arg
    }
}
fn modified(meta: &fs::Metadata) -> String {
    let secs = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs() as i64);
    slot_store::format_stamp(secs)
        .chars()
        .filter(char::is_ascii_digit)
        .collect()
}
fn facts(name: &str, meta: &fs::Metadata) -> String {
    format!(
        "type={};size={};modify={}; {name}",
        if meta.is_dir() { "dir" } else { "file" },
        meta.len(),
        modified(meta)
    )
}
fn clean_activity(name: &str) -> String {
    name.chars().filter(|c| !c.is_control()).take(50).collect()
}

fn list_date(meta: &fs::Metadata) -> String {
    let stamp = modified(meta);
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month = stamp[4..6].parse::<usize>().unwrap_or(1).clamp(1, 12);
    format!("{} {} {}", months[month - 1], &stamp[6..8], &stamp[..4])
}
