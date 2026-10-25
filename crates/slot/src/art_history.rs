//! Persistent scrape attempts. Existing artwork is never part of the retry queue.
use slot_store::Cart;
use std::{collections::HashMap, io::Write, path::Path, time::UNIX_EPOCH};

pub const FILE: &str = "Config/cart-art-history.tsv";
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
            let f: Vec<_> = line.splitn(6, '\t').collect();
            if f.len() != 6 || f[0] != "1" {
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
                key: f[5].into(),
                size,
                modified,
                screenscraper: f[3] == "1",
                result,
            };
            history.entries.insert(attempt.key.clone(), attempt);
        }
        history
    }
    pub fn status(&self, cart: &Cart) -> Option<ResultKind> {
        self.entries.get(&key(cart)).map(|a| a.result)
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
    ) -> std::io::Result<Attempt> {
        let (size, modified) = stamp(cart).unwrap_or_default();
        let attempt = Attempt {
            key: key(cart),
            size,
            modified,
            screenscraper: ss,
            result,
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
            "\n1\t{size}\t{modified}\t{}\t{}\t{}",
            u8::from(ss),
            result.code(),
            attempt.key
        )?;
        file.sync_data()?;
        self.entries.insert(attempt.key.clone(), attempt.clone());
        Ok(attempt)
    }
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
    fn attempts_survive_restarts_and_retries_are_explicit() {
        let d = tempfile::tempdir().unwrap();
        let c = cart(d.path());
        std::fs::write(&c.rom, b"rom").unwrap();
        let mut h = History::default();
        assert!(h.wants(&c, Queue::New, false));
        h.record(d.path(), &c, ResultKind::Missing, false).unwrap();
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
