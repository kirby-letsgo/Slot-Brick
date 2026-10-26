#!/usr/bin/env python3
import hashlib, importlib.util, io, json, sys, tempfile, unittest, zipfile
from pathlib import Path
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('setup',Path(__file__).with_name('setup-portmaster.py'))
setup=importlib.util.module_from_spec(spec);spec.loader.exec_module(setup)

class SetupTests(unittest.TestCase):
 def test_archive_paths_and_symlinks_are_rejected(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t)
   for name in ['../escape','/escape','foo\\escape']:
    with zipfile.ZipFile(root/'bad.zip','w') as z:z.writestr(name,'bad')
    with self.assertRaises(RuntimeError):setup.unpack(root/'bad.zip',root/'out')
   info=zipfile.ZipInfo('link');info.external_attr=(0o120777<<16)
   with zipfile.ZipFile(root/'bad.zip','w') as z:z.writestr(info,'outside')
   with self.assertRaises(RuntimeError):setup.unpack(root/'bad.zip',root/'out')
 def test_hash_mismatch_never_installs_a_download(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t)
   item=dict(name='bad',url='https://example.invalid/fixture',sha256='0'*64)
   with patch('urllib.request.urlopen',return_value=io.BytesIO(b'corrupt')):
    with self.assertRaises(RuntimeError):setup.download(item,root)
   self.assertEqual(list(root.iterdir()),[])
 def test_setup_preserves_existing_manager_game_and_saves(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t);card=root/'card';cache=root/'cache';cache.mkdir()
   (card/'Slot/System').mkdir(parents=True);(card/'Slot/System/portmaster-launch.sh').write_text('')
   (card/'Apps/PortMaster/PortMaster').mkdir(parents=True)
   (card/'Apps/PortMaster/PortMaster/pugwash').write_text('newer manager')
   (card/'Data/ports/balatro/saves').mkdir(parents=True)
   (card/'Data/ports/Balatro.sh').write_text('newer port')
   (card/'Data/ports/balatro/saves/profile.jkr').write_bytes(b'save')
   (card/'Data/ports/balatro/Balatro.exe').write_bytes(b'owned game')
   manifest=[]
   def item(name,data,dest=None):
    sha=hashlib.sha256(data).hexdigest();(cache/sha).write_bytes(data)
    manifest.append(dict(name=name,url='https://example.invalid',sha256=sha,**({'dest':dest} if dest else {})))
   buf=io.BytesIO()
   with zipfile.ZipFile(buf,'w') as z:
    z.writestr('bin/python3','fixture');z.writestr('lib/python3.11/encodings/__init__.py','fixture')
   item('Python',buf.getvalue())
   item('bash',b'bash','bin/bash');item('curl',b'curl','bin/curl-aarch64');item('bzip',b'bzip','lib/libbz2.so.1')
   (root/'portmaster-runtime.json').write_text(json.dumps(manifest))
   with patch.object(setup,'HERE',root),patch.object(sys,'argv',['setup',str(card),'--cache',str(cache)]):setup.main()
   self.assertEqual((card/'Apps/PortMaster/PortMaster/pugwash').read_text(),'newer manager')
   self.assertEqual((card/'Data/ports/Balatro.sh').read_text(),'newer port')
   self.assertEqual((card/'Data/ports/balatro/Balatro.exe').read_bytes(),b'owned game')
   self.assertEqual((card/'Data/ports/balatro/saves/profile.jkr').read_bytes(),b'save')
   self.assertTrue((card/'Slot/System/PortRuntime/bin/curl').stat().st_mode & 0o111)

if __name__=='__main__':unittest.main()
