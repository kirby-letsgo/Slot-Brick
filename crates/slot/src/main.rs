#[cfg(not(feature = "host"))]
mod device_app;
#[cfg(feature = "host")]
mod host_app;

fn main() {
    let mut args = std::env::args().skip(1);
    let action = args.next();
    if action.as_deref() == Some("--pico8-wget") {
        if let Err(error) = slot::pico8_download::run(args) {
            eprintln!("slot: PICO-8 download: {error}");
            std::process::exit(1);
        }
        return;
    }
    if matches!(action.as_deref(), Some("--run-pico8" | "--run-native")) {
        let root = std::env::var_os("SLOT_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "/mnt/SDCARD/Slot".into());
        if action.as_deref() == Some("--run-native") && slot::ports::pending(&root) {
            if let Err(error) = slot::ports::run_pending(&root) { slot::ports::record_error(&root, &error); }
        } else if let Err(error) = slot::pico8::run_pending(&root) {
            slot::pico8::record_error(&root, &error);
        }
        return;
    }
    if action.as_deref() == Some("--init-root") {
        let Some(dir) = args.next() else {
            eprintln!("slot: --init-root needs a directory");
            std::process::exit(2);
        };
        slot::root::ensure(std::path::Path::new(&dir));
        return;
    }
    // 1.6.0: on a crash, log the signal, thread and code location to stderr (slot.log).
    slot::crash::install();

    #[cfg(feature = "host")]
    host_app::run();

    #[cfg(not(feature = "host"))]
    device_app::run();
}
