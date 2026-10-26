//! A native PortMaster process runs only after the frontend releases its devices.
use std::{fs, path::{Path, PathBuf}, process::{Command, Stdio}};
use slot_store::{Cart, Platform};

const REQUEST: &str = "Config/port-launch";
pub const ERROR: &str = "Config/port-error";

#[derive(Clone, Debug)]
pub struct Launch { pub script: Option<PathBuf> }

pub fn check_runtime(root: &Path) -> Result<(), String> {
    let pm = slot_store::ports::manager(root);
    for file in ["control.txt", "pugwash", "gamecontrollerdb.txt"] {
        if !pm.join(file).is_file() {
            return Err("Install PortMaster using setup-portmaster.py on your computer. See Slot/System/PORTS.md".into());
        }
    }
    if !root.join("System/PortRuntime/bin/bash").is_file()
        || !root.join("System/PortRuntime/bin/python3").is_file() {
        return Err("PortMaster support files are missing. Run setup-portmaster.py; see Slot/System/PORTS.md".into());
    }
    Ok(())
}

fn validate(root: &Path, launch: &Launch) -> Result<(), String> {
    if let Some(script) = &launch.script {
        let dir = fs::canonicalize(slot_store::ports::directory(root)).map_err(|_| "Ports folder is missing")?;
        let path = fs::canonicalize(script).map_err(|_| "Port launcher is missing")?;
        if path.parent() != Some(dir.as_path()) || !path.is_file() || !Platform::Ports.accepts(&path) {
            return Err("Choose a top-level .sh launcher in Data/ports".into());
        }
    }
    Ok(())
}

pub fn prepare(root: &Path, cart: &Cart) -> Result<Launch, String> {
    if cart.platform != Platform::Ports { return Err("Not a native port".into()); }
    check_runtime(root)?;
    let manager = cart.code == "SLOT_PORTMASTER" && cart.rom == root.join("System/portmaster-launch.sh");
    let launch = Launch { script: (!manager).then(|| cart.rom.clone()) };
    validate(root, &launch)?;
    if cart.rom.file_name().is_some_and(|s| s.eq_ignore_ascii_case("Balatro.sh")) {
        let game = slot_store::ports::directory(root).join("balatro");
        if !["Balatro.exe", "balatro.exe", "Balatro.love", "balatro.love", "Balatro_pm", "Balatro"]
            .iter().any(|f| game.join(f).is_file()) {
            return Err("Copy your Balatro.exe or Balatro.love into Data/ports/balatro on the card".into());
        }
        if !slot_store::ports::manager(root).join("runtimes/love_11.5/love.aarch64").is_file() {
            return Err("Update PortMaster to install the LÖVE 11.5 runtime, then try Balatro again".into());
        }
    }
    Ok(launch)
}

pub fn queue(root: &Path, launch: &Launch) -> Result<(), String> {
    validate(root, launch)?;
    let path = launch.script.as_ref().map(|p| p.to_str().ok_or("Port path is not UTF-8")).transpose()?;
    slot_store::atomic_write(&root.join(REQUEST), path.unwrap_or("").as_bytes()).map_err(|e| e.to_string())
}

pub fn pending(root: &Path) -> bool { root.join(REQUEST).is_file() }

pub fn run_pending(root: &Path) -> Result<(), String> {
    use std::os::{fd::AsRawFd, unix::process::CommandExt};
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let request = fs::read_to_string(root.join(REQUEST)).map_err(|e| e.to_string())?;
    fs::remove_file(root.join(REQUEST)).map_err(|e| e.to_string())?;
    if request.len() > 4096 { return Err("Port path is too long".into()); }
    let launch = Launch { script: (!request.is_empty()).then(|| PathBuf::from(request)) };
    validate(&root, &launch)?;
    check_runtime(&root)?;
    fs::create_dir_all(root.join("Logs")).map_err(|e| e.to_string())?;
    let log = fs::File::create(root.join("Logs/ports.log")).map_err(|e| e.to_string())?;
    let mut audio = crate::pico8_audio::NativeAudio::new(&root);
    let volume = crate::pico8_audio::volume_channel(audio.output())?;
    let mut cmd = Command::new(root.join("System/PortRuntime/bin/bash"));
    cmd.arg(root.join("System/portmaster-launch.sh"));
    if let Some(script) = &launch.script { cmd.arg(script); }
    cmd.env("SLOT_ROOT", &root)
        .env("LD_LIBRARY_PATH", format!("{}:/usr/trimui/lib:/usr/lib:/lib", root.join("System/PortRuntime/lib").display()))
        .env("SLOT_PICO8_VOLUME_FD", volume.as_raw_fd().to_string())
        .env("SLOT_PICO8_VOLUME", audio.output().to_string())
        .env("LD_PRELOAD", root.join("System/pico8-sdl-nosensor.so"))
        .stdin(Stdio::null()).stdout(log.try_clone().map_err(|e| e.to_string())?).stderr(log)
        .current_dir(slot_store::ports::directory(&root)).process_group(0);
    let mut child = cmd.spawn().map_err(|e| format!("Port launch failed: {e}"))?;
    let result = crate::pico8::wait_native(&root, &mut child, &mut audio, &volume, true);
    // Helpers must not hold input/audio open when Slot returns. Signal only our group.
    unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL); }
    let _ = child.wait();
    audio.persist(&root);
    let status = result.map_err(|e| e.to_string())?;
    if !status.success() { return Err(format!("Port exited with {status}. See Slot/Logs/ports.log")); }
    Ok(())
}

pub fn record_error(root: &Path, message: &str) {
    eprintln!("slot: port: {message}");
    let _ = slot_store::atomic_write(&root.join(ERROR), message.as_bytes());
}
