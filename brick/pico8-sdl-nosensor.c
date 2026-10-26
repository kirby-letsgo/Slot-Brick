/*
 * PICO-8 requests SDL's sensor subsystem, which the Brick firmware omits.
 * Keep the firmware display/audio/controller drivers and mask sensor requests.
 * The audio bridge below applies Slot volume to the native audio stream.
 * Adapted from josegonzalez/minui-pico-8-pak, shim/sdl-nosensor.c (MIT).
 * Copyright (c) 2025 Jose Diaz-Gonzalez. See licenses/pico8-sdl-MIT.txt.
 */

#define _GNU_SOURCE
#include <dlfcn.h>
#include <stddef.h>
#include <stdint.h>

/* SDL_INIT_SENSOR, from SDL.h. Not included so the shim builds without SDL. */
#define SDL_INIT_SENSOR 0x00008000u

static void *real_symbol(const char *name)
{
  return dlsym(RTLD_NEXT, name);
}

static int forward_init(const char *name, uint32_t flags)
{
  int (*real)(uint32_t) = (int (*)(uint32_t))real_symbol(name);

  if (real == NULL) {
    return -1;
  }

  return real(flags & ~SDL_INIT_SENSOR);
}

/*
 * The load-bearing wrapper. pico-8 links SDL2 directly, so its call binds to
 * this definition and dlsym(RTLD_NEXT) then reaches the real one.
 */
int SDL_Init(uint32_t flags)
{
  return forward_init("SDL_Init", flags);
}

/*
 * Wrapped for a caller that reaches it directly. Nothing here relies on
 * catching SDL2's own internal SDL_Init to SDL_InitSubSystem call, which need
 * not go through a symbol this object can interpose.
 */
int SDL_InitSubSystem(uint32_t flags)
{
  return forward_init("SDL_InitSubSystem", flags);
}

/* Defensive only: SDL2 refcounts and no-ops a subsystem that never started. */
void SDL_QuitSubSystem(uint32_t flags)
{
  void (*real)(uint32_t) = (void (*)(uint32_t))real_symbol("SDL_QuitSubSystem");

  if (real != NULL) {
    real(flags & ~SDL_INIT_SENSOR);
  }
}

/*
 * SDL_WasInit(0) means "report every subsystem", so the flag cannot simply be
 * masked off: a caller asking only about the sensor would be handed the whole
 * set instead of the "not initialised" answer it is owed.
 */
uint32_t SDL_WasInit(uint32_t flags)
{
  uint32_t (*real)(uint32_t) = (uint32_t (*)(uint32_t))real_symbol("SDL_WasInit");

  if (real == NULL) {
    return 0;
  }

  if (flags == 0) {
    return real(0) & ~SDL_INIT_SENSOR;
  }

  if ((flags & ~SDL_INIT_SENSOR) == 0) {
    return 0;
  }

  return real(flags & ~SDL_INIT_SENSOR);
}

/* Apply Slot's volume after PICO-8 mixes, including live mute changes. */
#include "pico8-audio.h"
#include "ports-openal.h"
