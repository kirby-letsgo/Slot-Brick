#!/usr/bin/env python3
"""Check OpenAL gain preservation, startup mute and live native-port volume."""
from pathlib import Path
import os, subprocess, tempfile
repo=Path(__file__).resolve().parent.parent
with tempfile.TemporaryDirectory() as d:
 p=Path(d)
 (p/'fake.c').write_text('''
 static float gain=1.0f; static int error=0; static int context=1;
 void alGetSourcei(unsigned s,int p,int *v){(void)s;(void)p;*v=42;}
 void alSourcePlay(unsigned s){(void)s;}
 void *alcGetCurrentContext(void){return context ? &context : 0;}
 void alListenerf(int param,float value){if(param==0x100a)gain=value;else error=0xa002;}
 void alListenerfv(int p,const float *v){alListenerf(p,*v);}
 void alGetListenerf(int p,float *v){(void)p;*v=gain;}
 int alGetError(void){int e=error;error=0;return e;}
 void alcDestroyContext(void *p){(void)p;context=0;}
 float actual(void){return gain;}
 ''')
 (p/'test.c').write_text('''
 #include <assert.h>
 #include <fcntl.h>
 #include <stdlib.h>
 #include <unistd.h>
 extern void alListenerf(int,float),alListenerfv(int,const float*),alGetListenerf(int,float*);
 extern void alGetSourcei(unsigned,int,int*),alSourcePlay(unsigned);
 extern int alGetError(void);extern float actual(void);
 int main(void){
 int fd=atoi(getenv("SLOT_PICO8_VOLUME_FD")); unsigned char volume=0;float logical;
 alSourcePlay(1);assert(actual()==0.0f);assert(alGetError()==0);
 alListenerf(0x100a,0.8f);assert(actual()==0.0f);
 alGetListenerf(0x100a,&logical);assert(logical==0.8f);
 volume=50;assert(pwrite(fd,&volume,1,0)==1);int state;alGetSourcei(1,123,&state);assert(state==42);
 assert(actual()>0.199f && actual()<0.201f);
 float value=0.4f;alListenerfv(0x100a,&value);assert(actual()>0.099f && actual()<0.101f);
 volume=100;assert(pwrite(fd,&volume,1,0)==1);assert(alGetError()==0);assert(actual()==0.4f);
 alListenerf(123,1.0f);assert(alGetError()==0xa002);
 volume=0;assert(pwrite(fd,&volume,1,0)==1);assert(alGetError()==0);assert(actual()==0.0f);
 return 0;}
 ''')
 flags=['cc','-Wall','-Wextra','-Werror']
 subprocess.run(flags+['-shared','-fPIC',str(p/'fake.c'),'-o',str(p/'libfake.so')],check=True)
 subprocess.run(flags+['-shared','-fPIC',str(repo/'brick/pico8-sdl-nosensor.c'),'-ldl','-pthread','-o',str(p/'shim.so')],check=True)
 subprocess.run(flags+[str(p/'test.c'),'-L'+d,'-lfake','-Wl,-rpath,'+d,'-o',str(p/'test')],check=True)
 with tempfile.TemporaryFile() as f:
  f.write(b'\x00');f.flush()
  env=dict(os.environ,LD_PRELOAD=str(p/'shim.so'),SLOT_PICO8_VOLUME_FD=str(f.fileno()),SLOT_PICO8_VOLUME='0')
  subprocess.run([str(p/'test')],env=env,pass_fds=[f.fileno()],check=True)
print('OpenAL startup mute, live gain, game volume and errors: passed')
