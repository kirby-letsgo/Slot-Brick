//! Driver names from the pinned FBNeo Neo Geo source; filenames remain save identities.
const TITLES: &str = include_str!("neogeo-titles.tsv");
pub fn title(stem: &str) -> Option<&'static str> {
    TITLES.lines().find_map(|line| {
        let mut fields = line.split('\t');
        let name = fields.next()?;
        let title = fields.next()?;
        name.eq_ignore_ascii_case(stem).then_some(title)
    })
}
pub fn is_bios(stem: &str) -> bool {
    ["neogeo", "neocdz", "neocd", "aes"].iter().any(|name| name.eq_ignore_ascii_case(stem))
}
