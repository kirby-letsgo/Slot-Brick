use crate::draw::{Draw, TexId, OUT_H, OUT_W};
use crate::plate::{arrows_hint_face, centred_hints, hint_face, UndoFace, HINT_H, LEGEND_GAP, LEGEND_Y};
use crate::power_menu::{MENU_H, MENU_INK, MENU_PAD, MENU_PX};
use crate::slot_chrome::{edge, opening};
use crate::text;

/// The Brick Pro: set once at startup, it adds the Brick Pro's own rows (the stick lights).
static BRICK_PRO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_brick_pro(pro: bool) {
    BRICK_PRO.store(pro, std::sync::atomic::Ordering::Relaxed);
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickRow {
    FastForward,
    FastForwardSound,
    Screen,
    Rumble,
    DateTime,
    About,
    GbaShader,
    GbShader,
    GbaColourCorrection,
    Game,
    EjectSave,
    Turbo,
    Rewind,
    GbPalettes,
    Lights,
    LedMode,
    LedTop,
    LedButtons,
    LedTriggers,
    LedColor,
    LedEffect,
    Wifi,
    CartArt,
    Achievements,
    ScreenGb,
    ScreenGba,
    ScreenNes,
    ScreenSnes,
    GbColourCorrection,
    NesShader,
    SnesShader,
    GbOverlay,
    GbcOverlay,
    GbaOverlay,
    NesOverlay,
    SnesOverlay,
    AutoSave,
    BatteryShutdown,
    Update,
    System,
    AutoClock,
    LedSticks,
    Library,
    LibrarySearch,
    LibrarySort,
    LibraryShow,
    LibraryPlayTime,
    LibraryTitleCard,
    ScreenPs1,
    Ps1Shader,
    Ps1Dithering,
    Ps1Overlay,
    AutoSleep,
    ScreenMd,
    ScreenGg,
    MdShader,
    GgShader,
    MdOverlay,
    GgOverlay,
    ScreenSms,
    ScreenN64,
    ScreenNgp,
    ScreenS32x,
    ScreenSegaCd,
    SmsShader,
    N64Shader,
    NgpShader,
    S32xShader,
    SegaCdShader,
    SmsOverlay,
    N64Overlay,
    NgpOverlay,
    S32xOverlay,
    SegaCdOverlay,
    Controls,
    F1Shelf,
    F2Shelf,
    F1Game,
    F2Game,

}

impl QuickRow {
    pub const ALL: [QuickRow; 79] = [
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Screen,
        QuickRow::Rumble,
        QuickRow::DateTime,
        QuickRow::About,
        QuickRow::GbaShader,
        QuickRow::GbShader,
        QuickRow::GbaColourCorrection,
        QuickRow::Game,
        QuickRow::EjectSave,
        QuickRow::Turbo,
        QuickRow::Rewind,
        QuickRow::GbPalettes,
        QuickRow::Lights,
        QuickRow::LedMode,
        QuickRow::LedTop,
        QuickRow::LedButtons,
        QuickRow::LedTriggers,
        QuickRow::LedColor,
        QuickRow::LedEffect,
        QuickRow::Wifi,
        QuickRow::CartArt,
        QuickRow::Achievements,
        QuickRow::ScreenGb,
        QuickRow::ScreenGba,
        QuickRow::ScreenNes,
        QuickRow::ScreenSnes,
        QuickRow::GbColourCorrection,
        QuickRow::NesShader,
        QuickRow::SnesShader,
        QuickRow::GbOverlay,
        QuickRow::GbcOverlay,
        QuickRow::GbaOverlay,
        QuickRow::NesOverlay,
        QuickRow::SnesOverlay,
        QuickRow::AutoSave,
        QuickRow::BatteryShutdown,
        QuickRow::Update,
        QuickRow::System,
        QuickRow::AutoClock,
        QuickRow::LedSticks,
        QuickRow::Library,
        QuickRow::LibrarySearch,
        QuickRow::LibrarySort,
        QuickRow::LibraryShow,
        QuickRow::LibraryPlayTime,
        QuickRow::LibraryTitleCard,
        QuickRow::ScreenPs1,
        QuickRow::Ps1Shader,
        QuickRow::Ps1Dithering,
        QuickRow::Ps1Overlay,
        QuickRow::AutoSleep,
        QuickRow::ScreenMd,
        QuickRow::ScreenGg,
        QuickRow::MdShader,
        QuickRow::GgShader,
        QuickRow::MdOverlay,
        QuickRow::GgOverlay,
        QuickRow::ScreenSms,
        QuickRow::ScreenN64,
        QuickRow::ScreenNgp,
        QuickRow::ScreenS32x,
        QuickRow::ScreenSegaCd,
        QuickRow::SmsShader,
        QuickRow::N64Shader,
        QuickRow::NgpShader,
        QuickRow::S32xShader,
        QuickRow::SegaCdShader,
        QuickRow::SmsOverlay,
        QuickRow::N64Overlay,
        QuickRow::NgpOverlay,
        QuickRow::S32xOverlay,
        QuickRow::SegaCdOverlay,
        QuickRow::Controls,
        QuickRow::F1Shelf,
        QuickRow::F2Shelf,
        QuickRow::F1Game,
        QuickRow::F2Game,

    ];

    #[cfg(not(feature = "brick"))]
    pub const MAIN: [QuickRow; 4] = [QuickRow::Screen, QuickRow::Game, QuickRow::Library, QuickRow::System];
    #[cfg(not(feature = "brick"))]
    pub const SYSTEM: [QuickRow; 2] = [QuickRow::DateTime, QuickRow::About];

    #[cfg(feature = "brick")]
    pub const MAIN: [QuickRow; 8] = [QuickRow::Screen,QuickRow::Game,QuickRow::Library,QuickRow::Wifi,QuickRow::CartArt,QuickRow::Achievements,QuickRow::Update,QuickRow::System];
    /// System: the Brick's lights, its clock, and what Slot is.
    #[cfg(feature = "brick")]
    pub const SYSTEM: [QuickRow; 5] = [QuickRow::Lights,QuickRow::DateTime,QuickRow::AutoClock,QuickRow::About,QuickRow::Controls];
    pub const CONTROLS: [QuickRow; 4] = [QuickRow::F1Shelf, QuickRow::F2Shelf, QuickRow::F1Game, QuickRow::F2Game];
    pub fn control_index(self) -> Option<usize> { Self::CONTROLS.iter().position(|r| *r == self) }

    pub const LIGHTS: [QuickRow; 6] = [QuickRow::LedMode,QuickRow::LedTop,QuickRow::LedButtons,QuickRow::LedTriggers,QuickRow::LedColor,QuickRow::LedEffect];
    /// The Brick Pro's: its sticks have lights of their own.
    pub const LIGHTS_PRO: [QuickRow; 7] = [QuickRow::LedMode,QuickRow::LedTop,QuickRow::LedButtons,QuickRow::LedTriggers,QuickRow::LedSticks,QuickRow::LedColor,QuickRow::LedEffect];
    /// The LEDs page of the Brick this is (see `set_brick_pro`).
    pub fn lights() -> &'static [QuickRow] {
        match BRICK_PRO.load(std::sync::atomic::Ordering::Relaxed) {
            true => &QuickRow::LIGHTS_PRO,
            false => &QuickRow::LIGHTS,
        }
    }
    pub fn led_setting(self) -> bool { Self::LIGHTS_PRO.contains(&self) }

    /// Library: how the shelves are ordered, whether they hold only the favorites, and what is
    /// printed around them.
    #[cfg(not(feature = "brick"))]
    pub const LIBRARY: [QuickRow; 4] = [QuickRow::LibrarySort, QuickRow::LibraryShow, QuickRow::LibraryPlayTime, QuickRow::LibraryTitleCard];
    #[cfg(feature = "brick")]
    pub const LIBRARY: [QuickRow; 5] = [QuickRow::LibrarySearch, QuickRow::LibrarySort, QuickRow::LibraryShow, QuickRow::LibraryPlayTime, QuickRow::LibraryTitleCard];

    /// Screen: a page for each system, holding that system's own picture settings.
    pub const SCREEN: [QuickRow; 12] = [
        QuickRow::ScreenGb,
        QuickRow::ScreenGba,
        QuickRow::ScreenNes,
        QuickRow::ScreenSnes,
        QuickRow::ScreenMd,
        QuickRow::ScreenGg,
        QuickRow::ScreenSms,
        QuickRow::ScreenN64,
        QuickRow::ScreenNgp,
        QuickRow::ScreenS32x,
        QuickRow::ScreenSegaCd,
        QuickRow::ScreenPs1,
    ];
    pub const SCREEN_GB: [QuickRow; 5] =
        [QuickRow::GbShader, QuickRow::GbColourCorrection, QuickRow::GbPalettes, QuickRow::GbOverlay, QuickRow::GbcOverlay];
    pub const SCREEN_GBA: [QuickRow; 3] = [QuickRow::GbaShader, QuickRow::GbaColourCorrection, QuickRow::GbaOverlay];
    pub const SCREEN_NES: [QuickRow; 2] = [QuickRow::NesShader, QuickRow::NesOverlay];
    pub const SCREEN_SNES: [QuickRow; 2] = [QuickRow::SnesShader, QuickRow::SnesOverlay];
    pub const SCREEN_MD: [QuickRow; 2] = [QuickRow::MdShader, QuickRow::MdOverlay];
    pub const SCREEN_GG: [QuickRow; 2] = [QuickRow::GgShader, QuickRow::GgOverlay];
    pub const SCREEN_SMS: [QuickRow; 2] = [QuickRow::SmsShader, QuickRow::SmsOverlay];
    pub const SCREEN_N64: [QuickRow; 2] = [QuickRow::N64Shader, QuickRow::N64Overlay];
    pub const SCREEN_NGP: [QuickRow; 2] = [QuickRow::NgpShader, QuickRow::NgpOverlay];
    pub const SCREEN_32X: [QuickRow; 2] = [QuickRow::S32xShader, QuickRow::S32xOverlay];
    pub const SCREEN_SEGACD: [QuickRow; 2] = [QuickRow::SegaCdShader, QuickRow::SegaCdOverlay];
    pub const SCREEN_PS1: [QuickRow; 3] = [
        QuickRow::Ps1Shader,
        QuickRow::Ps1Dithering,
        QuickRow::Ps1Overlay,
    ];

    /// The system an Overlay row picks the overlay for.
    pub fn overlay_of(self) -> Option<slot_store::Platform> {
        use slot_store::Platform;
        Some(match self {
            QuickRow::GbOverlay => Platform::Gb,
            QuickRow::GbcOverlay => Platform::Gbc,
            QuickRow::GbaOverlay => Platform::Gba,
            QuickRow::NesOverlay => Platform::Nes,
            QuickRow::SnesOverlay => Platform::Snes,
            QuickRow::Ps1Overlay => Platform::Ps1,
            QuickRow::MdOverlay => Platform::Md,
            QuickRow::GgOverlay => Platform::Gg,
            QuickRow::SmsOverlay => Platform::Sms,
            QuickRow::N64Overlay => Platform::N64,
            QuickRow::NgpOverlay => Platform::Ngp,
            QuickRow::S32xOverlay => Platform::S32x,
            QuickRow::SegaCdOverlay => Platform::SegaCd,
            _ => return None,
        })
    }

    pub const GAME: [QuickRow; 9] = [
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Rewind,
        QuickRow::Turbo,
        QuickRow::Rumble,
        QuickRow::EjectSave,
        QuickRow::AutoSave,
        QuickRow::BatteryShutdown,
        QuickRow::AutoSleep,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn page(self) -> &'static [QuickRow] {
        match self {
            QuickRow::LedMode
            | QuickRow::LedTop
            | QuickRow::LedButtons
            | QuickRow::LedTriggers
            | QuickRow::LedSticks
            | QuickRow::LedColor
            | QuickRow::LedEffect => QuickRow::lights(),
            QuickRow::ScreenGb
            | QuickRow::ScreenGba
            | QuickRow::ScreenNes
            | QuickRow::ScreenSnes
            | QuickRow::ScreenMd
            | QuickRow::ScreenGg
            | QuickRow::ScreenSms
            | QuickRow::ScreenN64
            | QuickRow::ScreenNgp
            | QuickRow::ScreenS32x
            | QuickRow::ScreenSegaCd
            | QuickRow::ScreenPs1 => &QuickRow::SCREEN,
            QuickRow::GbShader
            | QuickRow::GbColourCorrection
            | QuickRow::GbPalettes
            | QuickRow::GbOverlay
            | QuickRow::GbcOverlay => &QuickRow::SCREEN_GB,
            QuickRow::GbaShader | QuickRow::GbaColourCorrection | QuickRow::GbaOverlay => &QuickRow::SCREEN_GBA,
            QuickRow::NesShader | QuickRow::NesOverlay => &QuickRow::SCREEN_NES,
            QuickRow::SnesShader | QuickRow::SnesOverlay => &QuickRow::SCREEN_SNES,
            QuickRow::MdShader | QuickRow::MdOverlay => &QuickRow::SCREEN_MD,
            QuickRow::GgShader | QuickRow::GgOverlay => &QuickRow::SCREEN_GG,
            QuickRow::SmsShader | QuickRow::SmsOverlay => &QuickRow::SCREEN_SMS,
            QuickRow::N64Shader | QuickRow::N64Overlay => &QuickRow::SCREEN_N64,
            QuickRow::NgpShader | QuickRow::NgpOverlay => &QuickRow::SCREEN_NGP,
            QuickRow::S32xShader | QuickRow::S32xOverlay => &QuickRow::SCREEN_32X,
            QuickRow::SegaCdShader | QuickRow::SegaCdOverlay => &QuickRow::SCREEN_SEGACD,
            QuickRow::Ps1Shader | QuickRow::Ps1Dithering | QuickRow::Ps1Overlay => {
                &QuickRow::SCREEN_PS1
            }
            QuickRow::FastForward
            | QuickRow::FastForwardSound
            | QuickRow::Rewind
            | QuickRow::Turbo
            | QuickRow::Rumble
            | QuickRow::EjectSave
            | QuickRow::AutoSave
            | QuickRow::BatteryShutdown
            | QuickRow::AutoSleep => &QuickRow::GAME,
            QuickRow::Controls | QuickRow::Lights | QuickRow::DateTime | QuickRow::AutoClock | QuickRow::About => &QuickRow::SYSTEM,
            QuickRow::LibrarySearch | QuickRow::LibrarySort | QuickRow::LibraryShow | QuickRow::LibraryPlayTime | QuickRow::LibraryTitleCard => &QuickRow::LIBRARY,
            QuickRow::F1Shelf | QuickRow::F2Shelf | QuickRow::F1Game | QuickRow::F2Game => &QuickRow::CONTROLS,
            _ => &QuickRow::MAIN,
        }
    }

    pub fn parent(self) -> Option<QuickRow> {
        match self.page()[0] {
            QuickRow::F1Shelf => Some(QuickRow::Controls),
            QuickRow::LedMode => Some(QuickRow::Lights),
            QuickRow::ScreenGb => Some(QuickRow::Screen),
            QuickRow::GbShader => Some(QuickRow::ScreenGb),
            QuickRow::GbaShader => Some(QuickRow::ScreenGba),
            QuickRow::NesShader => Some(QuickRow::ScreenNes),
            QuickRow::SnesShader => Some(QuickRow::ScreenSnes),
            QuickRow::MdShader => Some(QuickRow::ScreenMd),
            QuickRow::GgShader => Some(QuickRow::ScreenGg),
            QuickRow::SmsShader => Some(QuickRow::ScreenSms),
            QuickRow::N64Shader => Some(QuickRow::ScreenN64),
            QuickRow::NgpShader => Some(QuickRow::ScreenNgp),
            QuickRow::S32xShader => Some(QuickRow::ScreenS32x),
            QuickRow::SegaCdShader => Some(QuickRow::ScreenSegaCd),
            QuickRow::Ps1Shader => Some(QuickRow::ScreenPs1),
            QuickRow::FastForward => Some(QuickRow::Game),
            QuickRow::LibrarySearch | QuickRow::LibrarySort => Some(QuickRow::Library),
            first if first == QuickRow::SYSTEM[0] => Some(QuickRow::System),
            _ => None,
        }
    }

    pub fn child(self) -> Option<QuickRow> {
        match self {
            QuickRow::Controls => Some(QuickRow::F1Shelf),
            QuickRow::Lights => Some(QuickRow::lights()[0]),
            QuickRow::Screen => Some(QuickRow::SCREEN[0]),
            QuickRow::ScreenGb => Some(QuickRow::SCREEN_GB[0]),
            QuickRow::ScreenGba => Some(QuickRow::SCREEN_GBA[0]),
            QuickRow::ScreenNes => Some(QuickRow::SCREEN_NES[0]),
            QuickRow::ScreenSnes => Some(QuickRow::SCREEN_SNES[0]),
            QuickRow::ScreenMd => Some(QuickRow::SCREEN_MD[0]),
            QuickRow::ScreenGg => Some(QuickRow::SCREEN_GG[0]),
            QuickRow::ScreenSms => Some(QuickRow::SCREEN_SMS[0]),
            QuickRow::ScreenN64 => Some(QuickRow::SCREEN_N64[0]),
            QuickRow::ScreenNgp => Some(QuickRow::SCREEN_NGP[0]),
            QuickRow::ScreenS32x => Some(QuickRow::SCREEN_32X[0]),
            QuickRow::ScreenSegaCd => Some(QuickRow::SCREEN_SEGACD[0]),
            QuickRow::ScreenPs1 => Some(QuickRow::SCREEN_PS1[0]),
            QuickRow::Game => Some(QuickRow::GAME[0]),
            QuickRow::System => Some(QuickRow::SYSTEM[0]),
            QuickRow::Library => Some(QuickRow::LIBRARY[0]),
            _ => None,
        }
    }

    pub fn position(self) -> usize {
        self.page().iter().position(|r| *r == self).unwrap_or(0)
    }

    pub fn label(self) -> &'static str {
        match self {
            QuickRow::Wifi => "Wi-Fi",
            QuickRow::CartArt => "Cart Art",
            QuickRow::Achievements => "RetroAchievements",
            QuickRow::Update => "Update",
            QuickRow::System => "System",
            QuickRow::AutoClock => "Auto Clock",
            QuickRow::Lights => "LEDs",
            QuickRow::LedMode => "Lights",
            QuickRow::LedTop => "Top Bar",
            QuickRow::LedButtons => "Buttons",
            QuickRow::LedTriggers => "Triggers",
            QuickRow::LedColor => "Color",
            QuickRow::LedEffect => "Effect",
            QuickRow::LedSticks => "Sticks",
            QuickRow::Library => "Library",
            QuickRow::Controls => "Controls",
            QuickRow::F1Shelf => "F1 on Shelf",
            QuickRow::F2Shelf => "F2 on Shelf",
            QuickRow::F1Game => "F1 in Game",
            QuickRow::F2Game => "F2 in Game",
            QuickRow::LibrarySearch => "Search Games",
            QuickRow::LibrarySort => "Sort By",
            QuickRow::LibraryShow => "Show",
            QuickRow::LibraryPlayTime => "Play Time",
            QuickRow::LibraryTitleCard => "Title Card",
            QuickRow::FastForward => "Fast Forward",
            QuickRow::FastForwardSound => "Fast Forward Sound",
            QuickRow::Screen => "Screen",
            QuickRow::Rumble => "Rumble",
            QuickRow::DateTime => "Date & Time",
            QuickRow::About => "About",
            QuickRow::GbaShader
            | QuickRow::GbShader
            | QuickRow::NesShader
            | QuickRow::SnesShader
            | QuickRow::MdShader
            | QuickRow::GgShader
            | QuickRow::SmsShader
            | QuickRow::N64Shader
            | QuickRow::NgpShader
            | QuickRow::S32xShader
            | QuickRow::SegaCdShader
            | QuickRow::Ps1Shader => "Shader",
            QuickRow::GbColourCorrection | QuickRow::GbaColourCorrection => "Color Correction",
            QuickRow::GbOverlay => "GB Overlay",
            QuickRow::GbcOverlay => "GBC Overlay",
            QuickRow::GbaOverlay
            | QuickRow::NesOverlay
            | QuickRow::SnesOverlay
            | QuickRow::MdOverlay
            | QuickRow::GgOverlay
            | QuickRow::SmsOverlay
            | QuickRow::N64Overlay
            | QuickRow::NgpOverlay
            | QuickRow::S32xOverlay
            | QuickRow::SegaCdOverlay
            | QuickRow::Ps1Overlay => "Overlay",
            QuickRow::Ps1Dithering => "Dithering",
            QuickRow::ScreenGb => "Game Boy / GBC",
            QuickRow::ScreenGba => "Game Boy Advance",
            QuickRow::ScreenNes => "NES",
            QuickRow::ScreenSnes => "SNES",
            QuickRow::ScreenMd => "Mega Drive",
            QuickRow::ScreenGg => "Game Gear",
            QuickRow::ScreenSms => "Master System",
            QuickRow::ScreenN64 => "Nintendo 64",
            QuickRow::ScreenNgp => "Neo Geo Pocket",
            QuickRow::ScreenS32x => "Sega 32X",
            QuickRow::ScreenSegaCd => "Sega CD",
            QuickRow::ScreenPs1 => "PlayStation",
            QuickRow::Game => "Gameplay",
            QuickRow::EjectSave => "Auto Save on Eject",
            QuickRow::AutoSave => "Auto Save",
            QuickRow::BatteryShutdown => "Low Battery Shutdown",
            QuickRow::AutoSleep => "Auto Sleep",
            QuickRow::Turbo => "Turbo Buttons",
            QuickRow::Rewind => "Rewind",
            QuickRow::GbPalettes => "GB Palettes",
        }
    }

    pub fn opens(self) -> bool {
        matches!(
            self,
            QuickRow::Controls
                | QuickRow::LibrarySearch
                | QuickRow::DateTime
                | QuickRow::About
                | QuickRow::Screen
                | QuickRow::Game
                | QuickRow::Lights
                | QuickRow::Wifi
                | QuickRow::CartArt
                | QuickRow::Achievements
                | QuickRow::Update
                | QuickRow::System
                | QuickRow::Library
                | QuickRow::ScreenGb
                | QuickRow::ScreenGba
                | QuickRow::ScreenNes
                | QuickRow::ScreenSnes
                | QuickRow::ScreenMd
                | QuickRow::ScreenGg
                | QuickRow::ScreenSms
                | QuickRow::ScreenN64
                | QuickRow::ScreenNgp
                | QuickRow::ScreenS32x
                | QuickRow::ScreenSegaCd
                | QuickRow::ScreenPs1
        )
    }

    pub fn up(self) -> QuickRow {
        let page = self.page();
        page[(self.position() + page.len() - 1) % page.len()]
    }

    pub fn down(self) -> QuickRow {
        let page = self.page();
        page[(self.position() + 1) % page.len()]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickValue {
    Speed2,
    Speed3,
    Speed4,
    Speed6,
    On,
    Off,
    Lcd3x,
    Grid,
    Dot,
    Simpletex,
    Stock,
    Low,
    Medium,
    High,
    White,
    Blue,
    Green,
    Amber,
    Red,
    Purple,
    Steady,
    Breathe,
    Unavailable,
    Min1,
    Min5,
    Min10,
    Percent3,
    Percent5,
    Percent10,
    SortAz,
    SortRecent,
    SortMost,
    AllGames,
    Favorites,
    Minutes,
    Hours,
    Min2,
    Min15,
    Min30,
    KeySearch,
    KeyFavorite,
    KeyNextSystem,
    KeyPreviousSystem,
    KeySort,
    KeyMenu,
    KeySaveState,
    KeyLoadState,
    KeyPicture,
    KeyMute,
    KeyOverlayNext,

}

impl QuickValue {
    pub const ALL: [QuickValue; 50] = [
        QuickValue::Speed2,
        QuickValue::Speed3,
        QuickValue::Speed4,
        QuickValue::Speed6,
        QuickValue::On,
        QuickValue::Off,
        QuickValue::Lcd3x,
        QuickValue::Grid,
        QuickValue::Dot,
        QuickValue::Simpletex,
        QuickValue::Stock,
        QuickValue::Low,
        QuickValue::Medium,
        QuickValue::High,
        QuickValue::White,
        QuickValue::Blue,
        QuickValue::Green,
        QuickValue::Amber,
        QuickValue::Red,
        QuickValue::Purple,
        QuickValue::Steady,
        QuickValue::Breathe,
        QuickValue::Unavailable,
        QuickValue::Min1,
        QuickValue::Min5,
        QuickValue::Min10,
        QuickValue::Percent3,
        QuickValue::Percent5,
        QuickValue::Percent10,
        QuickValue::SortAz,
        QuickValue::SortRecent,
        QuickValue::SortMost,
        QuickValue::AllGames,
        QuickValue::Favorites,
        QuickValue::Minutes,
        QuickValue::Hours,
        QuickValue::Min2,
        QuickValue::Min15,
        QuickValue::Min30,
        QuickValue::KeySearch,
        QuickValue::KeyFavorite,
        QuickValue::KeyNextSystem,
        QuickValue::KeyPreviousSystem,
        QuickValue::KeySort,
        QuickValue::KeyMenu,
        QuickValue::KeySaveState,
        QuickValue::KeyLoadState,
        QuickValue::KeyPicture,
        QuickValue::KeyMute,
        QuickValue::KeyOverlayNext,

    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn text(self) -> &'static str {
        match self {
            QuickValue::Speed2 => "2×",
            QuickValue::Speed3 => "3×",
            QuickValue::Speed4 => "4×",
            QuickValue::Speed6 => "6×",
            QuickValue::On => "On",
            QuickValue::Off => "Off",
            QuickValue::Lcd3x => "LCD3x",
            QuickValue::Grid => "Grid",
            QuickValue::Dot => "Dot",
            QuickValue::Simpletex => "Simpletex",
            QuickValue::Stock => "Stock",
            QuickValue::Low => "Low",
            QuickValue::Medium => "Medium",
            QuickValue::High => "High",
            QuickValue::White => "White",
            QuickValue::Blue => "Blue",
            QuickValue::Green => "Green",
            QuickValue::Amber => "Amber",
            QuickValue::Red => "Red",
            QuickValue::Purple => "Purple",
            QuickValue::Steady => "Steady",
            QuickValue::Breathe => "Breathe",
            QuickValue::Unavailable => "Unavailable",
            QuickValue::Min1 => "1 min",
            QuickValue::Min5 => "5 min",
            QuickValue::Min10 => "10 min",
            QuickValue::Percent3 => "3%",
            QuickValue::Percent5 => "5%",
            QuickValue::Percent10 => "10%",
            QuickValue::SortAz => "A–Z",
            QuickValue::SortRecent => "Last Played",
            QuickValue::SortMost => "Most Played",
            QuickValue::AllGames => "All Games",
            QuickValue::Favorites => "Favorites",
            QuickValue::Minutes => "Minutes",
            QuickValue::Hours => "Hours",
            QuickValue::Min2 => "2 min",
            QuickValue::Min15 => "15 min",
            QuickValue::Min30 => "30 min",
            QuickValue::KeySearch => "Search Games",
            QuickValue::KeyFavorite => "Favorite",
            QuickValue::KeyNextSystem => "Next System",
            QuickValue::KeyPreviousSystem => "Previous System",
            QuickValue::KeySort => "Change Sort",
            QuickValue::KeyMenu => "Open Menu",
            QuickValue::KeySaveState => "Save State",
            QuickValue::KeyLoadState => "Load State",
            QuickValue::KeyPicture => "Picture Size",
            QuickValue::KeyMute => "Mute",
            QuickValue::KeyOverlayNext => "Next Overlay",

        }
    }

    pub fn shortcut(choice: slot_store::controls::Shortcut) -> Self {
        use slot_store::controls::Shortcut;
        match choice {
            Shortcut::Off => Self::Off,
            Shortcut::Search => Self::KeySearch,
            Shortcut::Favorite => Self::KeyFavorite,
            Shortcut::NextSystem => Self::KeyNextSystem,
            Shortcut::PreviousSystem => Self::KeyPreviousSystem,
            Shortcut::Sort => Self::KeySort,
            Shortcut::Menu => Self::KeyMenu,
            Shortcut::SaveState => Self::KeySaveState,
            Shortcut::LoadState => Self::KeyLoadState,
            Shortcut::Picture => Self::KeyPicture,
            Shortcut::Mute => Self::KeyMute,
            Shortcut::OverlayNext => Self::KeyOverlayNext,
        }
    }

    pub fn speed(frames: u8) -> Option<QuickValue> {
        match frames {
            2 => Some(QuickValue::Speed2),
            3 => Some(QuickValue::Speed3),
            4 => Some(QuickValue::Speed4),
            6 => Some(QuickValue::Speed6),
            _ => None,
        }
    }

    /// A time in minutes (an autosave interval, or how long before the Brick sleeps), 0 for off.
    pub fn minutes(min: u8) -> Option<QuickValue> {
        match min {
            0 => Some(QuickValue::Off),
            1 => Some(QuickValue::Min1),
            2 => Some(QuickValue::Min2),
            5 => Some(QuickValue::Min5),
            10 => Some(QuickValue::Min10),
            15 => Some(QuickValue::Min15),
            30 => Some(QuickValue::Min30),
            _ => None,
        }
    }

    /// A battery percent, 0 for off.
    pub fn percent(pct: u8) -> Option<QuickValue> {
        match pct {
            0 => Some(QuickValue::Off),
            3 => Some(QuickValue::Percent3),
            5 => Some(QuickValue::Percent5),
            10 => Some(QuickValue::Percent10),
            _ => None,
        }
    }

    pub fn flag(on: bool) -> QuickValue {
        if on {
            QuickValue::On
        } else {
            QuickValue::Off
        }
    }
}

pub const QUICK_PITCH: f32 = 52.0;
pub const QUICK_TOP: f32 = (OUT_H as f32 - quick_pitch(QuickRow::MAIN.len()) * QuickRow::MAIN.len() as f32) / 2.0;

/// Row spacing for a page of `rows`: QUICK_PITCH, or tighter for a page that would
/// otherwise run into the legend (the Brick's main menu).
pub const fn quick_pitch(rows: usize) -> f32 {
    let fit = ((QUICK_SPLIT - OUT_H as f32 / 2.0) * 2.0 / rows as f32) as u32 as f32;
    if fit < QUICK_PITCH { fit } else { QUICK_PITCH }
}
pub const QUICK_EDGE: f32 = 32.0;
const BAR_INSET: f32 = 4.0;
const TYPE_DROP: f32 = 4.0;
const CARET_GAP: f32 = 14.0;
const CARET_PX: f32 = 24.0;
pub const QUICK_SPLIT: f32 = LEGEND_Y - 2.0;
const DIM_INK: [u8; 3] = [0x9a, 0x9a, 0xa4];

pub fn quick_label_face(row: QuickRow) -> UndoFace {
    quick_text_face(row.label(), MENU_INK)
}

pub fn quick_value_face(text: &str, lit: bool) -> UndoFace {
    quick_text_face(text, if lit { MENU_INK } else { DIM_INK })
}

fn quick_text_face(label: &str, colour: [u8; 3]) -> UndoFace {
    let Some(font) = text::label_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let layout = text::fit(font, label, OUT_W as f32, 1, MENU_PX, MENU_PX);
    let set = layout
        .lines
        .iter()
        .map(|l| text::line_width(font, l, layout.px, layout.tracking))
        .fold(0.0, f32::max);
    let w = set.ceil() as u32 + 2 * MENU_PAD;
    let mut rgba = vec![0u8; (w * MENU_H * 4) as usize];
    text::draw_centred(&mut rgba, w, MENU_H, &layout, colour);
    UndoFace { rgba, w, h: MENU_H }
}

/// The least space kept between a row's label and its value.
const VALUE_GAP: f32 = 28.0;

/// The widest a value's face may be on a row whose label face is `label_w` wide, beside the
/// carets (their faces' widths, left and right) when the row is the one lit, so it never runs
/// into the label. Laid out as QuickMenu::draw lays it out.
pub fn quick_value_room(label_w: u32, carets: Option<(u32, u32)>) -> u32 {
    let pad = MENU_PAD as f32;
    let right = OUT_W as f32 - QUICK_EDGE;
    let label_end = QUICK_EDGE + label_w as f32 - 2.0 * pad;
    let room = match carets {
        None => right + 2.0 * pad - label_end - VALUE_GAP,
        Some((left, right_caret)) => {
            right - right_caret as f32 - 2.0 * CARET_GAP + 2.0 * pad - left as f32 - label_end - VALUE_GAP
        }
    };
    room.max(0.0) as u32
}

pub fn quick_caret_face(right: bool) -> UndoFace {
    let glyph = if right { '\u{f0da}' } else { '\u{f0d9}' };
    let (Some(symbols), Some(label)) = (crate::icon::symbols_font(), text::label_font()) else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let (m, cov) = symbols.rasterize(glyph, CARET_PX);
    let (w, h) = (m.width as u32, MENU_H);
    let centre = match label.horizontal_line_metrics(MENU_PX) {
        Some(v) => {
            (h as f32 - v.new_line_size) / 2.0 + v.ascent
                - label.metrics('H', MENU_PX).height as f32 / 2.0
        }
        None => h as f32 / 2.0,
    };
    let top = (centre - m.height as f32 / 2.0).round() as i32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for gy in 0..m.height {
        let y = top + gy as i32;
        if y < 0 || y >= h as i32 {
            continue;
        }
        for gx in 0..m.width {
            let at = ((y as u32 * w + gx as u32) * 4) as usize;
            let a = cov[gy * m.width + gx];
            rgba[at..at + 4].copy_from_slice(&[MENU_INK[0], MENU_INK[1], MENU_INK[2], a]);
        }
    }
    UndoFace { rgba, w, h }
}

pub fn quick_legend_faces() -> [UndoFace; 3] {
    [
        hint_face("B", "Back"),
        arrows_hint_face("Change"),
        hint_face("A", "Open"),
    ]
}

pub struct QuickMenuFaces {
    pub labels: Vec<(TexId, u32, u32)>,
    pub values: Vec<[(TexId, u32, u32); 2]>,
    pub carets: [(TexId, u32, u32); 2],
    pub legend: [(TexId, u32); 3],
}

pub struct QuickMenu<'a> {
    pub row: QuickRow,
    pub values: [Option<QuickValue>; QuickRow::ALL.len()],
    pub clock: Option<[(TexId, u32, u32); 2]>,
    /// Rows whose value is free text rather than one of the QuickValues (an overlay's name):
    /// its faces, dim and lit.
    pub texts: [Option<[(TexId, u32, u32); 2]>; QuickRow::ALL.len()],
    pub faces: Option<&'a QuickMenuFaces>,
    /// Rows shown greyed out that do nothing, such as Cart Art and RetroAchievements with Wi-Fi off.
    pub disabled: [bool; QuickRow::ALL.len()],
}

impl QuickMenu<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w: OUT_W as f32,
            h: OUT_H as f32,
            colour: opening(),
        });
        out.push(Draw::Rect {
            x: 0.0,
            y: row_top(self.row, self.row) + BAR_INSET,
            w: OUT_W as f32,
            h: quick_pitch(visible_rows(self.row).len()) - 2.0 * BAR_INSET,
            colour: edge(),
        });
        let Some(faces) = self.faces else {
            return;
        };
        let (right, pad) = (OUT_W as f32 - QUICK_EDGE, MENU_PAD as f32);
        for &row in &self.row.page()[visible_rows(self.row)] {
            let y = row_top(row, self.row) + TYPE_DROP;
            let lit = row == self.row;
            if let Some(&(tex, w, h)) = faces.labels.get(row.index()) {
                let alpha = if self.disabled[row.index()] { DISABLED_ALPHA } else { 1.0 };
                push_alpha(out, tex, QUICK_EDGE - pad, y, w, h, alpha);
            }
            let value = match row {
                QuickRow::DateTime => self.clock.map(|c| c[lit as usize]),
                _ if self.texts[row.index()].is_some() => self.texts[row.index()].map(|t| t[lit as usize]),
                _ => self.values[row.index()]
                    .and_then(|v| faces.values.get(v.index()))
                    .map(|v| v[lit as usize]),
            };
            let Some((tex, w, h)) = value else {
                continue;
            };
            if !lit || row.opens() {
                push(out, tex, right + pad - w as f32, y, w, h);
                continue;
            }
            let [(left_tex, lw, lh), (right_tex, rw, rh)] = faces.carets;
            let rx = right - rw as f32;
            push(out, right_tex, rx, y, rw, rh);
            let vx = rx - CARET_GAP + pad - w as f32;
            push(out, tex, vx, y, w, h);
            push(out, left_tex, vx + pad - CARET_GAP - lw as f32, y, lw, lh);
        }
        let visible = visible_rows(self.row);
        if visible.len() < self.row.page().len() {
            let height = quick_pitch(visible.len()) * visible.len() as f32;
            let top = (OUT_H as f32 - height) / 2.0;
            let unit = height / self.row.page().len() as f32;
            out.push(Draw::Rect {
                x: OUT_W as f32 - 12.0, y: top, w: 2.0, h: height,
                colour: [0.35, 0.35, 0.38, 0.45],
            });
            out.push(Draw::Rect {
                x: OUT_W as f32 - 12.0, y: top + unit * visible.start as f32,
                w: 2.0, h: unit * visible.len() as f32,
                colour: [0.85, 0.85, 0.88, 0.8],
            });
        }
        let [back, change, open] = faces.legend;
        let other = if self.row.opens() { open } else { change };
        let hints: &[(TexId, u32)] = if self.disabled[self.row.index()] { &[back] } else { &[back, other] };
        for (tex, w, x) in centred_hints(hints, LEGEND_GAP) {
            push(out, tex, x, LEGEND_Y, w, HINT_H);
        }
    }
}

/// Keep six full-size system rows on screen, with the selection near the middle
/// while scrolling. Short submenus keep their existing centred layout.
fn visible_rows(selected: QuickRow) -> std::ops::Range<usize> {
    let page = selected.page();
    let count = if page == QuickRow::SCREEN { page.len().min(6) } else { page.len() };
    let first = selected.position().saturating_sub(count / 2).min(page.len() - count);
    first..first + count
}

fn row_top(row: QuickRow, selected: QuickRow) -> f32 {
    let visible = visible_rows(selected);
    let pitch = quick_pitch(visible.len());
    let top = (OUT_H as f32 - pitch * visible.len() as f32) / 2.0;
    top + pitch * (row.position() - visible.start) as f32
}

#[cfg(test)]
mod scrolling_tests {
    use super::*;

    #[test]
    fn every_system_can_be_selected_without_overlapping_the_fixed_legend() {
        for &selected in &QuickRow::SCREEN {
            let visible = visible_rows(selected);
            assert_eq!(visible.len(), 6);
            assert!(visible.contains(&selected.position()));
            for &row in &QuickRow::SCREEN[visible] {
                let top = row_top(row, selected);
                assert!(top >= 0.0);
                assert!(top + TYPE_DROP + (MENU_H as f32) < LEGEND_Y);
            }
        }
        assert_eq!(visible_rows(QuickRow::SCREEN[0]).start, 0);
        assert_eq!(visible_rows(*QuickRow::SCREEN.last().unwrap()).end, QuickRow::SCREEN.len());
    }
}

const DISABLED_ALPHA: f32 = 0.35;

fn push(out: &mut Vec<Draw>, tex: TexId, x: f32, y: f32, w: u32, h: u32) {
    push_alpha(out, tex, x, y, w, h, 1.0);
}

fn push_alpha(out: &mut Vec<Draw>, tex: TexId, x: f32, y: f32, w: u32, h: u32, alpha: f32) {
    out.push(Draw::Tex {
        x: x.round(),
        y: y.round(),
        w: w as f32,
        h: h as f32,
        tex,
        alpha,
    });
}
