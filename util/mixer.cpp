/* mixer.cpp

   Copyright (C) 2012 Risto Saarelma

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "mixer.hpp"
#include <util/core.hpp>
#include <util/game_loop.hpp>
#include <vector>

static long sec_to_sample_time(float sec) {
  return static_cast<long>(sec * sampling_rate);
}

static float sample_time_to_sec(long sample_time) {
  return sample_time / static_cast<float>(sampling_rate);
}

Mixer::Mixer()
  : current_time(0) {
}

void Mixer::add_wave(Wave wave, float duration_sec) {
  waves.push_back({wave, current_time, current_time + sec_to_sample_time(duration_sec)});
  start();
}

void Mixer::start() {
  SDL_PauseAudio(0);
}

void Mixer::stop() {
  SDL_PauseAudio(1);
}

void mixer_dispatch(void *userdata, uint8_t* stream, int len) {
  Mixer* mixer = reinterpret_cast<Mixer*>(userdata);
  mixer->generate(reinterpret_cast<int8_t*>(stream), len);
}

void Mixer::init() {
  SDL_AudioSpec spec;
  spec.freq = sampling_rate;
  spec.format = AUDIO_S8;
  spec.channels = 1;
  spec.samples = 512;
  spec.callback = mixer_dispatch;
  spec.userdata = this;
  if (SDL_OpenAudio(&spec, nullptr) < 0)
    die("Audio error: %s\n", SDL_GetError());

}

void Mixer::generate(int8_t* stream, int len) {
  for (int i = 0; i < len; i++) {
    long t = current_time + i;
    std::vector<float> samples;
    for (auto sample = waves.begin(); sample != waves.end();) {
      if (sample->end_t < t) {
        sample = waves.erase(sample);
      } else {
        samples.push_back(sample->wave(sample_time_to_sec(t - sample->start_t)));
        sample++;
      }
    }
    float avg = 0;
    if (samples.size() > 0) {
      for (auto sample : samples)
        avg += sample;
      avg /= samples.size();
    }
    stream[i] = static_cast<int8_t>(avg * (1 << (sizeof(Sample) * 8 - 1)));
  }
  current_time += len;
  if (waves.empty())
    stop();
}

void add_wave(Mixer::Wave wave, float duration_sec) {
  Game_Loop::get().mixer.add_wave(wave, duration_sec);
}
