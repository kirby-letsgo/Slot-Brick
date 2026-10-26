#!/usr/bin/env python3
import argparse, importlib.util, os, shutil, subprocess, sys, tempfile, zipfile
from pathlib import Path
parser=argparse.ArgumentParser(description='Test the real PortMaster patcher using synthetic ZIPs, without Balatro game data.')
parser.add_argument('port',type=Path,help='extracted balatro directory from the pinned PortMaster wrapper')
parser.add_argument('sevenzip',type=Path,help='native 7zz/7za executable')
args=parser.parse_args()
repo=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('compat',repo/'brick/card/Slot/System/balatro-patch-compat.py');compat=importlib.util.module_from_spec(spec);spec.loader.exec_module(compat)
port=args.port.resolve(); seven=args.sevenzip.resolve()
assert compat.adapt(b'new upstream script') is None

def run(name, adapted, locale=True, broken=False):
 with tempfile.TemporaryDirectory() as tmp:
  root=Path(tmp); game=root/'Data/ports/balatro';shutil.copytree(port,game)
  shutil.copyfile(seven,game/'bin/7za.aarch64');(game/'bin/7za.aarch64').chmod(0o755)
  script=game/'tools/patchscript'
  script_before=script.read_bytes()
  if adapted:
   system=root/'Slot/System'; (system/'PortRuntime/bin').mkdir(parents=True)
   for helper in ['portmaster-env.sh','balatro-patch-compat.py']:
    shutil.copyfile(repo/'brick/card/Slot/System'/helper,system/helper)
   (system/'PortRuntime/bin/python3').symlink_to(sys.executable)
   (system/'PortRuntime/bin/bash').symlink_to(shutil.which('bash'))
  files={'main.lua':'require "challenges"\n', 'globals.lua':"crt = 70,\nbloom = 1\nshadows = 'On'\n",'game.lua':"love.graphics.setShader( G.SHADERS['CRT'])\n",'cardarea.lua':'(G.SETTINGS.reduced_motion and 0 or 1)*0.02*math.sin(2*G.TIMERS.REAL+card.T.x)\n'*11,'engine/text.lua':'''if self.config.quiver then
if self.config.rotate then letter.r = 0
(G.SETTINGS.reduced_motion and 0 or 1)*0.02*math.sin(2*G.TIMERS.REAL+k)
if self.config.float then letter.offset.y = 0
if self.config.bump then letter.offset.y = 0
''','resources/shaders/CRT.fs':'vec4 effect(vec4 color, Image tex, vec2 tc, vec2 pc)\n{\n}\n','resources/shaders/background.fs':'for (int i=0; i < 5; i++) {}\n'}
  if locale:files['localization/es_419.lua']='return { a_mult = "+#1# Multi.", }\n'
  if broken:files['game.lua']='unsupported version\n'
  source=game/'Balatro.love'
  with zipfile.ZipFile(source,'w') as z:
   for p,data in files.items():z.writestr(p,data)
  original=source.read_bytes()
  env=dict(os.environ,GAMEDIR=str(game),GAMEFILE='Balatro.love',OUTPUT_GAME='Balatro_pm',BUILD_STAMP=str(game/'.balatro-build.txt'),LAYOUT='small',FONT='nunito',PERFORMANCE='on',PERF_OPTIMIZATIONS='1',DEVICE_ARCH='aarch64')
  if adapted:env.update(SLOT_ROOT=str(root/'Slot'),BASH_ENV=str(root/'Slot/System/portmaster-env.sh'))
  p=subprocess.run(['bash',str(script)],env=env,capture_output=True,text=True,timeout=30)
  print(name,p.returncode,p.stdout[-350:],p.stderr)
  assert source.read_bytes()==original
  assert script.read_bytes()==script_before
  if adapted:assert (root/'Slot/Logs/balatro-patch.log').read_text()==p.stdout
  if adapted and not broken:
   assert p.returncode==0
   with zipfile.ZipFile(game/'Balatro_pm') as z:
    assert 'portmaster/loc_fix.lua' in z.namelist()
    if locale:assert z.read('localization/es_419.lua').decode()==files['localization/es_419.lua']
  else:
   assert p.returncode!=0
   assert not (game/'Balatro_pm').exists()
   if broken:assert 'Patch mismatch in game.lua' in p.stdout
run('original formatted locale',False)
run('adapted formatted locale',True)
run('adapted missing locale',True,locale=False)
run('adapted bad performance source',True,broken=True)
