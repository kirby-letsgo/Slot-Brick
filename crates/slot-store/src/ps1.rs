//! A PlayStation game is often several files: a .cue and the .bin tracks it names, or an .m3u
//! listing each disc of a game. The shelf shows each game once, by the file that holds the
//! rest; the files it holds are left off. A game may sit in a folder of its own.
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::scan::is_hidden;
use crate::Platform;

/// The games in `dir` and in the folders directly inside it: every PlayStation file that no
/// .cue or .m3u beside it names.
pub fn games(dir: &Path) -> Vec<PathBuf> {
    games_for(dir, Platform::Ps1)
}

/// Shared CUE/M3U grouping for CD systems; acceptance stays platform-specific.
pub fn games_for(dir: &Path, platform: Platform) -> Vec<PathBuf> {
    let mut files = files_in(dir, platform);
    for sub in subdirs(dir) {
        files.extend(files_in(&sub, platform));
    }
    let named: HashSet<PathBuf> = files.iter().flat_map(|f| named_by(f)).map(|p| key(&p)).collect();
    files.retain(|f| !named.contains(&key(f)));
    files
}

/// A game's discs, in order: those its .m3u lists, or the one file it is.
pub fn discs(rom: &Path) -> Vec<PathBuf> {
    let m3u = rom.extension().is_some_and(|e| e.eq_ignore_ascii_case("m3u"));
    let Some(dir) = rom.parent().filter(|_| m3u) else {
        return vec![rom.to_path_buf()];
    };
    let Some(text) = read_descriptor(rom) else {
        return vec![rom.to_path_buf()];
    };
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| dir.join(l.replace('\\', "/")))
        .collect()
}

/// The files a .cue (its tracks) or an .m3u (its discs) names, beside it.
fn named_by(file: &Path) -> Vec<PathBuf> {
    named_by_with(file, &read_descriptor)
}

// CUE/M3U files contain a small track list, never the disc data. Bound even malformed
// descriptors so a misnamed disc image cannot allocate hundreds of megabytes.
const DESCRIPTOR_MAX: u64 = 1024 * 1024;
fn read_descriptor(file: &Path) -> Option<String> {
    let mut text = String::new();
    std::fs::File::open(file).ok()?.take(DESCRIPTOR_MAX + 1).read_to_string(&mut text).ok()?;
    (text.len() as u64 <= DESCRIPTOR_MAX).then_some(text)
}

fn named_by_with(file: &Path, read: &impl Fn(&Path) -> Option<String>) -> Vec<PathBuf> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !matches!(ext.as_str(), "cue" | "m3u") { return Vec::new(); }
    let Some(dir) = file.parent() else {
        return Vec::new();
    };
    let Some(text) = read(file) else {
        return Vec::new();
    };
    let names: Vec<String> = match ext.as_str() {
        "cue" => text.lines().filter_map(cue_file).collect(),
        "m3u" => text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect(),
        _ => return Vec::new(),
    };
    let named: Vec<PathBuf> = names.iter().map(|n| dir.join(n.replace('\\', "/"))).collect();
    // A disc an .m3u names takes its tracks along with it.
    let tracks: Vec<PathBuf> = named.iter().filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("cue"))).flat_map(|p| named_by_with(p, read)).collect();
    named.into_iter().chain(tracks).collect()
}

/// `FILE "Track 01.bin" BINARY`: the file's name, quoted or not.
fn cue_file(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix("FILE")?.trim();
    match rest.strip_prefix('"') {
        Some(quoted) => quoted.split_once('"').map(|(name, _)| name.to_string()),
        None => rest.rsplit_once(' ').map(|(name, _)| name.trim().to_string()),
    }
}

/// Paths compared as the SD card's FAT compares them: without case.
fn key(p: &Path) -> PathBuf {
    PathBuf::from(p.to_string_lossy().to_lowercase())
}

fn files_in(dir: &Path, platform: Platform) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| !is_hidden(p) && p.is_file() && platform.accepts(p))
        .collect()
}

fn subdirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| !is_hidden(p) && p.is_dir()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disc_images_are_never_opened_as_text_during_a_scan() {
        for extension in ["chd", "CHD", "bin", "iso", "pbp", "exe"] {
            let file = PathBuf::from(format!("Games/PS1/Large disc.{extension}"));
            assert!(named_by_with(&file, &|path| {
                panic!("opened binary disc as text: {}", path.display())
            }).is_empty());
        }
    }

    #[test]
    fn only_playlists_and_cues_are_read_when_grouping_discs_and_tracks() {
        let opened = std::cell::RefCell::new(Vec::new());
        let named = named_by_with(Path::new("PS1/Game.M3U"), &|path| {
            opened.borrow_mut().push(path.to_path_buf());
            match path.to_str().unwrap() {
                "PS1/Game.M3U" => Some("Disc.CUE\nOther.chd\n".into()),
                "PS1/Disc.CUE" => Some("FILE \"Track.bin\" BINARY\n".into()),
                _ => panic!("unexpected disc read: {}", path.display()),
            }
        });
        assert_eq!(*opened.borrow(), [PathBuf::from("PS1/Game.M3U"), PathBuf::from("PS1/Disc.CUE")]);
        assert_eq!(named, [PathBuf::from("PS1/Disc.CUE"), PathBuf::from("PS1/Other.chd"), PathBuf::from("PS1/Track.bin")]);
    }

    #[test]
    fn oversized_or_binary_descriptors_are_ignored() {
        let d = tempfile::tempdir().unwrap();
        let cue = d.path().join("Oversized.cue");
        let file = std::fs::File::create(&cue).unwrap();
        file.set_len(DESCRIPTOR_MAX + 1).unwrap();
        assert!(read_descriptor(&cue).is_none());
        assert!(named_by(&cue).is_empty());
        std::fs::write(&cue, [255, 254, 253]).unwrap();
        assert!(read_descriptor(&cue).is_none());
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut out: Vec<String> = games(dir)
            .iter()
            .map(|p| p.strip_prefix(dir).unwrap().to_string_lossy().into_owned())
            .collect();
        out.sort();
        out
    }

    #[test]
    fn each_game_shows_once_by_the_file_that_holds_the_rest() {
        let d = tempfile::tempdir().unwrap();
        let put = |name: &str, text: &str| {
            let p = d.path().join(name);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        };
        // A .cue and its two tracks.
        put("Ridge Racer.cue", "FILE \"Ridge Racer (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\nFILE \"Ridge Racer (Track 2).BIN\" BINARY\n");
        put("Ridge Racer (Track 1).bin", "");
        put("Ridge Racer (Track 2).bin", "");
        // A two-disc game in a folder of its own: the .m3u, its .cues, their tracks.
        put("FF7/Final Fantasy VII.m3u", "# discs\nFinal Fantasy VII (Disc 1).cue\nFinal Fantasy VII (Disc 2).cue\n");
        put("FF7/Final Fantasy VII (Disc 1).cue", "FILE Final Fantasy VII (Disc 1).bin BINARY\n");
        put("FF7/Final Fantasy VII (Disc 1).bin", "");
        put("FF7/Final Fantasy VII (Disc 2).cue", "FILE \"Final Fantasy VII (Disc 2).bin\" BINARY\n");
        put("FF7/Final Fantasy VII (Disc 2).bin", "");
        // One file each: a .chd, and a .bin no .cue names.
        put("Crash Bandicoot.chd", "");
        put("Loose.bin", "");
        put("notes.txt", "");
        assert_eq!(names(d.path()), vec!["Crash Bandicoot.chd", "FF7/Final Fantasy VII.m3u", "Loose.bin", "Ridge Racer.cue"]);
        let ff7 = discs(&d.path().join("FF7/Final Fantasy VII.m3u"));
        assert_eq!(ff7.len(), 2);
        assert!(ff7[1].ends_with("FF7/Final Fantasy VII (Disc 2).cue"));
        assert_eq!(discs(&d.path().join("Crash Bandicoot.chd")).len(), 1);
    }
}
