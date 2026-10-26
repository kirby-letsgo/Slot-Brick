//! ScreenScraper's authenticated hash lookup. Credentials and URLs never enter logs.
use quick_xml::{events::Event, Reader};
use ring::digest::{Context, SHA1_FOR_LEGACY_USE_ONLY};
use slot_store::{Cart, Platform};
use std::{
    fs,
    io::Read,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub const ACCOUNT_FILE: &str = "Config/screenscraper.txt";
const API: &str = "https://api.screenscraper.fr";
#[derive(Clone, Default)]
pub struct Account {
    pub user: String,
    pub password: String,
    pub developer_id: String,
    pub developer_password: String,
}
impl Account {
    pub fn load(root: &Path) -> Self {
        let mut a = Self {
            developer_id: option_env!("SLOT_SCREENSCRAPER_DEVID").unwrap_or("").into(),
            developer_password: option_env!("SLOT_SCREENSCRAPER_DEVPASSWORD")
                .unwrap_or("")
                .into(),
            ..Self::default()
        };
        for line in fs::read_to_string(root.join(ACCOUNT_FILE))
            .unwrap_or_default()
            .lines()
        {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key {
                "username" => a.user = value.into(),
                "password" => a.password = value.into(),
                "developer_id" if !value.is_empty() => a.developer_id = value.into(),
                "developer_password" if !value.is_empty() => a.developer_password = value.into(),
                _ => {}
            }
        }
        a
    }
    pub fn ready(&self) -> bool {
        self.app_ready() && !self.user.is_empty() && !self.password.is_empty()
    }
    pub fn app_ready(&self) -> bool {
        !self.developer_id.is_empty() && !self.developer_password.is_empty()
    }
    pub fn save(&self, root: &Path) -> std::io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let fields = [
            ("username", &self.user),
            ("password", &self.password),
            ("developer_id", &self.developer_id),
            ("developer_password", &self.developer_password),
        ];
        if fields.iter().any(|(_, v)| v.contains(['\n', '\r'])) {
            return Err(std::io::Error::other("Invalid account details"));
        }
        let path = root.join(ACCOUNT_FILE);
        fs::create_dir_all(path.parent().unwrap())?;
        slot_store::atomic_write(
            &path,
            fields
                .iter()
                .map(|(k, v)| format!("{k}={v}\n"))
                .collect::<String>()
                .as_bytes(),
        )?;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
        Ok(())
    }
    fn query(&self) -> String {
        let e = crate::net::form_escape;
        format!(
            "devid={}&devpassword={}&softname=Slot-Brick&ssid={}&sspassword={}",
            e(&self.developer_id),
            e(&self.developer_password),
            e(&self.user),
            e(&self.password)
        )
    }
}
pub struct Media {
    pub kind: String,
    pub region: String,
    pub url: String,
}
pub struct Lookup {
    pub media: Vec<Media>,
}
pub fn lookup(
    cart: &Cart,
    account: &Account,
    cancel: &AtomicBool,
) -> Result<Option<Lookup>, String> {
    lookup_at(cart, account, cancel, API)
}
fn lookup_at(
    cart: &Cart,
    account: &Account,
    cancel: &AtomicBool,
    api: &str,
) -> Result<Option<Lookup>, String> {
    if !account.ready() {
        return Err("ScreenScraper account and app credentials are required".into());
    }
    let hash = hash_rom(cart, cancel)?;
    lookup_hash_at(cart, account, api, &hash)
}
pub(crate) type RomHash = (String, u64, String);
pub(crate) fn lookup_hashed(
    cart: &Cart,
    account: &Account,
    hash: &RomHash,
) -> Result<Option<Lookup>, String> {
    lookup_hash_at(cart, account, API, hash)
}
fn lookup_hash_at(
    cart: &Cart,
    account: &Account,
    api: &str,
    hash: &RomHash,
) -> Result<Option<Lookup>, String> {
    if !account.ready() {
        return Err("ScreenScraper account and app credentials are required".into());
    }
    let (sha, size, name) = hash;
    let system = if cart.platform == Platform::Ngp && name.to_ascii_lowercase().ends_with(".ngc") {
        82
    } else {
        system_id(cart.platform)
    };
    let path=format!("/api2/jeuInfos.php?{}&output=xml&systemeid={}&romtype=rom&sha1={sha}&romtaille={size}&romnom={}",account.query(),system,crate::net::form_escape(name));
    let reply = request(api, &path, 4 << 20)?;
    match reply.status {
        404 => return Ok(None),
        401 | 403 => return Err("ScreenScraper rejected the account or app credentials".into()),
        429 | 430 | 431 => return Err("ScreenScraper limit reached; retry later".into()),
        200 => {}
        _ => return Err(format!("ScreenScraper unavailable (HTTP {})", reply.status)),
    }
    let xml = std::str::from_utf8(&reply.body)
        .map_err(|_| "ScreenScraper returned unreadable game data")?;
    parse(xml).map(|v| v.map(|media| Lookup { media }))
}
fn system_id(p: Platform) -> u16 {
    match p {
        Platform::Md => 1,
        Platform::S32x => 19,
        Platform::SegaCd => 20,
        Platform::Pce => 31,
        Platform::PceCd => 114,
        Platform::Sms => 2,
        Platform::N64 => 14,
        Platform::Ngp => 25,
        Platform::Pico8 => 234,
        Platform::Nes => 3,
        Platform::Snes => 4,
        Platform::Gb => 9,
        Platform::Gbc => 10,
        Platform::Gba => 12,
        Platform::Gg => 21,
        Platform::Ps1 => 57,
    }
}
pub(crate) fn hash_rom(cart: &Cart, cancel: &AtomicBool) -> Result<RomHash, String> {
    // Do not identify a multi-file disc by the hash of its tiny playlist.
    if cart
        .rom
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cue") || e.eq_ignore_ascii_case("m3u"))
    {
        return Err("ScreenScraper hash lookup needs a single ROM or disc image".into());
    }
    let hash = |reader: &mut dyn Read| -> std::io::Result<(String, u64)> {
        let mut context = Context::new(&SHA1_FOR_LEGACY_USE_ONLY);
        let mut size = 0;
        let mut buffer = [0; 64 * 1024];
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Cancelled",
                ));
            }
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            size += n as u64;
            context.update(&buffer[..n]);
        }
        Ok((
            context
                .finish()
                .as_ref()
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect(),
            size,
        ))
    };
    let result = if slot_store::archive::is_archive(&cart.rom) {
        slot_store::archive::with_rom(&cart.rom, |entry, reader| {
            hash(reader).map(|(sha, size)| (sha, size, entry.name.clone()))
        })
    } else {
        fs::File::open(&cart.rom)
            .and_then(|mut f| hash(&mut f))
            .map(|(sha, size)| {
                (
                    sha,
                    size,
                    cart.rom
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                )
            })
    };
    result.map_err(|e| {
        if e.kind() == std::io::ErrorKind::Interrupted {
            "Cancelled".into()
        } else {
            "Cannot hash this ROM for ScreenScraper".into()
        }
    })
}
fn request(origin: &str, path: &str, limit: usize) -> Result<crate::net::Response, String> {
    let (tls, server, _) = crate::net::split_url(origin).ok_or("Invalid ScreenScraper address")?;
    crate::net::send(&crate::net::Request {
        tls,
        server: &server,
        method: "GET",
        path,
        form: None,
        limit,
    })
    .map_err(|_| "Cannot reach ScreenScraper".into())
}
fn trusted(url: &str) -> bool {
    crate::net::split_url(url).is_some_and(|(tls, server, _)| {
        let host = crate::net::host_of(&server);
        tls && (host == "screenscraper.fr" || host.ends_with(".screenscraper.fr"))
    })
}
pub fn download(url: &str, cancel: &AtomicBool) -> Result<Vec<u8>, String> {
    let mut url = url.to_string();
    for _ in 0..3 {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        if !trusted(&url) {
            return Err("ScreenScraper returned an unsupported media address".into());
        }
        let (_, _, path) = crate::net::split_url(&url).unwrap();
        let p = url.find("://").unwrap() + 3;
        let end = url[p..].find('/').map_or(url.len(), |i| p + i);
        let origin = url[..end].to_string();
        let reply = request(&origin, &path, 8 << 20)?;
        match reply.status {
            200 => return Ok(reply.body),
            301 | 302 | 303 | 307 | 308 => {
                let to = reply
                    .header("location")
                    .ok_or("ScreenScraper sent an invalid redirect")?;
                url = if to.starts_with('/') {
                    format!("{origin}{to}")
                } else {
                    to
                };
            }
            429 | 430 | 431 => return Err("ScreenScraper limit reached; retry later".into()),
            _ => return Err("ScreenScraper artwork is unavailable".into()),
        }
    }
    Err("ScreenScraper redirected too often".into())
}
fn parse(xml: &str) -> Result<Option<Vec<Media>>, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut found = false;
    let mut in_game = false;
    let mut current: Option<Media> = None;
    let mut media = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"jeu" => {
                    found = true;
                    in_game = true;
                }
                b"media" if in_game => {
                    let mut m = Media {
                        kind: String::new(),
                        region: String::new(),
                        url: String::new(),
                    };
                    let mut format = String::new();
                    for a in e.attributes() {
                        let a = a.map_err(|_| "Invalid ScreenScraper media")?;
                        let v = a
                            .decoded_and_normalized_value(
                                quick_xml::XmlVersion::Implicit1_0,
                                reader.decoder(),
                            )
                            .map_err(|_| "Invalid ScreenScraper media")?
                            .into_owned();
                        match a.key.as_ref() {
                            b"type" => m.kind = v,
                            b"region" => m.region = v,
                            b"format" => format = v,
                            _ => {}
                        }
                    }
                    if matches!(
                        m.kind.as_str(),
                        "support-texture" | "support-2D" | "support-2d"
                    ) && (format.is_empty() || format.eq_ignore_ascii_case("png"))
                    {
                        current = Some(m)
                    }
                }
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if let Some(m) = &mut current {
                    let s = e
                        .xml_content(quick_xml::XmlVersion::Implicit1_0)
                        .map_err(|_| "Invalid ScreenScraper media")?;
                    m.url.push_str(
                        &quick_xml::escape::unescape(&s)
                            .map_err(|_| "Invalid ScreenScraper media")?,
                    );
                }
            }
            Ok(Event::GeneralRef(e)) => {
                if let Some(m) = &mut current {
                    let entity = e.decode().map_err(|_| "Invalid ScreenScraper media")?;
                    let encoded = format!("&{entity};");
                    m.url.push_str(
                        &quick_xml::escape::unescape(&encoded)
                            .map_err(|_| "Invalid ScreenScraper media")?,
                    );
                }
            }
            Ok(Event::CData(e)) => {
                if let Some(m) = &mut current {
                    m.url
                        .push_str(&e.decode().map_err(|_| "Invalid ScreenScraper media")?);
                }
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"media" => {
                    if let Some(m) = current.take() {
                        if !m.url.is_empty() {
                            media.push(m)
                        }
                    }
                }
                b"jeu" => in_game = false,
                _ => {}
            },
            Ok(Event::Empty(e)) if e.name().as_ref() == b"jeu" => found = true,
            Ok(Event::Eof) => break,
            Err(_) => return Err("ScreenScraper returned invalid game data".into()),
            _ => {}
        }
    }
    if !found {
        return Err(
            "ScreenScraper returned no game response; check account and service status".into(),
        );
    }
    Ok(Some(media))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_roundtrip_requires_both_app_and_player_credentials() {
        let d = tempfile::tempdir().unwrap();
        let mut a = Account {
            user: "player=name".into(),
            password: "secret&+=".into(),
            ..Account::default()
        };
        assert!(!a.ready());
        a.developer_id = "app".into();
        a.developer_password = "app-secret".into();
        assert!(a.ready());
        a.save(d.path()).unwrap();
        let loaded = Account::load(d.path());
        assert!(loaded.ready());
        assert_eq!(loaded.user, a.user);
        assert_eq!(loaded.password, a.password);
        assert!(loaded.query().contains("sspassword=secret%26%2B%3D"));
    }
    #[test]
    fn archives_hash_the_unpacked_rom() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../slot-store/tests/fixtures");
        let mut c = Cart {
            platform: Platform::Gb,
            stem: "renamed".into(),
            rom: fixtures.join("rom.zip"),
            label: None,
            famicom_label: None,
            title: String::new(),
            code: String::new(),
            shell: None,
        };
        c.rom = fixtures.join("rom.gb");
        let plain = hash_rom(&c, &AtomicBool::new(false)).unwrap();
        for archive in ["rom.zip", "rom-lzma.7z", "rom-lzma2.7z"] {
            c.rom = fixtures.join(archive);
            let zipped = hash_rom(&c, &AtomicBool::new(false)).unwrap();
            assert_eq!((&zipped.0, zipped.1), (&plain.0, plain.1), "{archive}");
            assert!(zipped.2.ends_with(".gb"));
        }
    }
    #[test]
    fn api_uses_sha1_system_and_account_and_distinguishes_service_errors() {
        use std::{io::Write, net::TcpListener, thread};
        let d = tempfile::tempdir().unwrap();
        let rom = d.path().join("Sonic & Tails.gg");
        fs::write(&rom, b"abc").unwrap();
        let c = Cart {
            platform: Platform::Gg,
            stem: "Sonic".into(),
            rom,
            label: None,
            famicom_label: None,
            title: String::new(),
            code: String::new(),
            shell: None,
        };
        let account = Account {
            user: "player".into(),
            password: "secret&value".into(),
            developer_id: "app".into(),
            developer_password: "app-secret".into(),
        };
        for status in [200, 404, 403, 430, 500] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = format!("http://{}", listener.local_addr().unwrap());
            let t = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                    .unwrap();
                let mut raw = Vec::new();
                let mut b = [0; 1024];
                while !raw.windows(4).any(|w| w == b"\r\n\r\n") {
                    let n = stream.read(&mut b).unwrap();
                    assert!(n > 0);
                    raw.extend_from_slice(&b[..n]);
                }
                let text = String::from_utf8(raw).unwrap();
                assert!(text.contains("sha1=A9993E364706816ABA3E25717850C26C9CD0D89D"));
                assert!(text.contains("systemeid=21"));
                assert!(text.contains("romtaille=3"));
                assert!(text.contains("sspassword=secret%26value"));
                assert!(text.contains("romnom=Sonic%20%26%20Tails.gg"));
                let body="<Data><jeu><medias><media type=\"support-texture\" format=\"png\">https://www.screenscraper.fr/label.png</media></medias></jeu></Data>";
                write!(stream,"HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
            });
            let r = lookup_at(&c, &account, &AtomicBool::new(false), &addr);
            t.join().unwrap();
            match status {
                200 => assert_eq!(r.unwrap().unwrap().media.len(), 1),
                404 => assert!(r.unwrap().is_none()),
                _ => {
                    let error = r.err().unwrap();
                    assert!(!error.contains("secret"));
                    assert!(!error.contains("http"));
                }
            }
        }
    }
    #[test]
    fn prehashed_lookups_keep_new_console_ids_and_the_unpacked_filename() {
        use std::{io::Write, net::TcpListener, thread};
        let account = Account {
            user: "player".into(),
            password: "test".into(),
            developer_id: "app".into(),
            developer_password: "test".into(),
        };
        for (platform, name, system) in [
            (Platform::N64, "Game.z64", 14),
            (Platform::Ngp, "Game.ngp", 25),
            (Platform::Ngp, "Game.NGC", 82),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let cart = Cart {
                platform,
                stem: "Renamed archive".into(),
                // No ROM is present: the fallback must reuse the worker's hash.
                rom: dir.path().join("Renamed archive.zip"),
                label: None,
                famicom_label: None,
                title: String::new(),
                code: String::new(),
                shell: None,
            };
            let hash = (
                "A9993E364706816ABA3E25717850C26C9CD0D89D".into(),
                3,
                name.into(),
            );
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = format!("http://{}", listener.local_addr().unwrap());
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                    .unwrap();
                let mut raw = Vec::new();
                let mut buffer = [0; 1024];
                while !raw.windows(4).any(|w| w == b"\r\n\r\n") {
                    let n = stream.read(&mut buffer).unwrap();
                    assert!(n > 0);
                    raw.extend_from_slice(&buffer[..n]);
                }
                write!(
                    stream,
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
                .unwrap();
                String::from_utf8(raw).unwrap()
            });
            assert!(lookup_hash_at(&cart, &account, &addr, &hash)
                .unwrap()
                .is_none());
            let request = server.join().unwrap();
            assert!(request.contains(&format!("systemeid={system}&")));
            assert!(request.contains(&format!("romnom={name}")));
            assert!(request.contains(&format!("sha1={}&romtaille=3", hash.0)));
        }
    }

    #[test]
    fn media_parser_keeps_cartridge_art_and_decodes_urls() {
        let xml = r#"<Data><jeu id="1"><medias><media type="box-2D" format="png">https://bad/box</media><media type="support-texture" region="us" format="png">https://www.screenscraper.fr/art?a=1&amp;b=2</media></medias></jeu></Data>"#;
        let v = parse(xml).unwrap().unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].url, "https://www.screenscraper.fr/art?a=1&b=2");
        assert!(!trusted("https://screenscraper.fr.evil.test/a"));
        assert!(!trusted("http://www.screenscraper.fr/a"));
    }
    #[test]
    fn hashes_are_streamed_and_cancellable() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.gg");
        fs::write(&p, b"abc").unwrap();
        let c = Cart {
            platform: Platform::Gg,
            stem: "a".into(),
            rom: p,
            label: None,
            famicom_label: None,
            title: String::new(),
            code: String::new(),
            shell: None,
        };
        let h = hash_rom(&c, &AtomicBool::new(false)).unwrap();
        assert_eq!(h.0, "A9993E364706816ABA3E25717850C26C9CD0D89D");
        assert_eq!(h.1, 3);
        assert!(hash_rom(&c, &AtomicBool::new(true)).is_err());
    }
}
