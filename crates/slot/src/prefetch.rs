use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

use slot_input::Millis;

pub const PREFETCH_REST_MS: Millis = 500;

const CHUNK: usize = 256 << 10;
/// Limit speculative SD reads; full reads of large ROMs can contend with emulation and FTP.
pub const PREFETCH_MAX_BYTES: u64 = 4 << 20;

#[derive(Default)]
pub struct Prefetch {
    resting: Option<(PathBuf, Millis)>,
    read: Option<PathBuf>,
    turn: Arc<AtomicU64>,
    worker: Option<JoinHandle<u64>>,
}

impl Prefetch {
    pub fn browse(&mut self, rom: Option<&Path>, now: Millis) {
        let Some(rom) = rom else {
            self.resting = None;
            self.read = None;
            self.turn.fetch_add(1, Ordering::Relaxed);
            return;
        };
        let since = match &self.resting {
            Some((at, since)) if at == rom => *since,
            _ => {
                self.resting = Some((rom.to_path_buf(), now));
                self.read = None;
                self.turn.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };
        if now.saturating_sub(since) < PREFETCH_REST_MS || self.read.as_deref() == Some(rom) {
            return;
        }
        // Disc images and very large ROMs provide little benefit from speculative
        // sequential reads, and can monopolise the SD card during browsing.
        let disc = rom.extension().is_some_and(|ext| {
            ["chd", "cue", "iso", "pbp", "m3u"].iter().any(|name| ext.eq_ignore_ascii_case(name))
        });
        if disc {
            return;
        }
        self.read = Some(rom.to_path_buf());
        let turn = self.turn.clone();
        let mine = turn.fetch_add(1, Ordering::Relaxed) + 1;
        let path = rom.to_path_buf();
        self.worker = std::thread::Builder::new()
            .name("slot-prefetch".into())
            .spawn(move || read_through(&path, &turn, mine))
            .ok();
    }

    pub fn reading(&self) -> Option<&Path> {
        self.read.as_deref()
    }

    pub fn wait(&mut self) -> Option<u64> {
        self.worker.take().and_then(|w| w.join().ok())
    }
}

impl Drop for Prefetch {
    fn drop(&mut self) {
        // Cancelling must never join a worker that may be waiting on the SD card.
        self.turn.fetch_add(1, Ordering::Relaxed);
    }
}

fn read_through(path: &Path, turn: &AtomicU64, mine: u64) -> u64 {
    let start = Instant::now();
    if turn.load(Ordering::Relaxed) != mine {
        return 0;
    }
    // Even a metadata lookup may wait on a busy card. Keep it off the input thread,
    // and inspect a file only once per selection rather than on every idle frame.
    let Ok(metadata) = path.metadata() else { return 0; };
    if !metadata.is_file() || metadata.len() > (64 << 20)
        || turn.load(Ordering::Relaxed) != mine {
        return 0;
    }
    let Ok(mut file) = std::fs::File::open(path) else {
        return 0;
    };
    let total = read_chunks(&mut file, turn, mine);
    eprintln!(
        "slot: prefetch: {} bytes of {} in {:.1} ms",
        total,
        path.display(),
        start.elapsed().as_secs_f64() * 1e3
    );
    total
}

fn read_chunks(file: &mut impl Read, turn: &AtomicU64, mine: u64) -> u64 {
    let mut buf = vec![0u8; CHUNK];
    let mut total = 0u64;
    while turn.load(Ordering::Relaxed) == mine && total < PREFETCH_MAX_BYTES {
        let wanted = (PREFETCH_MAX_BYTES - total).min(CHUNK as u64) as usize;
        match file.read(&mut buf[..wanted]) {
            Ok(0) | Err(_) => break,
            Ok(n) => total += n as u64,
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_obsolete_read_stops_after_its_current_chunk() {
        struct ChangeSelection<'a>(&'a AtomicU64);
        impl Read for ChangeSelection<'_> {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                self.0.fetch_add(1, Ordering::Relaxed);
                Ok(buf.len())
            }
        }
        let turn = AtomicU64::new(1);
        assert_eq!(read_chunks(&mut ChangeSelection(&turn), &turn, 1), CHUNK as u64);
        assert_eq!(read_chunks(&mut ChangeSelection(&turn), &turn, 1), 0);
    }

    #[test]
    fn dropping_prefetch_cancels_its_worker_without_waiting() {
        let prefetch = Prefetch::default();
        let turn = prefetch.turn.clone();
        drop(prefetch);
        assert_eq!(turn.load(Ordering::Relaxed), 1);
    }
}
