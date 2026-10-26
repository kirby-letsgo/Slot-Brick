#!/usr/bin/env python3
"""Regenerate card/Slot/System/{nes,famicom,snes,snes-pal,md,md-jp,gg,sms,n64,ngp}-cart-labels.txt and ps1-disc-labels.txt from the LaunchBox Games Database.

The slot art set behind Cart Studio has no NES or SNES label scans, so Cart Art takes those
labels from LaunchBox's "Cart - Front" pictures: whole-cart photos it crops the label out of
(cart_art.rs). This keeps the ones of the console's standard US cart (PNG cut-outs, from the
regions that cart was sold in; Famicom, Super Famicom and PAL SNES carts are other shapes)
with the game's name and region, so the Brick never downloads the 100 MB metadata itself.
"famicom" keeps the NES games' Japanese pictures instead: the Famicom cart; "snes-pal" the SNES
games' Japanese and European ones: the rounder Super Famicom / PAL cart. "md" keeps the Mega
Drive games' Western pictures (the US, European and Brazilian cart, one shape); "md-jp" the
Japanese ones, with Korea's and the rest of Asia's, which came on the same cart. "ps1" keeps the
PlayStation games' "Disc" pictures, from every region: the disc itself, printed side up.
"gg" keeps Game Gear cart fronts from all supported regions; their shell has one shape.
"sms" keeps Master System and Japanese Mark III fronts; the device recognises both shapes.
"n64" keeps Nintendo 64 fronts; "ngp" combines Neo Geo Pocket and Pocket Color fronts.

The download is untrusted input: it is parsed with the standard library's expat (2.4+
guards against entity expansion), and only names without control characters and file names
of plain characters make it into the list, which the Brick checks again before any request.
An alternate name is left out when it is another game's own name (Super Robot Wars is also
"Target Earth") and the game has no cart picture from that name's region, as it would list
this game's pictures under that one; or when it is in another script and reduces to almost
nothing ("2", "ii"), as no ROM is named that. One that reduces to nothing at all is kept: the
Brick matches it only by the whole name. One with no region is kept unless the other game has
pictures of its own: a picture-less namesake is mostly a duplicate entry or a near miss
("Dragon Quest I・II" reduces as "Dragon Quest III" does).

Usage: python3 brick/make-cart-labels.py nes|famicom|snes|snes-pal|md|md-jp|gg|sms|n64|ngp|ps1 [Metadata.zip]
"""
import pathlib, re, sys, tempfile, unicodedata, urllib.request, zipfile
import xml.etree.ElementTree as ET

URL = 'https://gamesdb.launchbox-app.com/Metadata.zip'
# LaunchBox regions as the No-Intro tags slot's name matching scores. The NES cart was the
# same in the US and Europe; the SNES cart of Europe (and Japan) is another shape.
AMERICAS = {'North America': 'USA', 'United States': 'USA', 'Canada': 'Canada', 'World': 'World', '': ''}
EUROPE = {'Europe': 'Europe', 'United Kingdom': 'UK', 'Germany': 'Germany', 'France': 'France',
          'Spain': 'Spain', 'Italy': 'Italy', 'Australia': 'Australia', 'Sweden': 'Sweden',
          'Netherlands': 'Netherlands'}
# The Japanese Mega Drive cart was Korea's and the rest of Asia's too.
EAST = {'Japan': 'Japan', 'Korea': 'Korea', 'Asia': 'Asia', 'Taiwan': 'Taiwan', 'China': 'China', 'Hong Kong': 'Hong Kong'}
CONSOLES = {
    'n64': ('Nintendo 64', {**AMERICAS, **EUROPE, **EAST}, 'Cart - Front', 'cart'),
    'ngp': (('SNK Neo Geo Pocket', 'SNK Neo Geo Pocket Color'), {**AMERICAS, **EUROPE, **EAST}, 'Cart - Front', 'cart'),
    'nes': ('Nintendo Entertainment System', {**AMERICAS, **EUROPE}, 'Cart - Front', 'cart'),
    'famicom': ('Nintendo Entertainment System', {'Japan': 'Japan'}, 'Cart - Front', 'cart'),
    'snes': ('Super Nintendo Entertainment System', AMERICAS, 'Cart - Front', 'cart'),
    'snes-pal': ('Super Nintendo Entertainment System', {'Japan': 'Japan', **EUROPE}, 'Cart - Front', 'cart'),
    # Tec Toy's Brazilian carts were the Western shape too.
    'md': ('Sega Genesis', {**AMERICAS, **EUROPE, 'Brazil': 'Brazil', 'Oceania': 'Australia'}, 'Cart - Front', 'cart'),
    'md-jp': ('Sega Genesis', EAST, 'Cart - Front', 'cart'),
    'gg': ('Sega Game Gear', {**AMERICAS, **EUROPE, **EAST, 'Brazil': 'Brazil', 'Oceania': 'Australia'}, 'Cart - Front', 'cart'),
    'sms': ('Sega Master System', {**AMERICAS, **EUROPE, **EAST, 'Brazil': 'Brazil', 'Oceania': 'Australia'}, 'Cart - Front', 'cart'),
    'ps1': ('Sony Playstation', {'Japan': 'Japan', **AMERICAS, **EUROPE}, 'Disc', 'disc'),
}
if len(sys.argv) < 2 or sys.argv[1] not in CONSOLES:
    sys.exit(__doc__)
PLATFORM, REGIONS, IMAGE, KIND = CONSOLES[sys.argv[1]]
OUT = pathlib.Path(__file__).parent / f'card/Slot/System/{sys.argv[1]}-{KIND}-labels.txt'
SAFE = set('abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.')


def metadata():
    if len(sys.argv) > 2:
        return zipfile.ZipFile(sys.argv[2])
    tmp = tempfile.NamedTemporaryFile(suffix='.zip')
    request = urllib.request.Request(URL, headers={'User-Agent': 'slot-brick make-cart-labels'})
    with urllib.request.urlopen(request, timeout=600) as r:
        while chunk := r.read(1 << 20):
            tmp.write(chunk)
    tmp.flush()
    return zipfile.ZipFile(tmp.name)


def title_key(name):
    """A name as the Brick compares names (title_key in crates/slot-store/src/thumbs.rs): no
    tags in brackets, accents, apostrophes, punctuation, case or "the"; "&" is "and"."""
    plain, depth = [], 0
    for c in unicodedata.normalize('NFD', name):
        if c in '([':
            depth += 1
        elif c in ')]':
            depth = max(0, depth - 1)
        elif c == "'":
            pass
        elif depth == 0:
            plain.append(' and ' if c == '&' else c)
    words = re.split('[^A-Za-z0-9]', ''.join(plain))
    return ''.join(w.lower() for w in words if w and w.lower() != 'the')


games, names, carts = {}, {}, {}
with metadata().open('Metadata.xml') as f:
    for _, el in ET.iterparse(f):
        if el.tag == 'Game':
            if el.findtext('Platform') in (PLATFORM if isinstance(PLATFORM, tuple) else (PLATFORM,)):
                games[el.findtext('DatabaseID')] = el.findtext('Name')
            el.clear()
        elif el.tag == 'GameAlternateName':
            names.setdefault(el.findtext('DatabaseID'), []).append(
                (el.findtext('AlternateName') or '', el.findtext('Region') or ''))
            el.clear()
        elif el.tag == 'GameImage':
            if el.findtext('Type') == IMAGE:
                carts.setdefault(el.findtext('DatabaseID'), []).append(
                    (el.findtext('FileName') or '', el.findtext('Region') or ''))
            el.clear()

own = {}
for gid, name in games.items():
    own.setdefault(title_key(name or ''), set()).add(gid)


def alternate_ok(gid, title, region):
    key = title_key(title)
    if not key:
        return True
    # Short plain names are real titles ("F1", "V-V"), so only the others' are dropped.
    if len(key) < 4 and not title.isascii():
        return False
    # The name it was sold under where this game's cart was (Galaga in Japan) stays.
    released = region and region in {r for _, r in carts.get(gid, [])}
    others = own.get(key, set()) - {gid}
    return released or not others or not region and not any(o in carts for o in others)


lines = set()
for gid, name in games.items():
    for file, region in carts.get(gid, []):
        if region not in REGIONS or not file.endswith('.png') or not set(file) <= SAFE or len(file) > 64:
            continue
        tag = REGIONS[region]
        alternates = [(t, r) for t, r in names.get(gid, []) if alternate_ok(gid, t, r)]
        for title, alt_region in [(name, '')] + alternates:
            # The game's main (English) name too, so a ROM named for its US release finds its
            # Famicom picture.
            if (alt_region in REGIONS or not alt_region) and title and not any(ord(c) < 32 for c in title):
                lines.add(f'{title} ({tag})\t{file}' if tag else f'{title}\t{file}')
header = f'# {PLATFORM} {KIND} pictures from the LaunchBox Games Database ({URL}): name (region), file\n'
OUT.write_text(header + '\n'.join(sorted(lines)) + '\n')
print(f'{len(lines)} names for {sum(1 for g in games if g in carts)} games; wrote {OUT}', file=sys.stderr)
