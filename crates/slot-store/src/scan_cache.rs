//! A disposable library snapshot. Paths are relative to the card, so moving the card
//! between a computer and the handheld does not invalidate it. Never cache ROM bytes.
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use crate::{Cart, Platform, ShellChoice};

pub const FILE: &str = "System/Cache/library-v1.bin";
const MAGIC: &[u8] = b"SLOT-LIBRARY\x01";
const MAX_BYTES: u64 = 64 * 1024 * 1024;
const MAX_CARTS: usize = 100_000;

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid library index")
}

fn relative(root: &Path, path: &Path) -> io::Result<String> {
    let path = path.strip_prefix(root).map_err(|_| invalid())?;
    if !path.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(invalid());
    }
    path.to_str().map(str::to_owned).ok_or_else(invalid)
}

fn path(root: &Path, text: &str) -> Option<PathBuf> {
    let p = Path::new(text);
    if text.is_empty() || !p.components().all(|c| matches!(c, Component::Normal(_))) {
        return None;
    }
    Some(root.join(p))
}

pub fn write(root: &Path, carts: &[Cart]) -> io::Result<()> {
    // Ports live in the standard card-level PortMaster folder. Refresh their small
    // script listing on every boot, keeping this ROM cache confined to Slot.
    let carts: Vec<_> = carts.iter().filter(|c| c.platform != Platform::Ports).collect();
    if carts.len() > MAX_CARTS {
        return Err(invalid());
    }
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&(carts.len() as u32).to_le_bytes());
    for cart in carts {
        let fields = [
            cart.platform.dir_name().to_owned(),
            cart.stem.clone(),
            relative(root, &cart.rom)?,
            cart.label
                .as_ref()
                .map(|p| relative(root, p))
                .transpose()?
                .unwrap_or_default(),
            cart.famicom_label
                .as_ref()
                .map(|p| relative(root, p))
                .transpose()?
                .unwrap_or_default(),
            // ROM headers are read lazily for the selected cartridge. Persisting
            // them would keep stale identifiers after replacing a ROM in place.
            String::new(),
            if cart.code == "SLOT_SPLORE" {
                cart.code.clone()
            } else {
                String::new()
            },
            cart.shell.map(|s| s.to_value()).unwrap_or_default(),
        ];
        for field in fields {
            if field.len() > 16_384 {
                return Err(invalid());
            }
            bytes.extend_from_slice(&(field.len() as u32).to_le_bytes());
            bytes.extend_from_slice(field.as_bytes());
        }
        if bytes.len() as u64 + 4 > MAX_BYTES {
            return Err(invalid());
        }
    }
    bytes.extend_from_slice(&crc32fast::hash(&bytes).to_le_bytes());
    std::fs::create_dir_all(root.join("System/Cache"))?;
    crate::atomic_write(&root.join(FILE), &bytes)
}

/// Missing, obsolete, partial or damaged indexes all fall back to a real scan.
/// Reading this file does not stat or open any ROM or artwork.
pub fn read(root: &Path) -> Option<Vec<Cart>> {
    let mut bytes = Vec::new();
    std::fs::File::open(root.join(FILE))
        .ok()?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > MAX_BYTES || bytes.len() < MAGIC.len() + 8 {
        return None;
    }
    let (body, checksum) = bytes.split_at(bytes.len() - 4);
    if crc32fast::hash(body) != u32::from_le_bytes(checksum.try_into().ok()?) {
        return None;
    }
    let mut input = body.strip_prefix(MAGIC)?;
    let count = number(&mut input)? as usize;
    if count > MAX_CARTS || count > input.len() / 32 {
        return None;
    }
    let mut carts = Vec::with_capacity(count);
    let mut seen = std::collections::HashSet::new();
    for _ in 0..count {
        let platform = field(&mut input)?;
        let platform = Platform::ALL
            .into_iter()
            .find(|p| p.dir_name() == platform)?;
        let stem = field(&mut input)?.to_owned();
        if platform == Platform::Ports { return None; }
        if stem.is_empty() {
            return None;
        }
        let rom = path(root, field(&mut input)?)?;
        // Separate PS1 folders can contain discs with the same filename.
        if !seen.insert((platform, rom.clone())) {
            return None;
        }
        let label = field(&mut input)?;
        let label = if label.is_empty() {
            None
        } else {
            Some(path(root, label)?)
        };
        let famicom = field(&mut input)?;
        let famicom_label = if famicom.is_empty() {
            None
        } else {
            Some(path(root, famicom)?)
        };
        let title = field(&mut input)?.to_owned();
        let code = field(&mut input)?.to_owned();
        let shell = field(&mut input)?;
        let shell = if shell.is_empty() {
            None
        } else {
            Some(ShellChoice::parse(shell)?)
        };
        let splore = platform == Platform::Pico8
            && code == "SLOT_SPLORE"
            && rom == root.join("System/pico8-splore.png");
        if !splore && !rom.starts_with(root.join("Games").join(platform.dir_name())) {
            return None;
        }
        let labels = root.join("Labels").join(platform.dir_name());
        if label
            .iter()
            .chain(famicom_label.iter())
            .any(|p| !p.starts_with(&labels))
        {
            return None;
        }
        carts.push(Cart {
            platform,
            stem,
            rom,
            label,
            famicom_label,
            title,
            code,
            shell,
        });
    }
    if !input.is_empty() { return None; }
    carts.extend(crate::ports::scan(root));
    Some(carts)
}

fn number(input: &mut &[u8]) -> Option<u32> {
    let (value, rest) = input.split_at_checked(4)?;
    *input = rest;
    Some(u32::from_le_bytes(value.try_into().ok()?))
}

fn field<'a>(input: &mut &'a [u8]) -> Option<&'a str> {
    let len = number(input)? as usize;
    if len > 16_384 {
        return None;
    }
    let (value, rest) = input.split_at_checked(len)?;
    *input = rest;
    std::str::from_utf8(value).ok()
}
