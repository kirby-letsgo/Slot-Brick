mod alsa;
#[cfg(feature = "host")]
mod host;
mod ring;
mod sfx;
mod silence;
mod sink;
mod stub;
pub mod volume;

pub use alsa::AlsaSink;
#[cfg(feature = "host")]
pub use host::HostAudio;
pub use ring::{ring_capacity, Ring};
pub use sfx::Sfx;
pub use silence::Silence;
pub use sink::{AudioError, AudioSink};
pub use stub::StubSink;

pub const GBA_HZ: u32 = 32_768;

/// The output, opened on a thread of its own (open_early) while the screen comes up, for
/// Session::boot to take: opening it costs about a fifth of a second at startup.
static EARLY: std::sync::Mutex<Option<std::thread::JoinHandle<Box<dyn AudioSink>>>> =
    std::sync::Mutex::new(None);

pub fn open_early() {
    match std::thread::Builder::new()
        .name("slot-audio-open".into())
        .spawn(opened)
    {
        Ok(handle) => *EARLY.lock().unwrap_or_else(|e| e.into_inner()) = Some(handle),
        Err(e) => eprintln!("slot: audio: {e}"),
    }
}

/// The output open_early opened, or opened now if it was not asked for.
pub fn take_opened() -> Box<dyn AudioSink> {
    let early = EARLY.lock().unwrap_or_else(|e| e.into_inner()).take();
    early.and_then(|h| h.join().ok()).unwrap_or_else(opened)
}

fn opened() -> Box<dyn AudioSink> {
    let mut sink = open_sink();
    if let Err(e) = sink.open(GBA_HZ) {
        eprintln!("slot: audio: {e}");
    }
    sink
}

#[cfg(feature = "host")]
pub fn open_sink() -> Box<dyn AudioSink> {
    if std::env::var_os("SLOT_SILENT").is_some() {
        return Box::new(StubSink::draining());
    }
    Box::new(HostAudio::new())
}

#[cfg(all(not(feature = "host"), not(feature = "sim")))]
pub fn open_sink() -> Box<dyn AudioSink> {
    Box::new(AlsaSink::new())
}

/// The simulator has no speaker: sound is played into nothing at the speed a speaker would.
#[cfg(feature = "sim")]
pub fn open_sink() -> Box<dyn AudioSink> {
    Box::new(StubSink::draining())
}
