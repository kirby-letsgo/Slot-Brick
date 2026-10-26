/* LÖVE uses OpenAL rather than SDL audio. Preserve the game's listener gain and
 * multiply it by Slot's gain. LÖVE polls alGetSourcei throughout its audio update,
 * so live volume changes apply on that thread with its current OpenAL context.
 * No polling thread, mixer commands, or writes to the original game are needed. */
#include <pthread.h>

#define SLOT_AL_GAIN 0x100a
static pthread_mutex_t slot_al_lock = PTHREAD_MUTEX_INITIALIZER;
static void *slot_al_context;
static float slot_al_gain = 1.0f;
static unsigned slot_al_volume = 101;

/* Called with slot_al_lock held. Do nothing until there is a current context. */
static int slot_al_sync(void)
{
  void *(*current)(void) = (void *(*)(void))real_symbol("alcGetCurrentContext");
  void (*get)(int, float *) = (void (*)(int, float *))real_symbol("alGetListenerf");
  void (*set)(int, float) = (void (*)(int, float))real_symbol("alListenerf");
  void *ctx = current ? current() : NULL;
  if (!ctx || !get || !set) return 0;
  slot_audio_init();
  if (ctx != slot_al_context) {
    slot_al_context = ctx;
    get(SLOT_AL_GAIN, &slot_al_gain);
    slot_al_volume = 101;
  }
  unsigned volume = slot_volume ? __atomic_load_n(slot_volume, __ATOMIC_RELAXED) : slot_initial_volume;
  if (volume != slot_al_volume) {
    slot_al_volume = volume;
    set(SLOT_AL_GAIN, slot_al_gain * ((float)(volume * volume) / 10000.0f));
  }
  return 1;
}

void alListenerf(int param, float value)
{
  void (*real)(int, float) = (void (*)(int, float))real_symbol("alListenerf");
  if (!real) return;
  pthread_mutex_lock(&slot_al_lock);
  if (param == SLOT_AL_GAIN && value >= 0.0f && slot_al_sync()) {
    slot_al_gain = value;
    value *= (float)(slot_al_volume * slot_al_volume) / 10000.0f;
  }
  real(param, value);
  pthread_mutex_unlock(&slot_al_lock);
}

void alListenerfv(int param, const float *values)
{
  if (param == SLOT_AL_GAIN && values) { alListenerf(param, *values); return; }
  void (*real)(int, const float *) = (void (*)(int, const float *))real_symbol("alListenerfv");
  if (real) real(param, values);
}

void alGetListenerf(int param, float *value)
{
  void (*real)(int, float *) = (void (*)(int, float *))real_symbol("alGetListenerf");
  if (!real) return;
  pthread_mutex_lock(&slot_al_lock);
  real(param, value);
  if (param == SLOT_AL_GAIN && value && slot_al_sync()) *value = slot_al_gain;
  pthread_mutex_unlock(&slot_al_lock);
}

int alGetError(void)
{
  int (*real)(void) = (int (*)(void))real_symbol("alGetError");
  pthread_mutex_lock(&slot_al_lock);
  slot_al_sync();
  pthread_mutex_unlock(&slot_al_lock);
  return real ? real() : 0;
}

void alGetSourcei(unsigned source, int param, int *value)
{
  void (*real)(unsigned, int, int *) = (void (*)(unsigned, int, int *))real_symbol("alGetSourcei");
  pthread_mutex_lock(&slot_al_lock);
  slot_al_sync();
  pthread_mutex_unlock(&slot_al_lock);
  if (real) real(source, param, value);
}

void alSourcePlay(unsigned source)
{
  void (*real)(unsigned) = (void (*)(unsigned))real_symbol("alSourcePlay");
  pthread_mutex_lock(&slot_al_lock);
  slot_al_sync();
  pthread_mutex_unlock(&slot_al_lock);
  if (real) real(source);
}

void alSourcePlayv(int count, const unsigned *sources)
{
  void (*real)(int, const unsigned *) = (void (*)(int, const unsigned *))real_symbol("alSourcePlayv");
  pthread_mutex_lock(&slot_al_lock);
  slot_al_sync();
  pthread_mutex_unlock(&slot_al_lock);
  if (real) real(count, sources);
}

void alcDestroyContext(void *context)
{
  void (*real)(void *) = (void (*)(void *))real_symbol("alcDestroyContext");
  pthread_mutex_lock(&slot_al_lock);
  if (slot_al_context == context) { slot_al_context = NULL; slot_al_volume = 101; }
  pthread_mutex_unlock(&slot_al_lock);
  if (real) real(context);
}
