//! The official PICO-8 runs in its own process after Slot releases audio, video and input.
use slot_store::{Cart, Platform};
use crate::pico8_audio::{NativeAudio, volume_channel};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const EXIT_LAUNCH: i32 = 67;
const REQUEST: &str = "Config/pico8-launch";
const ERROR: &str = "Config/pico8-error";
const SDL_COMPAT: &str = "System/pico8-sdl-nosensor.so";

#[derive(Clone, Debug)]
pub struct Launch {
    pub rom: Option<PathBuf>,
}

pub fn prepare(root: &Path, cart: &Cart) -> Result<Launch, String> {
    if cart.platform != Platform::Pico8 {
        return Err("Not a PICO-8 cartridge".into());
    }
    check_runtime(root)?;
    let splore = cart.code == "SLOT_SPLORE" && cart.rom == root.join("System/pico8-splore.png");
    let launch = Launch {
        rom: if splore { None } else { Some(cart.rom.clone()) },
    };
    check_cart(root, &launch)?;
    Ok(launch)
}

pub(crate) fn check_runtime(root: &Path) -> Result<PathBuf, String> {
    let binary = root.join("BIOS/PICO8/pico8_64");
    let mut header = [0; 20];
    fs::File::open(&binary)
        .and_then(|mut f| f.read_exact(&mut header))
        .map_err(|_| {
            "Copy pico8_64 and pico8.dat from your Raspberry Pi download to Slot/BIOS/PICO8"
                .to_string()
        })?;
    if &header[..4] != b"\x7fELF"
        || header[4] != 2
        || header[5] != 1
        || u16::from_le_bytes([header[18], header[19]]) != 183
    {
        return Err("Use pico8_64 from the Raspberry Pi download (ARM64)".into());
    }
    if !fs::metadata(root.join("BIOS/PICO8/pico8.dat")).is_ok_and(|m| m.is_file() && m.len() > 0) {
        return Err("Copy pico8.dat beside pico8_64 in Slot/BIOS/PICO8".into());
    }
    Ok(binary)
}

fn check_cart(root: &Path, launch: &Launch) -> Result<(), String> {
    if let Some(rom) = &launch.rom {
        let games = fs::canonicalize(root.join("Games/PICO8"))
            .map_err(|_| "PICO-8 games folder is missing")?;
        let path = fs::canonicalize(rom).map_err(|_| "This PICO-8 cartridge is missing")?;
        if !path.starts_with(games) || !path.is_file() || !Platform::Pico8.accepts(&path) {
            return Err("Choose a .p8 or .png cart inside Slot/Games/PICO8".into());
        }
    }
    Ok(())
}

/// Write only a path, never shell code. An empty request opens Splore.
pub fn queue(root: &Path, launch: &Launch) -> Result<(), String> {
    let path = launch
        .rom
        .as_ref()
        .map(|p| p.to_str().ok_or("Cartridge path is not UTF-8"))
        .transpose()?;
    slot_store::atomic_write(&root.join(REQUEST), path.unwrap_or("").as_bytes())
        .map_err(|e| e.to_string())
}

fn take_request(root: &Path) -> Result<Launch, String> {
    let path =
        fs::read_to_string(root.join(REQUEST)).map_err(|_| "No PICO-8 launch was requested")?;
    fs::remove_file(root.join(REQUEST)).map_err(|e| e.to_string())?;
    if path.len() > 4096 {
        return Err("PICO-8 cartridge path is too long".into());
    }
    let launch = Launch {
        rom: (!path.is_empty()).then(|| PathBuf::from(path)),
    };
    check_cart(root, &launch)?;
    Ok(launch)
}

fn command(root: &Path, binary: &Path, launch: &Launch) -> Command {
    let mut command = Command::new(binary);
    // Only the native child needs this shim; Slot keeps the firmware's SDL unchanged.
    let mut preload = root.join(SDL_COMPAT).into_os_string();
    if let Some(inherited) = std::env::var_os("LD_PRELOAD").filter(|s| !s.is_empty()) {
        preload.push(":");
        preload.push(inherited);
    }
    command.env("LD_PRELOAD", preload);
    let mut path = root.join("System/pico8-bin").into_os_string();
    path.push(":");
    path.push(std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into()));
    command.env("PATH", path).env("SLOT_ROOT", root);
    let mut home = root.join("Config/PICO8").into_os_string();
    home.push("/");
    command
        .current_dir(root.join("BIOS/PICO8"))
        .arg("-home")
        .arg(home)
        .arg("-root_path")
        .arg(root.join("Games/PICO8"))
        .arg("-desktop")
        .arg(root.join("Screenshots/PICO8"))
        .args(["-joystick", "0", "-volume", "256"]);
    if let Some(rom) = &launch.rom {
        command.arg("-run").arg(rom);
    } else {
        command.arg("-splore");
    }
    command
}

fn configure(root: &Path) -> Result<(), String> {
    for path in ["Config/PICO8", "Saves/PICO8", "Screenshots/PICO8", "Logs"] {
        fs::create_dir_all(root.join(path)).map_err(|e| e.to_string())?;
    }
    let config = root.join("Config/PICO8/config.txt");
    let mut contents = fs::read_to_string(&config).unwrap_or_else(|_| {
        "windowed 0\nfullscreen_method 2\nblit_method 0\njoystick_index 0\n".into()
    });
    // Slot applies volume to SDL output, so native gain stays full even when launched muted.
    // Force the bundled downloader; stock firmware may have no usable HTTPS wget.
    contents = contents
        .lines()
        .filter(|line| {
            !matches!(
                line.split_whitespace().next(),
                Some("volume" | "cdata_path" | "use_wget")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    contents.push_str(&format!(
        "\nvolume 256\nuse_wget 1\ncdata_path {}/\n",
        root.join("Saves/PICO8").display()
    ));
    slot_store::atomic_write(&config, contents.as_bytes()).map_err(|e| e.to_string())?;
    let controls = root.join("Config/PICO8/sdl_controllers.txt");
    if !controls.exists() {
        let source = root.join("System/pico8-controllers.txt");
        fs::copy(source, controls).map_err(|e| format!("PICO-8 controller setup: {e}"))?;
    }
    Ok(())
}

/// Invoked by launch.sh, in a fresh process without a compositor or audio sink.
pub fn run_pending(root: &Path) -> Result<(), String> {
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let launch = take_request(&root)?;
    let binary = check_runtime(&root)?;
    if !root.join(SDL_COMPAT).is_file() {
        return Err("PICO-8 compatibility file missing; reinstall the Slot update".into());
    }
    if !root.join("System/pico8-bin/wget").is_file() {
        return Err("PICO-8 download helper missing; reinstall the Slot update".into());
    }
    configure(&root)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut mode = fs::metadata(&binary)
            .map_err(|e| e.to_string())?
            .permissions();
        mode.set_mode(mode.mode() | 0o100);
        fs::set_permissions(&binary, mode).map_err(|e| e.to_string())?;
        let helper = root.join("System/pico8-bin/wget");
        let mut mode = fs::metadata(&helper).map_err(|e| e.to_string())?.permissions();
        mode.set_mode(mode.mode() | 0o100);
        fs::set_permissions(helper, mode).map_err(|e| e.to_string())?;
    }
    let log = fs::File::create(root.join("Logs/pico8.log")).map_err(|e| e.to_string())?;
    let err = log.try_clone().map_err(|e| e.to_string())?;
    let mut audio = NativeAudio::new(&root);
    let shared_volume = volume_channel(audio.output())?;
    use std::os::fd::AsRawFd;
    let mut child = command(&root, &binary, &launch)
        .env("SLOT_PICO8_VOLUME_FD", shared_volume.as_raw_fd().to_string())
        .env("SLOT_PICO8_VOLUME", audio.output().to_string())
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(err)
        .spawn()
        .map_err(|e| format!("Could not launch PICO-8: {e}. See Logs/pico8.log"))?;
    let status = wait_native(&root, &mut child, &mut audio, &shared_volume, NativeProcess::Pico8).map_err(|e| {
        let _ = child.kill();
        let _ = child.wait();
        e.to_string()
    })?;
    audio.persist(&root);
    if !status.success() {
        return Err(format!("PICO-8 exited with {status}. See Logs/pico8.log"));
    }
    Ok(())
}

pub fn record_error(root: &Path, message: &str) {
    eprintln!("slot: PICO-8: {message}");
    let _ = slot_store::atomic_write(&root.join(ERROR), message.as_bytes());
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum NativeProcess { Pico8, Port, Dreamcast }

#[cfg(any(not(feature = "brick"), feature = "sim"))]
pub(crate) fn wait_native(
    _root: &Path,
    child: &mut std::process::Child,
    audio: &mut NativeAudio,
    shared_volume: &fs::File,
    process: NativeProcess,
) -> std::io::Result<std::process::ExitStatus> {
    let _ = (audio, shared_volume, process);
    child.wait()
}

#[cfg(all(feature = "brick", not(feature = "sim")))]
pub(crate) fn wait_native(
    root: &Path,
    child: &mut std::process::Child,
    audio: &mut NativeAudio,
    shared_volume: &fs::File,
    process: NativeProcess,
) -> std::io::Result<std::process::ExitStatus> {
    use slot_input::{Action, Gestures, InputSource};
    use slot_power::Platform as _;
    use std::time::{Duration, Instant};
    // On Brick, DeviceInput deliberately reads only KEY_POWER; HOME and volume
    // belong to the SDL joystick. Open that subsystem without a display/audio
    // sink so the native game keeps owning its screen and sound.
    let mut input: Box<dyn InputSource> = match crate::input::BrickInput::open(root) {
        Ok(input) => Box::new(input),
        Err(error) => {
            eprintln!("slot: native controls unavailable ({error}); power button only");
            Box::new(crate::input::DeviceInput::open(root))
        }
    };
    let mut gestures = Gestures::brick();
    let mut power = slot_power::BrickPlatform::new(root.to_owned());
    let mut cpu = crate::brick_power::CpuPower::open(root);
    // Standalone Flycast has no frame/audio telemetry for the adaptive policy.
    // A ceiling alone lets schedutil downclock between its bursts of work.
    let running_mode = if process == NativeProcess::Dreamcast {
        crate::brick_power::Mode::NativeGame
    } else {
        crate::brick_power::Mode::Boost
    };
    cpu.mode(running_mode, 0);
    let brightness = slot_store::read_slot_state(root).brightness;
    let mut paused = false;
    let started = Instant::now();
    let mut persisted_at = 0;
    let mut output = audio.output();
    let group = process != NativeProcess::Pico8;
    let target = if group { -(child.id() as i32) } else { child.id() as i32 };
    let result = (|| loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        let now = started.elapsed().as_millis() as u64;
        use std::os::unix::fs::FileExt;
        audio.headphones(power.headphones());
        let mut actions = Vec::new();
        for event in input.poll(now) {
            actions.extend(gestures.feed(event, now));
        }
        actions.extend(gestures.tick(now));
        for action in actions {
            if audio.apply(action, now) { continue; }
            match action {
                Action::PowerTap => {
                    let signal = if paused { libc::SIGCONT } else { libc::SIGSTOP };
                    if unsafe { libc::kill(target, signal) } == 0 {
                        paused = !paused;
                        power.set_backlight(if paused { 0 } else { brightness });
                        cpu.mode(
                            if paused {
                                crate::brick_power::Mode::Standby
                            } else {
                                running_mode
                            },
                            0,
                        );
                    }
                }
                Action::PowerOff | Action::Eject if group || action == Action::PowerOff => {
                    unsafe {
                        libc::kill(target, libc::SIGCONT);
                        libc::kill(target, libc::SIGTERM);
                    }
                    let deadline = Instant::now() + Duration::from_secs(3);
                    while child.try_wait()?.is_none() && Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    if child.try_wait()?.is_none() {
                        unsafe { libc::kill(target, libc::SIGKILL); }
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                    audio.persist(root);
                    cpu.restore();
                    if action == Action::PowerOff { power.poweroff(); }
                    use std::os::unix::process::ExitStatusExt;
                    return Ok(std::process::ExitStatus::from_raw(0));
                }
                _ => {}
            }
        }
        if audio.output() != output {
            output = audio.output();
            shared_volume.write_all_at(&[output], 0)?;
        }
        if now.saturating_sub(persisted_at) >= 300 {
            audio.persist(root);
            persisted_at = now;
        }
        std::thread::sleep(Duration::from_millis(20));
    })();
    if paused {
        power.set_backlight(brightness);
    }
    cpu.restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        crate::root::ensure(d.path());
        d
    }
    fn runtime(root: &Path) {
        let mut elf = [0; 64];
        elf[..6].copy_from_slice(b"\x7fELF\x02\x01");
        elf[18] = 183;
        fs::write(root.join("BIOS/PICO8/pico8_64"), elf).unwrap();
        fs::write(
            root.join("BIOS/PICO8/pico8.dat"),
            b"test fixture, not PICO-8",
        )
        .unwrap();
    }
    #[test]
    fn setup_refuses_missing_or_wrong_runtime_and_accepts_arm64_pair() {
        let d = root();
        assert!(check_runtime(d.path()).is_err());
        runtime(d.path());
        assert!(check_runtime(d.path()).is_ok());
        fs::write(d.path().join("BIOS/PICO8/pico8_64"), b"#!/bin/sh\n").unwrap();
        assert!(check_runtime(d.path()).is_err());
        runtime(d.path());
        fs::remove_file(d.path().join("BIOS/PICO8/pico8.dat")).unwrap();
        assert!(check_runtime(d.path()).is_err());
    }
    #[test]
    fn path_request_is_consumed_once_and_arguments_never_use_a_shell() {
        let d = root();
        let rom = d.path().join("Games/PICO8/a ' ; $(echo nope).p8");
        fs::write(&rom, b"pico-8 cartridge").unwrap();
        let launch = Launch {
            rom: Some(rom.clone()),
        };
        queue(d.path(), &launch).unwrap();
        let got = take_request(d.path()).unwrap();
        assert_eq!(got.rom, launch.rom);
        assert!(take_request(d.path()).is_err());
        let cmd = command(d.path(), Path::new("/official/pico8_64"), &got);
        let preload = cmd
            .get_envs()
            .find(|(key, _)| *key == "LD_PRELOAD")
            .and_then(|(_, value)| value)
            .unwrap();
        assert!(preload
            .as_encoded_bytes()
            .starts_with(d.path().join(SDL_COMPAT).as_os_str().as_encoded_bytes()));
        let args: Vec<_> = cmd.get_args().map(|s| s.to_owned()).collect();
        assert_eq!(args.last().unwrap(), rom.as_os_str());
        assert_eq!(args[args.len() - 2], "-run");
        queue(d.path(), &Launch { rom: None }).unwrap();
        let got = take_request(d.path()).unwrap();
        assert!(command(d.path(), Path::new("/official/pico8_64"), &got)
            .get_args()
            .any(|s| s == "-splore"));
    }
    #[test]
    fn configuration_keeps_preferences_and_selects_the_packaged_downloader() {
        let d = root();
        fs::create_dir_all(d.path().join("Config/PICO8")).unwrap();
        fs::write(d.path().join("Config/PICO8/config.txt"), "volume 0\nuse_wget 0\ngui_theme 2\ncdata_path /old/\n").unwrap();
        fs::write(d.path().join("System/pico8-controllers.txt"), "controller fixture").unwrap();
        configure(d.path()).unwrap();
        let config = fs::read_to_string(d.path().join("Config/PICO8/config.txt")).unwrap();
        assert!(config.contains("gui_theme 2\n"));
        assert!(config.contains("use_wget 1\n"));
        assert_eq!(config.lines().filter(|line| line.starts_with("volume ")).collect::<Vec<_>>(), ["volume 256"]);
        assert!(!config.contains("/old/"));
        let cmd = command(d.path(), Path::new("/pico8"), &Launch { rom: None });
        let path = cmd.get_envs().find(|(key, _)| *key == "PATH").unwrap().1.unwrap();
        assert!(path.as_encoded_bytes().starts_with(d.path().join("System/pico8-bin").as_os_str().as_encoded_bytes()));
        let args: Vec<_> = cmd.get_args().collect();
        let home = args.iter().position(|arg| *arg == "-home").unwrap();
        assert!(args[home + 1].as_encoded_bytes().ends_with(b"/"));
    }
    #[test]
    fn outside_game_paths_are_rejected_and_requests_are_not_replayed() {
        let d = root();
        let outside = d.path().join("Config/fake.p8");
        fs::write(&outside, b"data").unwrap();
        queue(d.path(), &Launch { rom: Some(outside) }).unwrap();
        assert!(take_request(d.path()).is_err());
        assert!(!d.path().join(REQUEST).exists());
    }
    #[test]
    fn an_incomplete_update_reports_the_missing_compatibility_file() {
        let d = root();
        runtime(d.path());
        queue(d.path(), &Launch { rom: None }).unwrap();
        let error = run_pending(d.path()).unwrap_err();
        assert!(error.contains("compatibility file missing"));
        assert!(!d.path().join(REQUEST).exists());
    }
    #[test]
    fn splore_marker_is_not_confused_with_a_game_named_splore() {
        let d = root();
        runtime(d.path());
        fs::write(d.path().join("Games/PICO8/Splore.p8"), b"pico-8 cartridge").unwrap();
        let carts = slot_store::scan(d.path()).unwrap();
        assert_eq!(carts.len(), 2);
        let game = carts.iter().find(|c| c.code.is_empty()).unwrap();
        let browser = carts.iter().find(|c| !c.code.is_empty()).unwrap();
        assert_ne!(game.stem, browser.stem);
        assert!(prepare(d.path(), game).unwrap().rom.is_some());
        assert!(prepare(d.path(), browser).unwrap().rom.is_none());
    }
}
