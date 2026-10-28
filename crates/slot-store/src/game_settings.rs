//! Game-specific emulator settings. Kept separately from system/global SlotState.
//! Config/game-settings.txt uses a hex-encoded UTF-8 ROM stem so every entry remains
//! one tab-separated line even when a filename contains punctuation.
use std::collections::HashMap;
use std::path::Path;

use crate::{atomic_write, Platform, Shader, FF_SPEEDS};

pub const GAME_SETTINGS_FILE: &str = "Config/game-settings.txt";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GameSettings {
    /// The FPS counter is opt-in for each game; it is not a system setting.
    pub show_fps: bool,
    /// When enabled, use the largest whole-number multiple of the displayed game
    /// picture that fits the actual output. Off keeps the existing Actual/Stretch mode.
    pub integer_scale: bool,
    pub shader: Option<Shader>,
    pub ff_speed: Option<u8>,
    pub rewind: Option<bool>,
    pub rumble: Option<bool>,
    pub turbo: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameSettingsStore {
    values: HashMap<(Platform, String), GameSettings>,
}

impl GameSettingsStore {
    pub fn read(root: &Path) -> Self {
        std::fs::read_to_string(root.join(GAME_SETTINGS_FILE))
            .map(|s| Self::parse(&s)).unwrap_or_default()
    }

    pub fn parse(text: &str) -> Self {
        let mut values = HashMap::new();
        for line in text.lines() {
            let cells: Vec<_> = line.split('\t').collect();
            // Older settings files have eight columns; integer scaling is an
            // optional ninth, appended to preserve all saved choices.
            let (platform, stem, fps, shader, ff, rewind, rumble, turbo, integer) = match cells.as_slice() {
                [p,s,f,sh,ff,rw,ru,t] => (*p,*s,*f,*sh,*ff,*rw,*ru,*t,None),
                [p,s,f,sh,ff,rw,ru,t,i] => (*p,*s,*f,*sh,*ff,*rw,*ru,*t,Some(*i)),
                _ => continue,
            };
            let Some(platform) = Platform::ALL.into_iter().find(|p| p.dir_name() == *platform) else { continue };
            let Some(stem) = decode(stem) else { continue };
            let Some(show_fps) = parse_bool(fps) else { continue };
            let shader = parse_choice(shader, Shader::parse);
            let ff_speed = parse_choice(ff, |s| s.parse().ok().filter(|v| FF_SPEEDS.contains(v)));
            let rewind = parse_choice(rewind, parse_bool);
            let rumble = parse_choice(rumble, parse_bool);
            let turbo = parse_choice(turbo, parse_bool);
            let integer_scale = integer.and_then(parse_bool).unwrap_or(false);
            let settings = GameSettings { show_fps, integer_scale, shader, ff_speed, rewind, rumble, turbo };
            if settings != GameSettings::default() { values.insert((platform, stem), settings); }
        }
        Self { values }
    }

    pub fn get(&self, platform: Platform, stem: &str) -> GameSettings {
        self.values.get(&(platform, stem.to_owned())).copied().unwrap_or_default()
    }

    pub fn set(&mut self, platform: Platform, stem: &str, settings: GameSettings) {
        let key = (platform, stem.to_owned());
        if settings == GameSettings::default() { self.values.remove(&key); }
        else { self.values.insert(key, settings); }
    }

    pub fn text(&self) -> String {
        let mut lines: Vec<_> = self.values.iter().map(|((platform, stem), s)| {
            format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                platform.dir_name(),
                encode(stem),
                u8::from(s.show_fps),
                s.shader.map_or("-".to_owned(), |v| v.name().to_owned()),
                s.ff_speed.map_or("-".to_owned(), |v| v.to_string()),
                option_bool(s.rewind), option_bool(s.rumble), option_bool(s.turbo),
                u8::from(s.integer_scale))
        }).collect();
        lines.sort();
        lines.concat()
    }

    pub fn write(&self, root: &Path) -> std::io::Result<()> {
        let path=root.join(GAME_SETTINGS_FILE);
        if let Some(parent)=path.parent(){std::fs::create_dir_all(parent)?;}
        atomic_write(&path, self.text().as_bytes())
    }
}

fn option_bool(value: Option<bool>) -> &'static str {
    match value { None => "-", Some(true) => "1", Some(false) => "0" }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value { "1" => Some(true), "0" => Some(false), _ => None }
}

fn parse_choice<T>(s: &str, parse: impl FnOnce(&str) -> Option<T>) -> Option<T> {
    if s == "-" { None } else { parse(s) }
}

fn encode(stem: &str) -> String {
    stem.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn decode(hex: &str) -> Option<String> {
    if hex.len() > 2048 || hex.len() % 2 != 0 || hex.is_empty() { return None; }
    let bytes = (0..hex.len()).step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i+2], 16).ok())
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_per_platform_and_unicode_rom_name() {
        let mut store = GameSettingsStore::default();
        let settings = GameSettings {
            show_fps: true,
            integer_scale: true,
            shader: Some(Shader::Grid),
            ff_speed: Some(3),
            rewind: Some(false),
            rumble: Some(true),
            turbo: Some(false),
        };
        store.set(Platform::Gba, "Pokémon\tBlue=1", settings);
        store.set(Platform::Md, "Pokémon\tBlue=1", GameSettings { show_fps:true, ..Default::default() });
        assert_eq!(GameSettingsStore::parse(&store.text()), store);
        assert_eq!(store.get(Platform::Gba, "Pokémon\tBlue=1"), settings);
        assert_eq!(store.get(Platform::Gg, "Pokémon\tBlue=1"), GameSettings::default());
        let dir=tempfile::tempdir().unwrap();
        store.write(dir.path()).unwrap();
        assert_eq!(GameSettingsStore::read(dir.path()), store);
        store.set(Platform::Gba, "Pokémon\tBlue=1", GameSettings::default());
        assert_eq!(store.get(Platform::Gba, "Pokémon\tBlue=1"), GameSettings::default());
        assert_eq!(GameSettingsStore::parse(&store.text()), store);
    }

    #[test]
    fn invalid_rows_and_unsupported_values_are_ignored() {
        let fake = "GBA\tzz\t1\tgrid\t3\t1\t1\t0\nGBA\t6162\tX\tgrid\t3\t1\t1\t0\nGBA\t6162\t1\tdot\t99\tyes\t1\t0\n";
        let store=GameSettingsStore::parse(fake);
        let settings=store.get(Platform::Gba, "ab");
        assert!(settings.show_fps);
        assert_eq!(settings.ff_speed,None);
        assert_eq!(settings.rewind,None);
        assert_eq!(settings.rumble,Some(true));
        assert!(!settings.integer_scale, "old files default to no integer scaling");
        let mut updated=store.clone();
        let settings=GameSettings { integer_scale:true, ..settings };
        updated.set(Platform::Gba,"ab",settings);
        assert!(GameSettingsStore::parse(&updated.text()).get(Platform::Gba,"ab").integer_scale);
    }

    #[test]
    fn legacy_file_columns_keep_all_existing_overrides() {
        let saved="GBA\t416c706861\t1\tgrid\t3\t0\t1\t0\n";
        let settings=GameSettingsStore::parse(saved).get(Platform::Gba,"Alpha");
        assert!(settings.show_fps);
        assert_eq!(settings.shader,Some(Shader::Grid));
        assert_eq!(settings.ff_speed,Some(3));
        assert_eq!(settings.rewind,Some(false));
        assert_eq!(settings.rumble,Some(true));
        assert_eq!(settings.turbo,Some(false));
        assert!(!settings.integer_scale);
    }
}
