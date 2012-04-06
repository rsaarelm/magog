/* audio.cpp

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

#include "audio.hpp"
#include <util/num.hpp>
#include <math.h>

float Effect_Wave::operator()(float t) const {
  float phase = t * fx.frequency;
  // From sine wave.
  const float period = 2 * pi;
  float mod_phase = (phase / period) - floor(phase / period);
  float amp = volume;

  if (t < fx.attack) {
    // Fx.Attack phase
    amp = lerp(0.0f, 1.0f, t / fx.attack);
  } else if (t < fx.attack + fx.decay) {
    // Decay phase
    amp = lerp(1.0f, fx.sustain, (t - fx.attack) / fx.decay);
  } else if (t < duration - fx.release) {
    // Sustain phase
    amp = fx.sustain;
  } else if (t < duration) {
    amp = lerp(fx.sustain, 0.0f, (t - duration + fx.release) / fx.release);
  } else {
    amp = 0;
  }

  float f = 0.0;
  switch (fx.waveform) {
  case sine_wave:
    f = sin(phase);
    break;
  case saw_wave:
    f = -1.0 + mod_phase;
    break;
  case square_wave:
    if (mod_phase < 1.0)
      f = -1.0;
    else
      f = 1.0;
    break;
  case noise_wave:
  {
    double integer;
    double fraction;
    fraction = modf(phase, &integer);
    f = lerp(int_noise(integer), int_noise(integer + 1), fraction);
    break;
  }
  }
  return amp * f;
}
