use slot::file_transfer::{Server, USER};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::{Duration, Instant};

fn server() -> (tempfile::TempDir, Server) {
    let root = tempfile::tempdir().unwrap();
    slot::root::ensure(&root.path().join("Slot"));
    let server = Server::start_card(&root.path().join("Slot"), "127.0.0.1:0".parse().unwrap()).unwrap();
    (root, server)
}

struct Ftp(BufReader<TcpStream>);
impl Ftp {
    fn connect(server: &Server) -> Self {
        let socket = TcpStream::connect(server.address()).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut ftp = Self(BufReader::new(socket));
        ftp.reply(220);
        ftp
    }
    fn login(server: &Server) -> Self {
        let mut ftp = Self::connect(server);
        ftp.cmd(&format!("USER {USER}"), 331);
        ftp.cmd(&format!("PASS {}", server.password()), 230);
        ftp
    }
    fn reply(&mut self, code: u16) -> String {
        let mut line = String::new();
        self.0.read_line(&mut line).unwrap();
        assert!(
            line.starts_with(&format!("{code} ")),
            "{line:?}, expected {code}"
        );
        line
    }
    fn cmd(&mut self, command: &str, code: u16) -> String {
        write!(self.0.get_mut(), "{command}\r\n").unwrap();
        self.reply(code)
    }
    fn passive(&mut self, extended: bool) -> TcpStream {
        let port = if extended {
            self.cmd("EPSV", 229)
                .split('|')
                .nth(3)
                .unwrap()
                .parse()
                .unwrap()
        } else {
            let line = self.cmd("PASV", 227);
            let numbers: Vec<u16> = line
                .split('(')
                .nth(1)
                .unwrap()
                .split(')')
                .next()
                .unwrap()
                .split(',')
                .map(|s| s.parse().unwrap())
                .collect();
            assert_eq!(&numbers[..4], &[127, 0, 0, 1]);
            numbers[4] * 256 + numbers[5]
        };
        let data = TcpStream::connect(("127.0.0.1", port)).unwrap();
        data.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        data
    }
    fn get(&mut self, command: &str, extended: bool) -> Vec<u8> {
        let mut data = self.passive(extended);
        self.cmd(command, 150);
        let mut bytes = Vec::new();
        data.read_to_end(&mut bytes).unwrap();
        self.reply(226);
        bytes
    }
    fn put(&mut self, path: &str, bytes: &[u8]) {
        let mut data = self.passive(false);
        self.cmd(&format!("STOR {path}"), 150);
        data.write_all(bytes).unwrap();
        data.shutdown(Shutdown::Write).unwrap();
        drop(data);
        self.reply(226);
    }
}

#[test]
fn requires_screen_password_and_exposes_the_whole_card() {
    let (root, mut server) = server();
    for path in ["Data/ports/balatro", "Apps/PortMaster", "Imgs/PORTS", "Roms/PORTS", "Slot/Logs", "Slot/System"] {
        fs::create_dir_all(root.path().join(path)).unwrap();
    }
    fs::write(root.path().join("Slot/Logs/balatro-patch.log"), "patch error").unwrap();
    let mut ftp = Ftp::connect(&server);
    ftp.cmd("PWD", 530);
    ftp.cmd("USER anonymous", 331);
    ftp.cmd("PASS nobody", 530);
    ftp.cmd("USER slot", 331);
    ftp.cmd("PASS wrong", 530);
    ftp.cmd(&format!("PASS {}", server.password()), 230);
    let list = String::from_utf8(ftp.get("NLST", false)).unwrap();
    assert_eq!(list, "Apps\r\nData\r\nImgs\r\nRoms\r\nSlot\r\n");
    assert_eq!(ftp.get("RETR /Slot/Logs/balatro-patch.log", true), b"patch error");
    ftp.put("/Data/ports/balatro/Balatro.exe", b"game fixture");
    ftp.put("/Data/ports/balatro/.balatro-build.txt", b"build stamp");
    ftp.put("/Apps/PortMaster/launcher.sh", b"port launcher");
    ftp.put("/Slot/System/helper.sh", b"slot helper");
    ftp.put("/Slot/Config/test.txt", b"setting");
    ftp.put("/root-file.txt", b"root file");
    assert_eq!(ftp.get("RETR /Data/ports/balatro/.balatro-build.txt", false), b"build stamp");
    ftp.cmd("RNFR /root-file.txt", 350);
    ftp.cmd("RNTO /renamed.txt", 250);
    ftp.cmd("DELE /renamed.txt", 250);
    ftp.cmd("MKD /NewFolder", 257);
    ftp.cmd("RMD /NewFolder", 250);
    for command in ["CWD ../../", "STOR /Slot/../../escape", "RMD /", "RNFR /",
                    "STOR /Slot/Games/bad\\name", "STOR /.slot-upload-hidden"] {
        ftp.cmd(command, 550);
    }
    ftp.cmd("PORT 127,0,0,1,10,10", 502);
    assert!(server.stop());
}

#[test]
fn passive_upload_download_listing_and_file_management() {
    let (root, mut server) = server();
    let mut ftp = Ftp::login(&server);
    ftp.cmd("TYPE I", 200);
    ftp.cmd("CWD /Slot/Games/GG", 250);
    let bytes: Vec<_> = (0..2_000_000).map(|i| (i % 251) as u8).collect();
    ftp.put("Sonic café.gg", &bytes);
    assert_eq!(
        fs::read(root.path().join("Slot/Games/GG/Sonic café.gg")).unwrap(),
        bytes
    );
    assert_eq!(ftp.get("RETR Sonic café.gg", true), bytes);
    assert!(ftp.cmd("SIZE Sonic café.gg", 213).contains("2000000"));
    let stamp = ftp.cmd("MDTM Sonic café.gg", 213);
    assert_eq!(stamp.trim().len(), 18);
    let facts = String::from_utf8(ftp.get("MLSD", true)).unwrap();
    assert!(facts.contains("type=file;size=2000000;modify="));
    assert!(facts.ends_with("; Sonic café.gg\r\n"));
    let list = String::from_utf8(ftp.get("LIST -la", false)).unwrap();
    assert!(list.starts_with("-rw-r--r-- 1 slot slot 2000000 "));
    ftp.cmd("MKD Folder", 257);
    ftp.cmd("RNFR Sonic café.gg", 350);
    ftp.cmd("RNTO Folder/Renamed.gg", 250);
    ftp.cmd("CWD Folder", 250);
    assert_eq!(ftp.cmd("PWD", 257).trim(), "257 \"/Slot/Games/GG/Folder\"");
    ftp.cmd("DELE Renamed.gg", 250);
    ftp.cmd("CDUP", 250);
    ftp.cmd("RMD Folder", 250);
    ftp.put("Empty.gg", b"");
    assert!(ftp.get("RETR Empty.gg", false).is_empty());
    assert_eq!(server.progress().uploaded, bytes.len() as u64);
    assert_eq!(server.progress().downloaded, bytes.len() as u64);
    assert!(server.stop());
}

#[test]
fn stop_discards_in_progress_upload_and_preserves_existing_file() {
    let (root, mut server) = server();
    let destination = root.path().join("Slot/Saves/GG/Sonic.sav");
    fs::write(&destination, b"original save").unwrap();
    let mut ftp = Ftp::login(&server);
    let mut data = ftp.passive(true);
    ftp.cmd("STOR /Slot/Saves/GG/Sonic.sav", 150);
    data.write_all(&[42; 100_000]).unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"original save");
    let start = Instant::now();
    assert!(!server.stop());
    assert!(start.elapsed() < Duration::from_secs(2));
    assert_eq!(fs::read(&destination).unwrap(), b"original save");
    assert_eq!(
        fs::read_dir(destination.parent().unwrap()).unwrap().count(),
        1
    );
    assert!(TcpStream::connect(server.address()).is_err());
}

#[test]
fn stopped_server_closes_idle_clients_and_pending_passive_accepts() {
    let (_root, mut server) = server();
    let mut idle = Ftp::login(&server);
    let mut passive = Ftp::login(&server);
    passive.cmd("EPSV", 229);
    passive.cmd("STOR /Slot/Games/GG/Pending.gg", 150);
    let start = Instant::now();
    server.stop();
    assert!(start.elapsed() < Duration::from_secs(2));
    let mut byte = [0];
    assert_eq!(idle.0.read(&mut byte).unwrap(), 0);
}

#[test]
fn symlinks_cannot_expose_files_outside_the_card() {
    let (root, mut server) = server();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("private"), b"private").unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join("Slot/Games/escape")).unwrap();
    let mut ftp = Ftp::login(&server);
    ftp.cmd("RETR /Slot/Games/escape/private", 550);
    ftp.cmd("STOR /Slot/Games/escape/private", 550);
    ftp.cmd("CWD /Slot/Games/escape", 550);
    assert!(!String::from_utf8(ftp.get("NLST /Slot/Games", true))
        .unwrap()
        .contains("escape"));
    assert!(!server.stop());
    assert_eq!(
        fs::read(outside.path().join("private")).unwrap(),
        b"private"
    );
}

#[test]
fn simultaneous_clients_and_oversized_commands() {
    let (_root, mut server) = server();
    let mut first = Ftp::login(&server);
    let mut second = Ftp::login(&server);
    first.put("/Slot/Games/GG/One.gg", b"one");
    second.put("/Slot/Games/SMS/Two.sms", b"two");
    assert_eq!(first.get("RETR /Slot/Games/SMS/Two.sms", false), b"two");
    assert_eq!(second.get("RETR /Slot/Games/GG/One.gg", true), b"one");
    second
        .0
        .get_mut()
        .write_all(format!("NOOP {}\r\n", "a".repeat(4500)).as_bytes())
        .unwrap();
    let mut byte = [0];
    match second.0.read(&mut byte) {
        Ok(0) => {}
        Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {}
        result => panic!("oversized command was not rejected: {result:?}"),
    }
    assert!(server.stop());
}
