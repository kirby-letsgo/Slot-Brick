//! The official PICO-8 runs in its own process after Slot releases audio, video and input.
use slot_store::{Cart, Platform};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const EXIT_LAUNCH: i32 = 67;
const REQUEST: &str = "Config/pico8-launch";
const ERROR: &str = "Config/pico8-error";

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

fn check_runtime(root: &Path) -> Result<PathBuf, String> {
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
    command
        .current_dir(root.join("BIOS/PICO8"))
        .arg("-home")
        .arg(root.join("Config/PICO8"))
        .arg("-root_path")
        .arg(root.join("Games/PICO8"))
        .arg("-desktop")
        .arg(root.join("Screenshots/PICO8"))
        .args(["-joystick", "0"]);
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
    // Keep PICO-8's preferences, while using Slot's volume and a stable save folder.
    contents = contents
        .lines()
        .filter(|line| {
            !matches!(
                line.split_whitespace().next(),
                Some("volume" | "cdata_path")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let state = slot_store::read_slot_state(root);
    #[cfg(feature = "brick")]
    let headphones = {
        use slot_power::Platform as _;
        slot_power::BrickPlatform::new(root.to_owned()).headphones()
    };
    #[cfg(not(feature = "brick"))]
    let headphones = false;
    let (volume, muted) = if headphones {
        (state.volume_hp, state.muted_hp)
    } else {
        (state.volume, state.muted)
    };
    let volume = if muted {
        0
    } else {
        u32::from(volume.min(100)) * 256 / 100
    };
    contents.push_str(&format!(
        "\nvolume {volume}\ncdata_path {}/\n",
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
    configure(&root)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut mode = fs::metadata(&binary)
            .map_err(|e| e.to_string())?
            .permissions();
        mode.set_mode(mode.mode() | 0o100);
        fs::set_permissions(&binary, mode).map_err(|e| e.to_string())?;
    }
    let log = fs::File::create(root.join("Logs/pico8.log")).map_err(|e| e.to_string())?;
    let err = log.try_clone().map_err(|e| e.to_string())?;
    let mut child = command(&root, &binary, &launch)
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(err)
        .spawn()
        .map_err(|e| format!("Could not launch PICO-8: {e}. See Logs/pico8.log"))?;
    let status = wait_native(&root, &mut child).map_err(|e| {
        let _ = child.kill();
        let _ = child.wait();
        e.to_string()
    })?;
    if !status.success() {
        return Err(format!("PICO-8 exited with {status}. See Logs/pico8.log"));
    }
    Ok(())
}

pub fn record_error(root: &Path, message: &str) {
    eprintln!("slot: PICO-8: {message}");
    let _ = slot_store::atomic_write(&root.join(ERROR), message.as_bytes());
}

#[cfg(any(not(feature = "brick"), feature = "sim"))]
fn wait_native(
    _root: &Path,
    child: &mut std::process::Child,
) -> std::io::Result<std::process::ExitStatus> {
    child.wait()
}

#[cfg(all(feature = "brick", not(feature = "sim")))]
fn wait_native(
    root: &Path,
    child: &mut std::process::Child,
) -> std::io::Result<std::process::ExitStatus> {
    use slot_input::{Action, Gestures, InputSource};
    use slot_power::Platform as _;
    use std::time::{Duration, Instant};
    let mut input = crate::input::DeviceInput::open(root);
    let mut gestures = Gestures::brick();
    let mut power = slot_power::BrickPlatform::new(root.to_owned());
    let mut cpu = crate::brick_power::CpuPower::open(root);
    cpu.mode(crate::brick_power::Mode::Boost, 0);
    let brightness = slot_store::read_slot_state(root).brightness;
    let mut paused = false;
    let started = Instant::now();
    let result = (|| loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        let now = started.elapsed().as_millis() as u64;
        let mut actions = Vec::new();
        for event in input.poll(now) {
            actions.extend(gestures.feed(event, now));
        }
        actions.extend(gestures.tick(now));
        for action in actions {
            match action {
                Action::PowerTap => {
                    let signal = if paused { libc::SIGCONT } else { libc::SIGSTOP };
                    if unsafe { libc::kill(child.id() as i32, signal) } == 0 {
                        paused = !paused;
                        power.set_backlight(if paused { 0 } else { brightness });
                        cpu.mode(
                            if paused {
                                crate::brick_power::Mode::Standby
                            } else {
                                crate::brick_power::Mode::Boost
                            },
                            0,
                        );
                    }
                }
                Action::PowerOff => {
                    unsafe {
                        libc::kill(child.id() as i32, libc::SIGCONT);
                        libc::kill(child.id() as i32, libc::SIGTERM);
                    }
                    let deadline = Instant::now() + Duration::from_secs(3);
                    while child.try_wait()?.is_none() && Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    if child.try_wait()?.is_none() {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                    cpu.restore();
                    power.poweroff();
                }
                _ => {}
            }
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
    fn outside_game_paths_are_rejected_and_requests_are_not_replayed() {
        let d = root();
        let outside = d.path().join("Config/fake.p8");
        fs::write(&outside, b"data").unwrap();
        queue(d.path(), &Launch { rom: Some(outside) }).unwrap();
        assert!(take_request(d.path()).is_err());
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
