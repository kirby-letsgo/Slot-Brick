//! Optional, experimental standalone Flycast on TrimUI Brick.
//!
//! Flycast owns SDL video/audio/input in a separate process. Slot exits first,
//! then the launcher returns to the shelf. A third-party runtime is *not*
//! bundled: install a compatible AArch64 Flycast and its runtime libraries.
//! No shell ever receives a ROM-derived command string.
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use slot_store::{Cart, GameSettingsStore, Platform};

pub const REQUEST: &str = "Config/dreamcast-launch";
pub const ERROR: &str = "Config/dreamcast-error";
pub const BINARY: &str = "System/Flycast/flycast";
pub const TEMPLATE: &str = "System/Flycast/emu.cfg";
pub const MAPPING: &str = "System/Flycast/mappings/SDL_Xbox 360 Controller.cfg";
const AUDIO_BRIDGE: &str = "System/pico8-sdl-nosensor.so";
const BIOS: &str = "BIOS/DC/dc_boot.bin";
const CONFIG: &str = "Config/DC/emu.cfg";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    pub rom: PathBuf,
}

pub fn check_runtime(root: &Path) -> Result<PathBuf, String> {
    let binary = root.join(BINARY);
    let mut header = [0u8; 20];
    fs::File::open(&binary).and_then(|mut f| f.read_exact(&mut header))
        .map_err(|_| "Install the optional Flycast runtime with brick/setup-dreamcast.py. See docs/DREAMCAST.md")?;
    if &header[..4] != b"\x7fELF" || header[4] != 2 || header[5] != 1
        || u16::from_le_bytes([header[18],header[19]]) != 183 {
        return Err("Flycast must be a Linux ARM64 executable for the TrimUI Brick".into());
    }
    if !root.join(TEMPLATE).is_file() || !root.join(MAPPING).is_file() {
        return Err("Flycast config/mapping not found. Re-run brick/setup-dreamcast.py".into());
    }
    if !root.join(AUDIO_BRIDGE).is_file() {
        return Err("Slot's native audio bridge is missing. Reinstall the Slot card package".into());
    }
    Ok(binary)
}

/// The tested standalone build is configured for a real BIOS by default.
/// This is only a file check, not a promise that an arbitrary dump is valid.
pub fn check_bios(root: &Path) -> Result<(), String> {
    let path = root.join(BIOS);
    if path.metadata().is_ok_and(|m| m.is_file() && m.len()==2*1024*1024) {
        Ok(())
    } else {
        Err("Add your own 2 MiB dc_boot.bin to Slot/BIOS/DC. See docs/DREAMCAST.md".into())
    }
}

fn validate(root: &Path, launch: &Launch) -> Result<(), String> {
    let base=fs::canonicalize(root.join("Games/DC"))
        .map_err(|_| "Dreamcast games directory is missing")?;
    let rom=fs::canonicalize(&launch.rom)
        .map_err(|_| "Dreamcast game file is missing")?;
    if !rom.starts_with(&base) || !rom.is_file() || !Platform::Dc.accepts(&rom) {
        return Err("Choose a .chd, .gdi, .cdi, .cue or .m3u inside Slot/Games/DC".into());
    }
    Ok(())
}

pub fn prepare(root: &Path, cart: &Cart) -> Result<Launch, String> {
    if cart.platform!=Platform::Dc {return Err("Not a Dreamcast game".into());}
    check_runtime(root)?;
    check_bios(root)?;
    let launch=Launch {rom:cart.rom.clone()};
    validate(root,&launch)?;
    Ok(launch)
}

/// Only a ROM path goes in the launch request. Consume it once so a crash
/// cannot replay stale content when the native launcher restarts.
pub fn queue(root: &Path, launch: &Launch) -> Result<(), String> {
    validate(root,launch)?;
    let text=launch.rom.to_str().ok_or("Dreamcast game path is not UTF-8")?;
    if text.len()>4096 || text.contains('\n') || text.contains('\r') {
        return Err("Dreamcast game path cannot be queued".into());
    }
    slot_store::atomic_write(&root.join(REQUEST),text.as_bytes()).map_err(|e|e.to_string())
}

pub fn pending(root: &Path) -> bool {root.join(REQUEST).is_file()}

fn take_request(root: &Path) -> Result<Launch,String> {
    let path=root.join(REQUEST);
    let text=fs::read_to_string(&path).map_err(|_|"No Dreamcast game was requested")?;
    fs::remove_file(&path).map_err(|e|e.to_string())?;
    if text.len()>4096 || text.contains('\n') || text.contains('\r') {
        return Err("Invalid Dreamcast launch request".into());
    }
    let launch=Launch {rom:PathBuf::from(text)};
    validate(root,&launch)?;
    Ok(launch)
}

/// Replace just the FPS line so preferences saved by Flycast survive closing a
/// game. The original value is restored when the child exits.
fn set_fps(cfg:&str,on:bool)->String {
    let value=if on {"yes"}else{"no"};
    let mut out=String::new();
    let mut found=false;
    for line in cfg.lines(){
        if line.trim_start().starts_with("rend.ShowFPS") &&
            line.split_once('=').is_some_and(|(k,_)|k.trim()=="rend.ShowFPS") {
            if !found {out.push_str(&format!("rend.ShowFPS = {value}\n"));found=true;}
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !found {
        if !out.contains("[config]") {out.push_str("[config]\n");}
        out.push_str(&format!("rend.ShowFPS = {value}\n"));
    }
    out
}

/// The upstream Xbox mapping assigns Select (6) to settings and HOME (8)
/// to exit. On Brick, HOME and its Pro alias (15) must open the native menu.
fn menu_mapping(mapping: &str) -> String {
    let mut out = Vec::new();
    let mut digital = false;
    let mut bindings = Vec::new();
    for line in mapping.lines().chain(std::iter::once("[slot-end]")) {
        if line.trim().starts_with('[') {
            if digital {
                let next = bindings.iter().filter_map(|line: &String|
                    line.trim().strip_prefix("bind")?.split_once('=')?.0.trim().parse::<usize>().ok()
                ).max().map_or(0, |n| n + 1);
                out.append(&mut bindings);
                out.push(format!("bind{next} = 8:btn_menu"));
                out.push(format!("bind{} = 15:btn_menu", next + 1));
            }
            digital = line.trim() == "[digital]";
            if line != "[slot-end]" { out.push(line.to_owned()); }
        } else if digital {
            let binding = line.split_once('=').and_then(|(key, value)|
                key.trim().starts_with("bind").then_some(value.trim())
            ).and_then(|value| value.split_once(':'));
            let reserved = binding.is_some_and(|(button, action)|
                matches!(button.trim(), "8" | "15") ||
                (button.trim() == "6" && matches!(action.trim(), "btn_menu" | "btn_escape")));
            if !reserved { bindings.push(line.to_owned()); }
        } else {
            out.push(line.to_owned());
        }
    }
    if !mapping.lines().any(|line| line.trim() == "[digital]") {
        out.extend(["[digital]".into(), "bind0 = 8:btn_menu".into(), "bind1 = 15:btn_menu".into()]);
    }
    out.join("\n") + "\n"
}

fn configure(root:&Path)->Result<(),String>{
    for dir in ["Config/DC","Config/DC/mappings","Saves/DC","Screenshots/DC","Logs"] {
        fs::create_dir_all(root.join(dir)).map_err(|e|e.to_string())?;
    }
    if !root.join(CONFIG).is_file() {
        fs::copy(root.join(TEMPLATE),root.join(CONFIG))
            .map_err(|e|format!("Flycast default configuration: {e}"))?;
    }
    let map=root.join("Config/DC/mappings/SDL_Xbox 360 Controller.cfg");
    if !map.is_file(){
        fs::copy(root.join(MAPPING),&map).map_err(|e|format!("Flycast controller mapping: {e}"))?;
    }
    let mapping = fs::read_to_string(&map).map_err(|e| e.to_string())?;
    let corrected = menu_mapping(&mapping);
    if corrected != mapping {
        slot_store::atomic_write(&map, corrected.as_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn command(root:&Path,binary:&Path,launch:&Launch)->Command {
    let mut cmd=Command::new(binary);
    let mut preload = root.join(AUDIO_BRIDGE).into_os_string();
    if let Some(inherited) = std::env::var_os("LD_PRELOAD").filter(|s| !s.is_empty()) {
        preload.push(":");
        preload.push(inherited);
    }
    cmd.env("LD_PRELOAD", preload);
    let mut libs=root.join("System/Flycast/lib").into_os_string();
    libs.push(":");
    libs.push(std::env::var_os("LD_LIBRARY_PATH").unwrap_or_else(|| "/usr/trimui/lib:/usr/lib:/lib".into()));
    let mut bios=root.join("BIOS/DC").into_os_string();
    bios.push("/");
    let mut config=root.join("Config/DC").into_os_string();
    config.push("/");
    let mut saves=root.join("Saves/DC").into_os_string();
    saves.push("/");
    cmd.current_dir(root)
        .arg(&launch.rom)
        .env("LD_LIBRARY_PATH",libs)
        .env("HOME",root.join("Config/DC"))
        .env("FLYCAST_BIOS_DIR",bios)
        .env("FLYCAST_CONFIG_DIR",config)
        .env("FLYCAST_DATA_DIR",saves)
        .env("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS","1")
        .env("SLOT_ROOT",root);
    cmd
}

/// Called after the frontend has relinquished graphics, audio, and input.
/// Saves/VMU state belong to Flycast rather than Slot's libretro savestates.
pub fn run_pending(root:&Path)->Result<(),String>{
    let root=fs::canonicalize(root).map_err(|e|e.to_string())?;
    let launch=take_request(&root)?;
    let binary=check_runtime(&root)?;
    check_bios(&root)?;
    configure(&root)?;

    let settings=GameSettingsStore::read(&root).get(Platform::Dc,
        launch.rom.file_stem().and_then(|s|s.to_str()).unwrap_or(""));
    let config_path=root.join(CONFIG);
    let initial=fs::read_to_string(&config_path).map_err(|e|e.to_string())?;
    slot_store::atomic_write(&config_path,set_fps(&initial,settings.show_fps).as_bytes())
        .map_err(|e|e.to_string())?;

    let result=run_child(&root,&binary,&launch);
    // Flycast can save other settings while running. Restore only the FPS flag
    // to avoid overwriting those user edits on exit.
    if let Ok(latest)=fs::read_to_string(&config_path) {
        let restored=initial.lines().find_map(|line|
            line.split_once('=').filter(|(k,_)|k.trim()=="rend.ShowFPS")
                .map(|(_,v)|v.trim()=="yes")
        ).unwrap_or(false);
        if let Err(e)=slot_store::atomic_write(&config_path,set_fps(&latest,restored).as_bytes()) {
            eprintln!("slot: Flycast FPS preference: {e}");
        }
    }
    result
}

fn run_child(root:&Path,binary:&Path,launch:&Launch)->Result<(),String>{
    use std::os::{fd::AsRawFd,unix::process::CommandExt};
    let log=fs::File::create(root.join("Logs/dreamcast.log")).map_err(|e|e.to_string())?;
    let err=log.try_clone().map_err(|e|e.to_string())?;
    let mut audio=crate::pico8_audio::NativeAudio::new(root);
    let volume=crate::pico8_audio::volume_channel(audio.output())?;
    let mut cmd=command(root,binary,launch);
    cmd.env("SLOT_PICO8_VOLUME_FD",volume.as_raw_fd().to_string())
        .env("SLOT_PICO8_VOLUME",audio.output().to_string())
        .stdin(Stdio::null()).stdout(log).stderr(err).process_group(0);
    let mut child=cmd.spawn().map_err(|e|format!("Flycast launch: {e}. See Logs/dreamcast.log"))?;
    let result=crate::pico8::wait_native(root,&mut child,&mut audio,&volume,true);
    // Release any subprocesses, never another user's unrelated process.
    unsafe{libc::kill(-(child.id() as i32),libc::SIGKILL);}
    let _=child.wait();
    audio.persist(root);
    let status=result.map_err(|e|e.to_string())?;
    if !status.success(){return Err(format!("Flycast exited with {status}. See Logs/dreamcast.log"));}
    Ok(())
}

pub fn record_error(root:&Path,message:&str){
    eprintln!("slot: Dreamcast: {message}");
    let _=slot_store::atomic_write(&root.join(ERROR),message.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root()->tempfile::TempDir{
        let d=tempfile::tempdir().unwrap();
        crate::root::ensure(d.path());
        d
    }
    fn runtime(root:&Path){
        let mut elf=[0u8;64];elf[..6].copy_from_slice(b"\x7fELF\x02\x01");elf[18]=183;
        fs::create_dir_all(root.join("System/Flycast/mappings")).unwrap();
        fs::write(root.join(BINARY),elf).unwrap();
        fs::write(root.join(AUDIO_BRIDGE),b"test audio bridge").unwrap();
        fs::write(root.join(TEMPLATE),"[config]\nrend.ShowFPS = no\n").unwrap();
        fs::write(root.join(MAPPING),"[emulator]\nversion = 3\n").unwrap();
        fs::write(root.join(BIOS),vec![0u8;2*1024*1024]).unwrap();
    }

    #[test]
    fn launch_requires_compatible_runtime_and_user_bios() {
        let d=root();
        assert!(check_runtime(d.path()).is_err());
        runtime(d.path());
        assert!(check_runtime(d.path()).is_ok());
        assert!(check_bios(d.path()).is_ok());
        fs::remove_file(d.path().join(BIOS)).unwrap();
        assert!(check_bios(d.path()).is_err());
        fs::write(d.path().join(BIOS),vec![0u8;128]).unwrap();
        assert!(check_bios(d.path()).is_err());
        fs::write(d.path().join(BINARY),b"#!/bin/sh").unwrap();
        assert!(check_runtime(d.path()).is_err());
    }

    #[test]
    fn request_is_single_use_and_cannot_escape_the_games_directory() {
        let d=root();runtime(d.path());
        let rom=d.path().join("Games/DC/Test ' ; $(do-not-run).gdi");
        fs::write(&rom,"1 1 4 2352 file.bin 0\n").unwrap();
        let launch=Launch{rom:rom.clone()};
        queue(d.path(),&launch).unwrap();
        assert_eq!(take_request(d.path()).unwrap(),launch);
        assert!(take_request(d.path()).is_err());
        let cmd=command(d.path(),&d.path().join(BINARY),&launch);
        assert_eq!(cmd.get_args().collect::<Vec<_>>(),vec![rom.as_os_str()]);
        let outside=d.path().join("Config/Outside.chd");
        fs::write(&outside,b"not a Dreamcast disc").unwrap();
        assert!(queue(d.path(),&Launch{rom:outside}).is_err());
    }

    #[test]
    fn fps_config_toggle_preserves_unrelated_values() {
        let cfg="[config]\nrend.ShowFPS = no\nrend.Resolution = 480\n";
        assert!(set_fps(cfg,true).contains("rend.ShowFPS = yes\n"));
        assert!(set_fps(cfg,true).contains("rend.Resolution = 480\n"));
        assert_eq!(set_fps(&set_fps(cfg,true),false),cfg);
    }

    #[test]
    fn brick_menu_opens_settings_without_select_or_exit_and_migrates_existing_maps() {
        let original = "[analog]\nbind0 = 0-:btn_analog_left\n[digital]\nbind0 = 0:btn_a\nbind6 = 6:btn_menu\nbind8 = 8:btn_escape\nbind9 = 9:btn_jump_state\n[emulator]\nversion = 3\n";
        let corrected = menu_mapping(original);
        assert!(corrected.contains("8:btn_menu"));
        assert!(corrected.contains("15:btn_menu"));
        assert!(!corrected.contains("6:btn_menu"));
        assert!(!corrected.contains("btn_escape"));
        assert!(corrected.contains("bind0 = 0:btn_a"));
        assert!(corrected.contains("bind0 = 0-:btn_analog_left"));
        assert!(corrected.contains("bind9 = 9:btn_jump_state"));
        assert_eq!(menu_mapping(&corrected), corrected);
        let d = root(); runtime(d.path());
        configure(d.path()).unwrap();
        let map = d.path().join("Config/DC/mappings/SDL_Xbox 360 Controller.cfg");
        fs::write(&map, original).unwrap();
        configure(d.path()).unwrap();
        assert_eq!(fs::read_to_string(map).unwrap(), corrected);
    }

    #[test]
    fn native_audio_bridge_is_required_and_preloaded_for_flycast() {
        let d = root(); runtime(d.path());
        let launch = Launch { rom: d.path().join("Games/DC/Game.chd") };
        let cmd = command(d.path(), &d.path().join(BINARY), &launch);
        let preload = cmd.get_envs().find(|(key, _)| *key == "LD_PRELOAD")
            .and_then(|(_, value)| value).unwrap().to_string_lossy();
        assert!(preload.starts_with(d.path().join(AUDIO_BRIDGE).to_str().unwrap()));
        fs::remove_file(d.path().join(AUDIO_BRIDGE)).unwrap();
        assert!(check_runtime(d.path()).unwrap_err().contains("audio bridge"));
    }

    #[test]
    fn config_copy_and_original_controller_mapping_are_only_seeded_once(){
        let d=root();runtime(d.path());
        configure(d.path()).unwrap();
        let config=d.path().join(CONFIG);
        fs::write(&config,b"[config]\nrend.Resolution = 240\n").unwrap();
        configure(d.path()).unwrap();
        assert!(fs::read_to_string(&config).unwrap().contains("Resolution = 240"));
    }
}
