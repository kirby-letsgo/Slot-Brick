"""Exercise live PICO-8 gain with real interposition and an inherited shared fd."""
from pathlib import Path
import os
import subprocess
import tempfile

source = Path(__file__).resolve().parent / 'pico8-sdl-nosensor.c'
abi = r'''
#include <stdint.h>
#include <stddef.h>
typedef struct {
 int freq; uint16_t format; uint8_t channels, silence;
 uint16_t samples, padding; uint32_t size;
 void (*callback)(void *, uint8_t *, int); void *userdata;
} Spec;
uint32_t SDL_OpenAudioDevice(const char *, int, const Spec *, Spec *, int);
void SDL_CloseAudioDevice(uint32_t);
int SDL_OpenAudio(const Spec *, Spec *);
void SDL_CloseAudio(void);
int SDL_QueueAudio(uint32_t, const void *, uint32_t);
'''
with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    (root/'sdl.c').write_text(abi + r'''
#include <string.h>
Spec saved[8]; int closed; unsigned next = 1;
uint8_t queued[64];
uint32_t SDL_OpenAudioDevice(const char *name, int capture, const Spec *want, Spec *got, int changes) {
 (void)name; (void)capture; (void)changes;
 unsigned id = next++;
 saved[id] = *want;
 if (got) *got = *want;
 return id;
}
void SDL_CloseAudioDevice(uint32_t id) { (void)id; closed++; }
int SDL_OpenAudio(const Spec *want, Spec *got) {
 /* Deliberately interpose the nested call, as some SDL builds do. */
 saved[0] = *want; SDL_OpenAudioDevice(NULL, 0, want, got, 0); return 0;
}
void SDL_CloseAudio(void) { closed++; }
int SDL_QueueAudio(uint32_t id, const void *data, uint32_t len) {
 (void)id; memcpy(queued, data, len); return 0;
}
void render(unsigned id, void *out, int len) { saved[id].callback(saved[id].userdata, out, len); }
''')
    (root/'pico.c').write_text(abi + r'''
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
extern void render(unsigned, void *, int);
extern uint8_t queued[64];
extern int closed;
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed: %s\n", #x); return 1; } } while (0)
static int16_t samples[] = {10000, -10000, 32767, -32768};
static int bad_userdata;
static void callback(void *userdata, uint8_t *out, int len) {
 if (userdata != samples) bad_userdata++;
 memcpy(out, samples, len);
}
int main(void) {
 int fd = atoi(getenv("SLOT_PICO8_VOLUME_FD"));
 uint8_t volume = 0; CHECK(pwrite(fd, &volume, 1, 0) == 1);
 Spec spec = { .freq=44100, .format=0x8010, .channels=2, .samples=1024,
               .callback=callback, .userdata=samples }, got;
 uint32_t id = SDL_OpenAudioDevice(NULL, 0, &spec, &got, 0);
 CHECK(id && got.callback == callback && got.userdata == samples);
 int16_t out[4]; render(id, out, sizeof(out));
 CHECK(out[0] == 0 && out[3] == 0); /* muted before the very first callback */
 volume = 50; CHECK(pwrite(fd, &volume, 1, 0) == 1);
 render(id, out, sizeof(out));
 CHECK(out[0] == 2500 && out[1] == -2500 && out[2] == 8191 && out[3] == -8192);
 volume = 100; CHECK(pwrite(fd, &volume, 1, 0) == 1);
 render(id, out, sizeof(out)); CHECK(memcmp(out, samples, sizeof(out)) == 0);
 volume = 0; CHECK(pwrite(fd, &volume, 1, 0) == 1);
 render(id, out, sizeof(out)); CHECK(out[0] == 0);
 SDL_CloseAudioDevice(id);
 /* Legacy API, including nested OpenAudioDevice: gain must only be applied once. */
 volume = 50; CHECK(pwrite(fd, &volume, 1, 0) == 1);
 CHECK(SDL_OpenAudio(&spec, &got) == 0);
 render(0, out, sizeof(out)); CHECK(out[0] == 2500 && !bad_userdata);
 SDL_CloseAudio();
 /* Queue API leaves the caller's buffer untouched, and supports unsigned silence. */
 spec.callback = NULL; spec.format = 0x0008;
 id = SDL_OpenAudioDevice(NULL, 0, &spec, NULL, 0);
 uint8_t u8[] = {0, 128, 255};
 CHECK(SDL_QueueAudio(id, u8, sizeof(u8)) == 0);
 CHECK(queued[0] == 96 && queued[1] == 128 && queued[2] == 159 && u8[0] == 0);
 SDL_CloseAudioDevice(id);
 /* Float32 and signed big-endian streams use the negotiated format. */
 spec.format = 0x8120;
 id = SDL_OpenAudioDevice(NULL, 0, &spec, &got, 1);
 float f[] = {1.0f, -0.5f}, scaled[2];
 CHECK(SDL_QueueAudio(id, f, sizeof(f)) == 0); memcpy(scaled, queued, sizeof(scaled));
 CHECK(scaled[0] == 0.25f && scaled[1] == -0.125f);
 SDL_CloseAudioDevice(id);
 spec.format = 0x9010;
 id = SDL_OpenAudioDevice(NULL, 0, &spec, &got, 1);
 uint8_t be[] = {0x40, 0, 0xc0, 0};
 CHECK(SDL_QueueAudio(id, be, sizeof(be)) == 0);
 CHECK(queued[0] == 0x10 && queued[1] == 0 && queued[2] == 0xf0 && queued[3] == 0);
 SDL_CloseAudioDevice(id);
 CHECK(closed == 5);
 puts("startup mute, live gain, legacy/device/queued audio and sample formats passed");
 return 0;
}
''')
    cc = os.environ.get('CC', 'cc')
    flags = [cc, '-Wall', '-Wextra', '-Werror']
    subprocess.run(flags + ['-shared','-fPIC',str(root/'sdl.c'),'-o',str(root/'libfake-sdl.so')],check=True)
    subprocess.run(flags + [str(root/'pico.c'),'-L'+str(root),'-Wl,-rpath,'+str(root),'-lfake-sdl','-o',str(root/'pico')],check=True)
    subprocess.run(flags + ['-shared','-fPIC',str(source),'-ldl','-o',str(root/'shim.so')],check=True)
    with tempfile.TemporaryFile() as volume:
        volume.write(bytes([0])); volume.flush()
        env = dict(os.environ, LD_PRELOAD=str(root/'shim.so'), SLOT_PICO8_VOLUME_FD=str(volume.fileno()))
        subprocess.run([str(root/'pico')], env=env, pass_fds=(volume.fileno(),), check=True)
