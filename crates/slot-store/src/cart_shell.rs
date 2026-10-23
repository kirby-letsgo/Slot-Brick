use std::collections::HashMap;

use unicode_normalization::UnicodeNormalization;

use crate::Platform;

pub const CART_SHELL_FILE: &str = "Config/cart_shell.ini";
pub const LABELS_SHELL_FILE: &str = "Labels/cart_shell.ini";

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Outline {
    Auto,
    Notched,
    Rounded,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ShellFinish {
    Solid,
    Clear,
    Glitter,
    /// Polished metal, as the gold NES carts were moulded.
    Gold,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ShellChoice {
    pub outline: Outline,
    pub colour: [u8; 3],
    pub finish: ShellFinish,
}

impl ShellChoice {
    pub fn parse(value: &str) -> Option<ShellChoice> {
        let mut words = value.split_whitespace();
        let outline = match words.next()? {
            "auto" => Outline::Auto,
            "notched" => Outline::Notched,
            "rounded" => Outline::Rounded,
            _ => return None,
        };
        let hex = words.next()?;
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let colour = [0, 2, 4].map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0));
        let finish = match words.next()? {
            "solid" => ShellFinish::Solid,
            "clear" => ShellFinish::Clear,
            "glitter" => ShellFinish::Glitter,
            "gold" => ShellFinish::Gold,
            _ => return None,
        };
        words.next().is_none().then_some(ShellChoice {
            outline,
            colour,
            finish,
        })
    }

    pub fn to_value(&self) -> String {
        let outline = match self.outline {
            Outline::Auto => "auto",
            Outline::Notched => "notched",
            Outline::Rounded => "rounded",
        };
        let finish = match self.finish {
            ShellFinish::Solid => "solid",
            ShellFinish::Clear => "clear",
            ShellFinish::Glitter => "glitter",
            ShellFinish::Gold => "gold",
        };
        let [r, g, b] = self.colour;
        format!("{outline} {r:02x}{g:02x}{b:02x} {finish}")
    }
}

pub fn key(stem: &str) -> String {
    stem.nfc().collect()
}

pub fn choices(text: &str) -> HashMap<String, ShellChoice> {
    crate::ini::parse(text)
        .into_iter()
        .filter_map(|(stem, value)| Some((key(&stem), ShellChoice::parse(&value)?)))
        .collect()
}

/// Each game's choice, by key: `None` where the labels layer said "auto" (the game's own
/// plastic), so the system layer's choice for it no longer counts.
pub fn layered(system: &str, labels: &str) -> HashMap<String, Option<ShellChoice>> {
    let mut out: HashMap<String, Option<ShellChoice>> = choices(system)
        .into_iter()
        .map(|(k, c)| (k, Some(c)))
        .collect();
    for (stem, value) in crate::ini::parse(labels) {
        if value.trim() == "auto" {
            out.insert(key(&stem), None);
        } else if let Some(choice) = ShellChoice::parse(&value) {
            out.insert(key(&stem), Some(choice));
        }
    }
    out
}

/// A game's choice: the one kept under its system (what Slot writes), else one kept under its
/// bare name (older Slots, other tools), for the systems that had those.
pub fn choice_for(
    shells: &HashMap<String, Option<ShellChoice>>,
    platform: Platform,
    stem: &str,
) -> Option<ShellChoice> {
    shells
        .get(&key(&platform.game_key(stem)))
        .or_else(|| {
            shells
                .get(&key(stem))
                .filter(|_| platform.reads_bare_keys())
        })
        .copied()
        .flatten()
}

/// Whether Reset must write "auto" under the system key: only when a bare entry in either layer
/// would otherwise come back. Otherwise the system key is just removed.
pub fn reset_hides_something(system: &str, labels: &str, platform: Platform, stem: &str) -> bool {
    let bare = key(stem);
    platform.reads_bare_keys()
        && [system, labels]
            .iter()
            .any(|text| crate::ini::parse(text).keys().any(|k| key(k) == bare))
}
