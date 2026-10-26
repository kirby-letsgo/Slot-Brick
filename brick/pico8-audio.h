/* Slot's native PICO-8 audio bridge. SDL2 ABI declarations, no SDK dependency. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

typedef struct {
  int freq;
  uint16_t format;
  uint8_t channels, silence;
  uint16_t samples, padding;
  uint32_t size;
  void (*callback)(void *, uint8_t *, int);
  void *userdata;
} SlotAudioSpec;

typedef struct SlotAudio {
  uint32_t device;
  uint16_t format;
  void (*callback)(void *, uint8_t *, int);
  void *userdata;
  struct SlotAudio *next;
} SlotAudio;

static SlotAudio *slot_devices, *slot_legacy;
static __thread int slot_opening_legacy;
static const uint8_t *slot_volume;
static uint8_t slot_initial_volume = 100;

static void slot_audio_init(void)
{
  static int initialized;
  if (initialized) return;
  initialized = 1;
  const char *initial = getenv("SLOT_PICO8_VOLUME");
  if (initial) {
    int value = atoi(initial);
    slot_initial_volume = value < 0 ? 0 : value > 100 ? 100 : (uint8_t)value;
  }
  const char *value = getenv("SLOT_PICO8_VOLUME_FD");
  if (value) {
    char *end;
    long fd = strtol(value, &end, 10);
    if (*value && !*end && fd >= 0) {
      void *mapped = mmap(NULL, 1, PROT_READ, MAP_SHARED, (int)fd, 0);
      if (mapped != MAP_FAILED) slot_volume = mapped;
      else fprintf(stderr, "slot: PICO-8 live volume mapping failed\n");
    }
  }
}

static void slot_audio_scale(uint8_t *data, int len, uint16_t format)
{
  unsigned volume = slot_volume ? __atomic_load_n(slot_volume, __ATOMIC_RELAXED) : slot_initial_volume;
  if (volume >= 100) return;
  unsigned gain = volume * volume;
  unsigned bits = format & 255;
  unsigned bytes = bits / 8;
  if (!(bytes == 1 || bytes == 2 || bytes == 4)) return;
  for (int i = 0; i + (int)bytes <= len; i += (int)bytes) {
    uint32_t raw = 0;
    for (unsigned j = 0; j < bytes; j++)
      raw |= (uint32_t)data[i + j] << (8 * ((format & 0x1000) ? bytes - j - 1 : j));
    if ((format & 0x100) && bits == 32) {
      float sample;
      memcpy(&sample, &raw, 4);
      sample = volume ? sample * ((float)gain / 10000.0f) : 0.0f;
      memcpy(&raw, &sample, 4);
    } else {
      int64_t midpoint = (int64_t)1 << (bits - 1);
      int64_t sample = raw;
      if (format & 0x8000) {
        if (sample >= midpoint) sample -= midpoint * 2;
      } else sample -= midpoint;
      sample = sample * gain / 10000;
      if (!(format & 0x8000)) sample += midpoint;
      raw = (uint32_t)sample;
    }
    for (unsigned j = 0; j < bytes; j++)
      data[i + j] = (uint8_t)(raw >> (8 * ((format & 0x1000) ? bytes - j - 1 : j)));
  }
}

static void slot_audio_callback(void *userdata, uint8_t *stream, int len)
{
  SlotAudio *audio = userdata;
  audio->callback(audio->userdata, stream, len);
  slot_audio_scale(stream, len, audio->format);
}

uint32_t SDL_OpenAudioDevice(const char *device, int capture, const SlotAudioSpec *desired,
                            SlotAudioSpec *obtained, int changes)
{
  uint32_t (*real)(const char *, int, const SlotAudioSpec *, SlotAudioSpec *, int) = real_symbol("SDL_OpenAudioDevice");
  if (!real || !desired) return 0;
  if (capture || slot_opening_legacy) return real(device, capture, desired, obtained, changes);
  slot_audio_init();
  SlotAudio *audio = calloc(1, sizeof(*audio));
  if (!audio) return 0;
  SlotAudioSpec request = *desired, actual;
  audio->callback = desired->callback;
  audio->userdata = desired->userdata;
  if (request.callback) { request.callback = slot_audio_callback; request.userdata = audio; }
  audio->device = real(device, capture, &request, obtained ? &actual : NULL, changes);
  if (!audio->device) { free(audio); return 0; }
  audio->format = obtained ? actual.format : desired->format;
  if (obtained) { *obtained = actual; obtained->callback = desired->callback; obtained->userdata = desired->userdata; }
  audio->next = slot_devices;
  slot_devices = audio;
  return audio->device;
}

void SDL_CloseAudioDevice(uint32_t device)
{
  void (*real)(uint32_t) = real_symbol("SDL_CloseAudioDevice");
  if (real) real(device); /* joins the callback thread before freeing its context */
  SlotAudio **at = &slot_devices;
  while (*at) {
    if ((*at)->device == device) { SlotAudio *old = *at; *at = old->next; free(old); break; }
    at = &(*at)->next;
  }
}

int SDL_OpenAudio(const SlotAudioSpec *desired, SlotAudioSpec *obtained)
{
  int (*real)(const SlotAudioSpec *, SlotAudioSpec *) = real_symbol("SDL_OpenAudio");
  if (!real || !desired || slot_legacy) return -1;
  slot_audio_init();
  SlotAudio *audio = calloc(1, sizeof(*audio));
  if (!audio) return -1;
  SlotAudioSpec request = *desired, actual;
  audio->callback = desired->callback;
  audio->userdata = desired->userdata;
  if (request.callback) { request.callback = slot_audio_callback; request.userdata = audio; }
  slot_opening_legacy = 1;
  int result = real(&request, obtained ? &actual : NULL);
  slot_opening_legacy = 0;
  if (result < 0) { free(audio); return result; }
  audio->format = obtained ? actual.format : desired->format;
  if (obtained) { *obtained = actual; obtained->callback = desired->callback; obtained->userdata = desired->userdata; }
  slot_legacy = audio;
  return result;
}

void SDL_CloseAudio(void)
{
  void (*real)(void) = real_symbol("SDL_CloseAudio");
  if (real) real();
  free(slot_legacy);
  slot_legacy = NULL;
}

int SDL_QueueAudio(uint32_t device, const void *data, uint32_t len)
{
  int (*real)(uint32_t, const void *, uint32_t) = real_symbol("SDL_QueueAudio");
  if (!real || len > INT32_MAX) return -1;
  SlotAudio *audio = slot_devices;
  while (audio && audio->device != device) audio = audio->next;
  if (!audio || !len) return real(device, data, len);
  uint8_t *copy = malloc(len);
  if (!copy) return -1;
  memcpy(copy, data, len);
  slot_audio_scale(copy, (int)len, audio->format);
  int result = real(device, copy, len);
  free(copy);
  return result;
}
