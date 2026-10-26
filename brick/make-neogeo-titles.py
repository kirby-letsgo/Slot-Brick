#!/usr/bin/env python3
"""Regenerate Neo Geo names from the source revision pinned in build-fbneo.sh.
Usage: python3 brick/make-neogeo-titles.py FBNEO_SOURCE
"""
from pathlib import Path
import re, sys
r=Path(__file__).resolve().parent.parent
src=(Path(sys.argv[1])/'src/burn/drv/neogeo/d_neogeo.cpp').read_text();rows=[]
for m in re.finditer(r'struct BurnDriver\w* \w+\s*=\s*\{(.*?)\n\};',src,re.S):
 block=m.group(1)
 if 'BDF_GAME_WORKING' not in block:continue
 fields=re.findall(r'"((?:[^"\\]|\\.)*)"|\b(NULL)\b',block.split('\n\tNULL,')[0])
 if len(fields)<6:continue
 name=fields[0][0];title=fields[5][0].split('\\0')[0]
 if not re.fullmatch('[a-z0-9_]+',name) or not title:continue
 parent=fields[1][0]
 title=title.replace('\\"','"')
 rows.append((name,title,parent))
(r/'crates/slot-store/src/neogeo-titles.tsv').write_text(''.join('\t'.join(row)+'\n' for row in sorted(rows)))
print('Neo Geo title entries:',len(rows))
