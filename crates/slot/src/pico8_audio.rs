//! Native audio preferences and the anonymous shared byte read by the SDL shim.
use slot_input::{Action, MUTE_CHORD_MS};
use slot_store::SlotState;
use std::{fs::File, path::Path};

#[cfg_attr(any(not(feature = "brick"), feature = "sim"), allow(dead_code))]
pub(crate) struct NativeAudio {
    state: SlotState,
    headphones: bool,
    before: Vec<(u8, bool, u64)>,
    dirty: bool,
}

#[cfg_attr(any(not(feature = "brick"), feature = "sim"), allow(dead_code))]
impl NativeAudio {
    pub(crate) fn new(root: &Path) -> Self {
        #[cfg(feature = "brick")]
        let headphones = {
            use slot_power::Platform as _;
            slot_power::BrickPlatform::new(root.to_owned()).headphones()
        };
        #[cfg(not(feature = "brick"))]
        let headphones = false;
        Self { state: slot_store::read_slot_state(root), headphones, before: Vec::new(), dirty: false }
    }
    fn current(&self) -> (u8, bool) {
        if self.headphones { (self.state.volume_hp, self.state.muted_hp) }
        else { (self.state.volume, self.state.muted) }
    }
    fn set(&mut self, level: u8, muted: bool) {
        if self.headphones { self.state.volume_hp = level; self.state.muted_hp = muted; }
        else { self.state.volume = level; self.state.muted = muted; }
        self.dirty = true;
    }
    pub(crate) fn output(&self) -> u8 {
        let (level, muted) = self.current();
        if muted { 0 } else { level.min(100) }
    }
    pub(crate) fn headphones(&mut self, plugged: bool) {
        if plugged != self.headphones { self.headphones = plugged; self.before.clear(); }
    }
    pub(crate) fn apply(&mut self, action: Action, now: u64) -> bool {
        let (level, muted) = self.current();
        match action {
            Action::VolumeUp | Action::VolumeDown => {
                if self.before.len() == 2 { self.before.remove(0); }
                self.before.push((level, muted, now));
                self.set(if action == Action::VolumeUp { level.saturating_add(5).min(100) }
                    else { level.saturating_sub(5) }, false);
            }
            Action::MuteToggle => {
                // Undo the first volume key in the two-key mute gesture, as Slot does.
                let (level, muted) = self.before.iter()
                    .find(|(_, _, at)| now.saturating_sub(*at) <= MUTE_CHORD_MS)
                    .map(|(v, m, _)| (*v, *m)).unwrap_or((level, muted));
                self.before.clear();
                self.set(level, !muted);
            }
            _ => return false,
        }
        true
    }
    pub(crate) fn persist(&mut self, root: &Path) {
        if !self.dirty { return; }
        // Merge only audio fields so another setting is never rolled back.
        let mut state = slot_store::read_slot_state(root);
        state.volume = self.state.volume;
        state.muted = self.state.muted;
        state.volume_hp = self.state.volume_hp;
        state.muted_hp = self.state.muted_hp;
        match slot_store::write_slot_state(root, &state) {
            Ok(()) => self.dirty = false,
            Err(e) => eprintln!("slot: PICO-8 audio preferences: {e}"),
        }
    }
}

pub(crate) fn volume_channel(volume: u8) -> Result<File, String> {
    use std::io::Write;
    use std::os::fd::FromRawFd;
    // Inherited by PICO-8, unlinked immediately: no card writes in the audio callback,
    // no named IPC file left after a crash. mkstemp does not set close-on-exec.
    let mut name = b"/tmp/slot-pico8-volume-XXXXXX\0".to_vec();
    let fd = unsafe { libc::mkstemp(name.as_mut_ptr().cast()) };
    if fd < 0 { return Err(std::io::Error::last_os_error().to_string()); }
    let mut file = unsafe { File::from_raw_fd(fd) };
    unsafe { libc::unlink(name.as_ptr().cast()); }
    file.write_all(&[volume]).map_err(|e| e.to_string())?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_mute_live_buttons_and_headphone_preferences_persist() {
        let d = tempfile::tempdir().unwrap(); crate::root::ensure(d.path());
        let mut state = SlotState::default();
        state.volume = 40; state.muted = true; state.volume_hp = 70;
        slot_store::write_slot_state(d.path(), &state).unwrap();
        let mut audio = NativeAudio::new(d.path());
        audio.headphones(false);
        assert_eq!(audio.output(), 0);
        audio.apply(Action::VolumeUp, 0);
        assert_eq!(audio.output(), 45);
        audio.apply(Action::MuteToggle, 50);
        assert_eq!(audio.output(), 40, "mute chord restores the level and toggles original mute");
        audio.apply(Action::MuteToggle, 1000);
        assert_eq!(audio.output(), 0);
        audio.headphones(true);
        assert_eq!(audio.output(), 70);
        audio.apply(Action::VolumeDown, 2000);
        audio.persist(d.path());
        let saved = slot_store::read_slot_state(d.path());
        assert_eq!((saved.volume, saved.muted, saved.volume_hp), (40, true, 65));
    }
    #[test]
    fn channel_is_inheritable_and_updates_are_visible_without_card_writes() {
        use std::os::{fd::AsRawFd, unix::fs::FileExt};
        let file = volume_channel(0).unwrap();
        assert_eq!(unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC, 0);
        let ptr = unsafe { libc::mmap(std::ptr::null_mut(), 1, libc::PROT_READ, libc::MAP_SHARED, file.as_raw_fd(), 0) };
        assert_ne!(ptr, libc::MAP_FAILED);
        assert_eq!(unsafe { ptr.cast::<u8>().read_volatile() }, 0);
        file.write_all_at(&[50], 0).unwrap();
        assert_eq!(unsafe { ptr.cast::<u8>().read_volatile() }, 50);
        unsafe { libc::munmap(ptr, 1); }
        #[cfg(target_os = "linux")]
        {
            let child = std::process::Command::new("od")
                .args(["-An", "-tu1", &format!("/proc/self/fd/{}", file.as_raw_fd())])
                .output().unwrap();
            assert!(child.status.success());
            assert_eq!(String::from_utf8(child.stdout).unwrap().trim(), "50");
        }
    }
}
