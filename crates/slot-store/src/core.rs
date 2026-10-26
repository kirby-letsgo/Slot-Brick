use std::collections::HashMap;
use std::path::Path;

use crate::Platform;

pub const SELECTED_CORE_FILE: &str = "Config/selected_core.ini";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Core {
    #[default]
    Mgba,
    Gpsp,
    Fceumm,
    Snes9x,
    PcsxRearmed,
    GenesisPlusGx,
    ParallelN64,
    BeetleNgp,
    Pico8,
    PicoDrive,
    BeetlePceFast,
    FbNeo,
    BeetleWswan,
    BeetleLynx,
    PortMaster,
}

impl Core {
    pub const ALL: [Core; 15] = [
        Core::Mgba,
        Core::Gpsp,
        Core::Fceumm,
        Core::Snes9x,
        Core::PcsxRearmed,
        Core::GenesisPlusGx,
        Core::ParallelN64,
        Core::BeetleNgp,
        Core::Pico8,
        Core::PicoDrive,
        Core::BeetlePceFast,
        Core::FbNeo,
        Core::BeetleWswan,
        Core::BeetleLynx,
        Core::PortMaster,
    ];
    /// The GBA's choice of cores, the picker board's two sockets (in Core::index order).
    pub const PICKER: [Core; 2] = [Core::Mgba, Core::Gpsp];

    pub fn as_str(&self) -> &'static str {
        match self {
            Core::Mgba => "mgba",
            Core::Gpsp => "gpsp",
            Core::Fceumm => "fceumm",
            Core::Snes9x => "snes9x",
            Core::PcsxRearmed => "pcsx_rearmed",
            Core::GenesisPlusGx => "genesis_plus_gx",
            Core::ParallelN64 => "parallel_n64",
            Core::BeetleNgp => "mednafen_ngp",
            Core::BeetleWswan => "mednafen_wswan",
            Core::BeetleLynx => "mednafen_lynx",
            Core::Pico8 => "pico8",
            Core::PortMaster => "portmaster",
            Core::PicoDrive => "picodrive",
            Core::BeetlePceFast => "mednafen_pce_fast",
            Core::FbNeo => "fbneo_neogeo",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn text(self) -> &'static str {
        match self {
            Core::Mgba => "mGBA",
            Core::Gpsp => "gpSP",
            Core::Fceumm => "FCEUmm",
            Core::Snes9x => "Snes9x",
            Core::PcsxRearmed => "PCSX ReARMed",
            Core::GenesisPlusGx => "Genesis Plus GX",
            Core::ParallelN64 => "ParaLLEl N64",
            Core::BeetleNgp => "Beetle NeoPop",
            Core::BeetleWswan => "Beetle Cygne",
            Core::BeetleLynx => "Beetle Lynx",
            Core::Pico8 => "PICO-8 (official)",
            Core::PortMaster => "PortMaster (native)",
            Core::PicoDrive => "PicoDrive",
            Core::BeetlePceFast => "Beetle PCE Fast",
            Core::FbNeo => "FinalBurn Neo",
        }
    }

    pub fn parse(s: &str) -> Option<Core> {
        match s.trim().to_ascii_lowercase().as_str() {
            "mgba" => Some(Core::Mgba),
            "gpsp" => Some(Core::Gpsp),
            "fceumm" => Some(Core::Fceumm),
            "snes9x" => Some(Core::Snes9x),
            "pcsx_rearmed" => Some(Core::PcsxRearmed),
            "genesis_plus_gx" => Some(Core::GenesisPlusGx),
            "parallel_n64" => Some(Core::ParallelN64),
            "mednafen_ngp" => Some(Core::BeetleNgp),
            "mednafen_wswan" => Some(Core::BeetleWswan),
            "mednafen_lynx" => Some(Core::BeetleLynx),
            "pico8" => Some(Core::Pico8),
            "portmaster" => Some(Core::PortMaster),
            "picodrive" => Some(Core::PicoDrive),
            "mednafen_pce_fast" => Some(Core::BeetlePceFast),
            "fbneo_neogeo" => Some(Core::FbNeo),
            _ => None,
        }
    }

    pub fn runs(self, platform: Platform) -> bool {
        match self {
            Core::Mgba => matches!(platform, Platform::Gba | Platform::Gb | Platform::Gbc),
            Core::Gpsp => platform == Platform::Gba,
            Core::Fceumm => platform == Platform::Nes,
            Core::Snes9x => platform == Platform::Snes,
            Core::PcsxRearmed => platform == Platform::Ps1,
            Core::GenesisPlusGx => matches!(platform, Platform::Md | Platform::Gg | Platform::Sms),
            Core::ParallelN64 => platform == Platform::N64,
            Core::BeetleNgp => platform == Platform::Ngp,
            Core::BeetleWswan => platform == Platform::Ws,
            Core::BeetleLynx => platform == Platform::Lynx,
            Core::Pico8 => platform == Platform::Pico8,
            Core::PortMaster => platform == Platform::Ports,
            Core::PicoDrive => matches!(platform, Platform::S32x | Platform::SegaCd),
            Core::FbNeo => platform == Platform::NeoGeo,
            Core::BeetlePceFast => matches!(platform, Platform::Pce | Platform::PceCd),
        }
    }

    pub fn default_for(platform: Platform) -> Core {
        match platform {
            Platform::Gba | Platform::Gb | Platform::Gbc => Core::Mgba,
            Platform::Nes => Core::Fceumm,
            Platform::Snes => Core::Snes9x,
            Platform::Md | Platform::Gg | Platform::Sms => Core::GenesisPlusGx,
            Platform::Ps1 => Core::PcsxRearmed,
            Platform::N64 => Core::ParallelN64,
            Platform::Ngp => Core::BeetleNgp,
            Platform::Ws => Core::BeetleWswan,
            Platform::Lynx => Core::BeetleLynx,
            Platform::Pico8 => Core::Pico8,
            Platform::Ports => Core::PortMaster,
            Platform::S32x | Platform::SegaCd => Core::PicoDrive,
            Platform::Pce | Platform::PceCd => Core::BeetlePceFast,
            Platform::NeoGeo => Core::FbNeo,
        }
    }
}

pub fn read_selected_cores(root: &Path) -> HashMap<String, Core> {
    crate::ini::read(root, SELECTED_CORE_FILE)
        .into_iter()
        .filter_map(|(stem, name)| Core::parse(&name).map(|core| (stem, core)))
        .collect()
}

pub fn core_for(root: &Path, stem: &str) -> Core {
    crate::ini::value(root, SELECTED_CORE_FILE, stem)
        .as_deref()
        .and_then(Core::parse)
        .unwrap_or_default()
}

pub fn core_for_platform(root: &Path, stem: &str, platform: Platform) -> Core {
    crate::ini::value(root, SELECTED_CORE_FILE, stem)
        .as_deref()
        .and_then(Core::parse)
        .filter(|core| core.runs(platform))
        .unwrap_or_else(|| Core::default_for(platform))
}

pub fn write_selected_core(root: &Path, stem: &str, core: Core) -> std::io::Result<()> {
    crate::ini::write(root, SELECTED_CORE_FILE, stem, core.as_str())
}
