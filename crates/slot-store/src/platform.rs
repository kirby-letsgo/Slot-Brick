use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Platform {
    #[default]
    Gba,
    Gb,
    Gbc,
    Nes,
    Snes,
    Md,
    Ps1,
    Gg,
    Sms,
    N64,
    Ngp,
}

impl Platform {
    pub const ALL: [Platform; 11] = [
        Platform::Gba,
        Platform::Gb,
        Platform::Gbc,
        Platform::Nes,
        Platform::Snes,
        Platform::Md,
        Platform::Ps1,
        Platform::Gg,
        Platform::Sms,
        Platform::N64,
        Platform::Ngp,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Platform::Gba => "Game Boy Advance",
            Platform::Gb => "Game Boy",
            Platform::Gbc => "Game Boy Color",
            Platform::Nes => "Nintendo Entertainment System",
            Platform::Snes => "Super Nintendo Entertainment System",
            Platform::Md => "Sega Mega Drive",
            Platform::Ps1 => "PlayStation",
            Platform::Gg => "Sega Game Gear",
            Platform::Sms => "Sega Master System",
            Platform::N64 => "Nintendo 64",
            Platform::Ngp => "Neo Geo Pocket",
        }
    }

    /// Where the platform is in Platform::ALL, for arrays kept one per platform.
    pub fn index(self) -> usize {
        match self {
            Platform::Gba => 0,
            Platform::Gb => 1,
            Platform::Gbc => 2,
            Platform::Nes => 3,
            Platform::Snes => 4,
            Platform::Md => 5,
            Platform::Ps1 => 6,
            Platform::Gg => 7,
            Platform::Sms => 8,
            Platform::N64 => 9,
            Platform::Ngp => 10,
        }
    }

    /// Systems older Slots kept choices for under the bare name; the Mega Drive came after
    /// keys were per system.
    pub fn reads_bare_keys(self) -> bool {
        !matches!(
            self,
            Platform::Md | Platform::Gg | Platform::Sms | Platform::N64 | Platform::Ngp
        )
    }

    /// A per-game setting's key: the game's name under its system, so a Mega Drive and a SNES
    /// game of one name keep their own. Older Slots wrote the bare name.
    pub fn game_key(self, stem: &str) -> String {
        format!("{}/{}", self.dir_name(), stem)
    }

    pub fn dir_name(self) -> &'static str {
        match self {
            Platform::Gba => "GBA",
            Platform::Gb => "GB",
            Platform::Gbc => "GBC",
            Platform::Nes => "NES",
            Platform::Snes => "SNES",
            Platform::Md => "MD",
            Platform::Ps1 => "PS1",
            Platform::Gg => "GG",
            Platform::Sms => "SMS",
            Platform::N64 => "N64",
            Platform::Ngp => "NGP",
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Platform::Gba => &["gba"],
            Platform::Gb | Platform::Gbc => &["gb", "gbc"],
            Platform::Nes => &["nes"],
            Platform::Snes => &["sfc", "smc"],
            Platform::Md => &["md", "gen", "smd", "bin"],
            Platform::Gg => &["gg"],
            Platform::Sms => &["sms"],
            Platform::N64 => &["z64", "v64", "n64"],
            Platform::Ngp => &["ngp", "ngc"],
            // A .bin counts only when no .cue names it (see `ps1::games`).
            Platform::Ps1 => &["cue", "chd", "pbp", "m3u", "bin", "iso", "exe"],
        }
    }

    pub fn picture(self) -> (u32, u32) {
        match self {
            Platform::Gba => (240, 160),
            Platform::Gb | Platform::Gbc | Platform::Gg => (160, 144),
            // With FCEUmm's and snes9x's 8-line overscan crop top and bottom.
            Platform::Nes | Platform::Snes => (256, 224),
            // 320x224, as most games draw; a 256-wide picture is widened to the full width as it
            // arrives, and PAL's 240 lines fill the canvas.
            Platform::Md => (320, 224),
            // Whatever size it draws at (256 to 640 wide, 240 or 480 tall), a PlayStation
            // picture fills a 4:3 screen: each frame is stretched over the whole canvas.
            Platform::Ps1 => (640, 480),
            // Fit all SMS video modes to the console's 4:3 television picture.
            Platform::Sms => (256, 192),
            Platform::N64 => (320, 240),
            Platform::Ngp => (160, 152),
        }
    }

    /// The frame canvas a core draws into, its picture centred: GBA-sized for the Game
    /// Boys, the full 256x240 (overscan included) for the NES and SNES. A SNES hi-res frame
    /// (512 wide, or 448 lines interlaced) is halved into it as it arrives. A Mega Drive picture
    /// 256 wide is widened to the full width as it arrives.
    pub fn canvas(self) -> (u32, u32) {
        match self {
            Platform::Gba | Platform::Gb | Platform::Gbc | Platform::Gg => (240, 160),
            Platform::Nes | Platform::Snes => (256, 240),
            Platform::Md => (320, 240),
            Platform::Ps1 => (640, 480),
            // Fit all SMS video modes to the console's 4:3 television picture.
            Platform::Sms => (256, 192),
            Platform::N64 => (320, 240),
            Platform::Ngp => (160, 152),
        }
    }

    pub fn accepts(self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                self.extensions()
                    .iter()
                    .any(|k| ext.eq_ignore_ascii_case(k))
            })
    }

    /// Whether the shelf takes ROMs zipped or 7z'd. A disc would take too long to unpack on the
    /// Brick, and a .chd is compressed already.
    pub fn takes_archives(self) -> bool {
        !matches!(self, Platform::Ps1)
    }
}
