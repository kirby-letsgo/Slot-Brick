//! The two function keys, with separate assignments on the shelf and in a game.
use std::path::Path;

pub const CONTROLS_FILE: &str = "Config/controls.txt";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shortcut {
    Off,
    Search,
    Favorite,
    NextSystem,
    PreviousSystem,
    Sort,
    Menu,
    SaveState,
    LoadState,
    Picture,
    Mute,
    OverlayNext,
}

impl Shortcut {
    pub const SHELF: [Self; 7] = [
        Self::Off,
        Self::Search,
        Self::Favorite,
        Self::NextSystem,
        Self::PreviousSystem,
        Self::Sort,
        Self::Menu,
    ];
    pub const GAME: [Self; 7] = [
        Self::Off,
        Self::SaveState,
        Self::LoadState,
        Self::Menu,
        Self::Picture,
        Self::Mute,
        Self::OverlayNext,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Search => "search",
            Self::Favorite => "favorite",
            Self::NextSystem => "next_system",
            Self::PreviousSystem => "previous_system",
            Self::Sort => "sort",
            Self::Menu => "menu",
            Self::SaveState => "save_state",
            Self::LoadState => "load_state",
            Self::Picture => "picture",
            Self::Mute => "mute",
            Self::OverlayNext => "overlay_next",
        }
    }
}

/// F1 shelf, F2 shelf, F1 game, F2 game. Unknown or inappropriate values retain the default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controls(pub [Shortcut; 4]);

impl Default for Controls {
    fn default() -> Self {
        Self([
            Shortcut::Search,
            Shortcut::Off,
            Shortcut::SaveState,
            Shortcut::LoadState,
        ])
    }
}

impl Controls {
    const KEYS: [&'static str; 4] = ["f1_shelf", "f2_shelf", "f1_game", "f2_game"];

    pub fn choices(index: usize) -> &'static [Shortcut] {
        if index < 2 {
            &Shortcut::SHELF
        } else {
            &Shortcut::GAME
        }
    }

    pub fn parse(text: &str) -> Self {
        let mut controls = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let Some(i) = Self::KEYS.iter().position(|k| *k == key.trim()) else {
                continue;
            };
            if let Some(choice) = Self::choices(i).iter().find(|c| c.name() == value.trim()) {
                controls.0[i] = *choice;
            }
        }
        controls
    }

    pub fn read(root: &Path) -> Self {
        std::fs::read_to_string(root.join(CONTROLS_FILE))
            .map(|s| Self::parse(&s))
            .unwrap_or_default()
    }

    pub fn write(&self, root: &Path) -> std::io::Result<()> {
        let text: String = Self::KEYS
            .iter()
            .zip(self.0)
            .map(|(key, value)| format!("{key}={}\n", value.name()))
            .collect();
        crate::atomic_write(&root.join(CONTROLS_FILE), text.as_bytes())
    }

    pub fn step(&mut self, index: usize, right: bool) {
        let choices = Self::choices(index);
        let at = choices
            .iter()
            .position(|c| *c == self.0[index])
            .unwrap_or(0);
        self.0[index] = choices[(at + if right { 1 } else { choices.len() - 1 }) % choices.len()];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_and_invalid_assignments_are_safe() {
        assert_eq!(
            Controls::parse("f1_shelf=save_state\nf2_game=search\nunknown=off\nbad"),
            Controls::default()
        );
        assert_eq!(
            Controls::parse("f1_shelf=off\nf2_game=mute").0,
            [
                Shortcut::Off,
                Shortcut::Off,
                Shortcut::SaveState,
                Shortcut::Mute
            ]
        );
    }
    #[test]
    fn all_choices_persist_and_cycle_both_ways() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("Config")).unwrap();
        let mut c = Controls::default();
        for i in 0..4 {
            for _ in 0..Controls::choices(i).len() {
                let before = c.clone();
                c.step(i, true);
                c.write(root.path()).unwrap();
                assert_eq!(Controls::read(root.path()), c);
                c.step(i, false);
                assert_eq!(c, before);
                c.step(i, true);
            }
        }
    }
}
