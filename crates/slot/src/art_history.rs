//! Persistent scrape attempts. Existing artwork is never part of the retry queue.
use slot_store::Cart;
use std::{collections::HashMap, io::Write, path::Path, time::UNIX_EPOCH};

pub const FILE: &str = "Config/cart-art-history.tsv";
pub const FAILURES_FILE: &str = "Logs/cart-art-failures.json";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultKind {
    Found,
    Missing,
    Failed,
}
impl ResultKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Found => "ART SAVED",
            Self::Missing => "NO MATCH",
            Self::Failed => "FAILED",
        }
    }
    fn code(self) -> &'static str {
        match self {
            Self::Found => "found",
            Self::Missing => "missing",
            Self::Failed => "failed",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Queue {
    New,
    Failed,
    Missing,
    All,
}
impl Queue {
    pub const ALL: [Self; 4] = [Self::New, Self::Failed, Self::Missing, Self::All];
    pub fn label(self) -> &'static str {
        match self {
            Self::New => "NEW GAMES",
            Self::Failed => "RETRY FAILED",
            Self::Missing => "RETRY NO MATCH",
            Self::All => "ALL MISSING ART",
        }
    }
}
#[derive(Clone, Debug)]
pub struct Attempt {
    pub key: String,
    pub size: u64,
    pub modified: u128,
    pub screenscraper: bool,
    pub result: ResultKind,
    pub sha1: Option<String>,
}
#[derive(Clone, Default)]
pub struct History {
    pub entries: HashMap<String, Attempt>,
}
pub fn key(cart: &Cart) -> String {
    format!("{}/{}", cart.platform.dir_name(), cart.stem)
}
pub fn stamp(cart: &Cart) -> Option<(u64, u128)> {
    let m = std::fs::metadata(&cart.rom).ok()?;
    Some((
        m.len(),
        m.modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos(),
    ))
}
impl History {
    pub fn load(root: &Path) -> Self {
        let mut history = Self::default();
        for line in std::fs::read_to_string(root.join(FILE))
            .unwrap_or_default()
            .lines()
        {
            let f: Vec<_> = line.splitn(7, '\t').collect();
            let (sha1, key) = match f.as_slice() {
                ["1", _, _, _, _, key] => (None, *key),
                ["2", _, _, _, _, sha, key] => (
                    (sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit()))
                        .then(|| sha.to_string()),
                    *key,
                ),
                _ => continue,
            };
            if key.is_empty() {
                continue;
            }
            let (Ok(size), Ok(modified)) = (f[1].parse(), f[2].parse()) else {
                continue;
            };
            let result = match f[4] {
                "found" => ResultKind::Found,
                "missing" => ResultKind::Missing,
                "failed" => ResultKind::Failed,
                _ => continue,
            };
            let attempt = Attempt {
                key: key.into(),
                size,
                modified,
                screenscraper: f[3] == "1",
                result,
                sha1,
            };
            history.entries.insert(attempt.key.clone(), attempt);
        }
        history
    }
    pub fn status(&self, cart: &Cart) -> Option<ResultKind> {
        self.entries.get(&key(cart)).map(|a| a.result)
    }
    /// Only reuse a checksum while the ROM's size and modification time still match.
    pub fn sha1(&self, cart: &Cart) -> Option<&str> {
        let a = self.entries.get(&key(cart))?;
        (stamp(cart) == Some((a.size, a.modified)))
            .then_some(a.sha1.as_deref())
            .flatten()
    }
    pub fn export_failures(&self, root: &Path) -> std::io::Result<()> {
        let mut entries: Vec<_> = self
            .entries
            .values()
            .filter(|a| a.result != ResultKind::Found)
            .collect();
        entries.sort_by(|a, b| a.key.cmp(&b.key));
        let rows: Vec<_> = entries
            .into_iter()
            .filter_map(|a| {
                let (platform, name) = a.key.split_once('/')?;
                Some(format!(
                    "  {{\"name\": {}, \"sha1\": {}, \"platform\": {}}}",
                    json_string(name),
                    a.sha1.as_deref().map_or("null".into(), json_string),
                    json_string(platform),
                ))
            })
            .collect();
        let json = format!("[\n{}\n]\n", rows.join(",\n"));
        let path = root.join(FAILURES_FILE);
        std::fs::create_dir_all(path.parent().unwrap())?;
        slot_store::atomic_write(&path, json.as_bytes())
    }
    pub fn wants(&self, cart: &Cart, queue: Queue, ss: bool) -> bool {
        let prior = self.entries.get(&key(cart));
        let fresh = prior.filter(|a| stamp(cart) == Some((a.size, a.modified)));
        match queue {
            Queue::New => fresh.is_none_or(|a| ss && !a.screenscraper),
            Queue::Failed => fresh.is_some_and(|a| a.result == ResultKind::Failed),
            Queue::Missing => fresh.is_some_and(|a| a.result == ResultKind::Missing),
            Queue::All => true,
        }
    }
    // Append after every completed game, so cancel/reboot retains completed attempts.
    pub fn record(
        &mut self,
        root: &Path,
        cart: &Cart,
        result: ResultKind,
        ss: bool,
        sha1: Option<String>,
    ) -> std::io::Result<Attempt> {
        let (size, modified) = stamp(cart).unwrap_or_default();
        let attempt = Attempt {
            key: key(cart),
            size,
            modified,
            screenscraper: ss,
            result,
            sha1,
        };
        if attempt.key.contains(['\t', '\r', '\n']) {
            return Err(std::io::Error::other("Unsupported game name"));
        }
        let path = root.join(FILE);
        std::fs::create_dir_all(path.parent().unwrap())?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        // A leading newline also isolates a torn final record after power loss.
        writeln!(
            file,
            "\n2\t{size}\t{modified}\t{}\t{}\t{}\t{}",
            u8::from(ss),
            result.code(),
            attempt.sha1.as_deref().unwrap_or(""),
            attempt.key
        )?;
        file.sync_data()?;
        self.entries.insert(attempt.key.clone(), attempt.clone());
        Ok(attempt)
    }
}

fn json_string(value: &str) -> String {
    use std::fmt::Write;
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0}'..='\u{1f}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cart(root: &Path) -> Cart {
        Cart {
            platform: slot_store::Platform::Gg,
            stem: "Sonic".into(),
            rom: root.join("Sonic.gg"),
            label: None,
            famicom_label: None,
            title: String::new(),
            code: String::new(),
            shell: None,
        }
    }
    #[test]
    fn failure_export_survives_restart_and_successful_retry_removes_only_that_game() {
        let d = tempfile::tempdir().unwrap();
        let mut c = cart(d.path());
        std::fs::write(&c.rom, b"rom").unwrap();
        c.stem = "Sonic \"雪\"\\Test".into();
        let sha = "A9993E364706816ABA3E25717850C26C9CD0D89D";
        let mut h = History::default();
        h.record(d.path(), &c, ResultKind::Failed, false, Some(sha.into()))
            .unwrap();
        let mut sms = c.clone();
        sms.platform = slot_store::Platform::Sms;
        h.record(d.path(), &sms, ResultKind::Missing, false, None)
            .unwrap();
        let mut h = History::load(d.path());
        h.export_failures(d.path()).unwrap();
        let json = std::fs::read_to_string(d.path().join(FAILURES_FILE)).unwrap();
        assert_eq!(json,format!("[\n  {{\"name\": \"Sonic \\\"雪\\\"\\\\Test\", \"sha1\": \"{sha}\", \"platform\": \"GG\"}},\n  {{\"name\": \"Sonic \\\"雪\\\"\\\\Test\", \"sha1\": null, \"platform\": \"SMS\"}}\n]\n"));
        h.record(d.path(), &c, ResultKind::Found, false, None)
            .unwrap();
        h.export_failures(d.path()).unwrap();
        let json = std::fs::read_to_string(d.path().join(FAILURES_FILE)).unwrap();
        assert!(!json.contains("GG"));
        assert!(json.contains("SMS"));
        assert_eq!(History::load(d.path()).status(&c), Some(ResultKind::Found));
    }
    #[test]
    fn legacy_history_and_changed_rom_hashes_are_handled() {
        let d = tempfile::tempdir().unwrap();
        let c = cart(d.path());
        std::fs::write(&c.rom, b"rom").unwrap();
        let (size, modified) = stamp(&c).unwrap();
        std::fs::create_dir_all(d.path().join("Config")).unwrap();
        std::fs::write(
            d.path().join(FILE),
            format!("1\t{size}\t{modified}\t0\tfailed\tGG/Sonic\n"),
        )
        .unwrap();
        let mut h = History::load(d.path());
        assert_eq!(h.status(&c), Some(ResultKind::Failed));
        assert!(h.sha1(&c).is_none());
        h.export_failures(d.path()).unwrap();
        assert!(std::fs::read_to_string(d.path().join(FAILURES_FILE))
            .unwrap()
            .contains("\"sha1\": null"));
        let sha = "A9993E364706816ABA3E25717850C26C9CD0D89D";
        h.record(d.path(), &c, ResultKind::Failed, false, Some(sha.into()))
            .unwrap();
        let h = History::load(d.path());
        assert_eq!(h.sha1(&c), Some(sha));
        std::fs::write(&c.rom, b"changed").unwrap();
        assert!(h.sha1(&c).is_none());
        assert!(h.wants(&c, Queue::New, false));
        assert_eq!(
            json_string("line\n\t\u{0000}"),
            "\"line\\u000a\\u0009\\u0000\""
        );
    }
    #[test]
    fn attempts_survive_restarts_and_retries_are_explicit() {
        let d = tempfile::tempdir().unwrap();
        let c = cart(d.path());
        std::fs::write(&c.rom, b"rom").unwrap();
        let mut h = History::default();
        assert!(h.wants(&c, Queue::New, false));
        h.record(d.path(), &c, ResultKind::Missing, false, None)
            .unwrap();
        let h = History::load(d.path());
        assert!(!h.wants(&c, Queue::New, false));
        assert!(h.wants(&c, Queue::Missing, false));
        assert!(!h.wants(&c, Queue::Failed, false));
        assert!(
            h.wants(&c, Queue::New, true),
            "newly configured fallback must get a chance"
        );
        std::fs::write(&c.rom, b"different rom").unwrap();
        assert!(h.wants(&c, Queue::New, false));
    }
}
