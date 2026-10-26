"""Exercise the native PICO-8 compatibility shim against a sensorless SDL ABI."""
from pathlib import Path
import os
import subprocess
import tempfile

source = Path(__file__).resolve().parent / "pico8-sdl-nosensor.c"
with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    (root / "sdl.c").write_text(r'''
#include <stdint.h>
#define SENSOR 0x8000u
uint32_t init_flags, subsystem_flags, quit_flags, query_flags;
int query_count;
int SDL_Init(uint32_t flags) {
    init_flags = flags;
    return flags & SENSOR ? -1 : 7;
}
int SDL_InitSubSystem(uint32_t flags) {
    subsystem_flags = flags;
    return flags & SENSOR ? -1 : 9;
}
void SDL_QuitSubSystem(uint32_t flags) { quit_flags = flags; }
uint32_t SDL_WasInit(uint32_t flags) {
    query_flags = flags;
    query_count++;
    return flags ? flags : 0xFFFFu;
}
''')
    (root / "pico.c").write_text(r'''
#include <stdint.h>
#include <stdio.h>
int SDL_Init(uint32_t);
int SDL_InitSubSystem(uint32_t);
void SDL_QuitSubSystem(uint32_t);
uint32_t SDL_WasInit(uint32_t);
extern uint32_t init_flags, subsystem_flags, quit_flags, query_flags;
extern int query_count;
#define CHECK(test) do { if (!(test)) { \
    fprintf(stderr, "failed: %s\n", #test); return 1; } } while (0)
int main(void) {
    CHECK(SDL_Init(0xFFFFu) == 7);
    CHECK(init_flags == 0x7FFFu);
    CHECK(SDL_InitSubSystem(0x8210u) == 9);
    CHECK(subsystem_flags == 0x210u);
    CHECK(SDL_InitSubSystem(0x8000u) == 9);
    CHECK(subsystem_flags == 0);
    SDL_QuitSubSystem(0x8210u);
    CHECK(quit_flags == 0x210u);
    CHECK(SDL_WasInit(0) == 0x7FFFu);
    CHECK(query_flags == 0 && query_count == 1);
    CHECK(SDL_WasInit(0x8000u) == 0);
    CHECK(query_count == 1);
    CHECK(SDL_WasInit(0x8210u) == 0x210u);
    CHECK(query_flags == 0x210u && query_count == 2);
    puts("sensor request masked; video/audio/controller flags and SDL results preserved");
    return 0;
}
''')
    cc = os.environ.get("CC", "cc")
    flags = [cc, "-Wall", "-Wextra", "-Werror"]
    subprocess.run(flags + ["-shared", "-fPIC", str(root / "sdl.c"),
                           "-o", str(root / "libfake-sdl.so")], check=True)
    subprocess.run(flags + [str(root / "pico.c"), "-L" + str(root),
                           "-Wl,-rpath," + str(root), "-lfake-sdl",
                           "-o", str(root / "pico")], check=True)
    subprocess.run(flags + ["-shared", "-fPIC", str(source), "-ldl",
                           "-o", str(root / "shim.so")], check=True)
    env = dict(os.environ)
    env.pop("LD_PRELOAD", None)
    before = subprocess.run([str(root / "pico")], env=env, capture_output=True, text=True)
    assert before.returncode == 1 and "SDL_Init" in before.stderr, before
    env["LD_PRELOAD"] = str(root / "shim.so")
    subprocess.run([str(root / "pico")], env=env, check=True)
