use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use slot_store::{write_slot_state, GameSettingsStore, SlotState};

#[derive(Default)]
struct Queue {
    next: Option<(PathBuf, SlotState)>,
    game_next: Option<(PathBuf, GameSettingsStore)>,
    busy: bool,
    closed: bool,
}

type Shared = Arc<(Mutex<Queue>, Condvar)>;

pub struct StateWriter {
    shared: Shared,
    threaded: bool,
    failing: Arc<AtomicBool>,
}

impl StateWriter {
    pub fn spawn() -> Self {
        let shared: Shared = Arc::default();
        let failing: Arc<AtomicBool> = Arc::default();
        let worker = shared.clone();
        let flag = failing.clone();
        let threaded = thread::Builder::new()
            .name("slot-state".into())
            .spawn(move || work(&worker, &flag))
            .is_ok();
        StateWriter {
            shared,
            threaded,
            failing,
        }
    }

    pub fn save(&self, root: &Path, state: &SlotState) {
        if !self.threaded {
            return write(root, state, &self.failing);
        }
        let (lock, wake) = &*self.shared;
        let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
        q.next = Some((root.to_path_buf(), state.clone()));
        wake.notify_all();
    }

    /// Keep game overrides off the input thread. Only the newest pending
    /// settings snapshot is needed; a single worker serializes SD card writes.
    pub fn save_game_settings(&self, root: &Path, settings: &GameSettingsStore) {
        if !self.threaded {
            return write_games(root, settings);
        }
        let (lock, wake) = &*self.shared;
        let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
        q.game_next = Some((root.to_path_buf(), settings.clone()));
        wake.notify_all();
    }

    pub fn card_failing(&self) -> Arc<AtomicBool> {
        self.failing.clone()
    }

    pub fn flush(&self) {
        let (lock, wake) = &*self.shared;
        let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
        while q.next.is_some() || q.game_next.is_some() || q.busy {
            q = wake.wait(q).unwrap_or_else(|e| e.into_inner());
        }
    }
}

impl Drop for StateWriter {
    fn drop(&mut self) {
        self.flush();
        let (lock, wake) = &*self.shared;
        lock.lock().unwrap_or_else(|e| e.into_inner()).closed = true;
        wake.notify_all();
    }
}

enum Job {
    Slot(PathBuf, SlotState),
    Games(PathBuf, GameSettingsStore),
}

fn work(shared: &Shared, failing: &AtomicBool) {
    let (lock, wake) = &**shared;
    loop {
        let job = {
            let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if let Some((root,state)) = q.next.take() {
                    q.busy = true;
                    break Job::Slot(root,state);
                }
                if let Some((root,settings)) = q.game_next.take() {
                    q.busy = true;
                    break Job::Games(root,settings);
                }
                if q.closed {
                    return;
                }
                q = wake.wait(q).unwrap_or_else(|e| e.into_inner());
            }
        };
        match job {
            Job::Slot(root,state) => write(&root, &state, failing),
            Job::Games(root,settings) => write_games(&root, &settings),
        }
        lock.lock().unwrap_or_else(|e| e.into_inner()).busy = false;
        wake.notify_all();
    }
}

fn write(root: &Path, state: &SlotState, failing: &AtomicBool) {
    let written = write_slot_state(root, state);
    if let Err(e) = &written {
        eprintln!("slot: slot.state: {e}");
    }
    failing.store(written.is_err(), Ordering::Relaxed);
}

fn write_games(root: &Path, settings: &GameSettingsStore) {
    if let Err(e) = settings.write(root) {
        eprintln!("slot: game settings: {e}");
    }
}
