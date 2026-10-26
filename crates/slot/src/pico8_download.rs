//! The small wget interface used by PICO-8's Linux cart downloader.
//! Reuse Slot's TLS roots instead of depending on stock firmware wget/libcurl.
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
struct Download { url: String, output: PathBuf }

fn parse(args: impl IntoIterator<Item = String>) -> Result<Download, String> {
    let mut args = args.into_iter();
    let mut output = None;
    let mut url = None;
    let mut options = true;
    while let Some(arg) = args.next() {
        if options && arg == "--" { options = false; continue; }
        if options && matches!(arg.as_str(), "-q" | "--quiet" | "-nv" | "--no-verbose") { continue; }
        if options && matches!(arg.as_str(), "-O" | "--output-document") {
            output = Some(PathBuf::from(args.next().ok_or("wget: missing output filename")?));
        } else if options && arg.starts_with("--output-document=") {
            output = Some(PathBuf::from(&arg[18..]));
        } else if options && arg.starts_with("-O") && arg.len() > 2 {
            output = Some(PathBuf::from(&arg[2..]));
        } else if options && arg.starts_with('-') {
            return Err(format!("PICO-8 download: unsupported wget option {arg}"));
        } else {
            if url.is_some() { return Err("PICO-8 download: expected one URL".into()); }
            if crate::net::split_url(&arg).is_none() { return Err("PICO-8 download: invalid URL".into()); }
            url = Some(arg);
        }
    }
    let url = url.ok_or("PICO-8 download: missing URL")?;
    let output = output.unwrap_or_else(|| {
        let (_, _, path) = crate::net::split_url(&url).unwrap();
        let name = path.split('?').next().unwrap().rsplit('/').next().unwrap();
        PathBuf::from(if name.is_empty() || matches!(name, "." | "..") { "index.html" } else { name })
    });
    if output.as_os_str().is_empty() { return Err("PICO-8 download: empty filename".into()); }
    Ok(Download { url, output })
}

pub fn run(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    use std::io::Write;
    let request = parse(args)?;
    // Carts are small PNGs; bound responses and retry transient network failures once.
    let bytes = crate::net::get(&request.url, 32 * 1024 * 1024)
        .or_else(|_| crate::net::get(&request.url, 32 * 1024 * 1024))?;
    if request.output == std::path::Path::new("-") {
        std::io::stdout().lock().write_all(&bytes).map_err(|e| e.to_string())
    } else {
        slot_store::atomic_write(&request.output, &bytes).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pico8_arguments_preserve_paths_and_query_strings() {
        let got = parse(["-q", "-O", "/tmp/a ' cart.png", "https://www.lexaloffle.com/bbs/cposts/x.p8.png?g=1&v=2"].map(str::to_string)).unwrap();
        assert_eq!(got.output, PathBuf::from("/tmp/a ' cart.png"));
        assert!(got.url.ends_with("?g=1&v=2"));
        for option in ["--output-document=cart.png", "-Ocart.png"] {
            assert_eq!(parse([option, "https://example.com/game.png"].map(str::to_string)).unwrap().output, PathBuf::from("cart.png"));
        }
        assert!(parse(["--no-check-certificate", "https://example.com"].map(str::to_string)).is_err());
        assert!(parse(["-O"].map(str::to_string)).is_err());
    }
    #[test]
    fn downloader_retries_and_does_not_replace_carts_with_errors() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/cart", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for reply in ["HTTP/1.0 503 Busy\r\nContent-Length: 0\r\n\r\n", "HTTP/1.0 200 OK\r\nContent-Length: 4\r\n\r\ncart", "HTTP/1.0 404 No\r\nContent-Length: 0\r\n\r\n", "HTTP/1.0 404 No\r\nContent-Length: 0\r\n\r\n"] {
                let (mut socket, _) = listener.accept().unwrap();
                let mut byte = [0]; let mut head = Vec::new();
                while !head.ends_with(b"\r\n\r\n") { socket.read_exact(&mut byte).unwrap(); head.push(byte[0]); }
                socket.write_all(reply.as_bytes()).unwrap();
            }
        });
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("cart.png");
        let args = vec!["-q".into(), "-O".into(), output.to_str().unwrap().into(), url];
        run(args.clone()).unwrap();
        assert_eq!(std::fs::read(&output).unwrap(), b"cart");
        assert!(run(args).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), b"cart");
        server.join().unwrap();
    }
}
