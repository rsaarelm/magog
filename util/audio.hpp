/* audio.hpp

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
#ifndef UTIL_AUDIO_HPP
#define UTIL_AUDIO_HPP

#include <functional>

enum Waveform {
  sine_wave,
  saw_wave,
  square_wave,
  noise_wave,
};

typedef std::function<float (float)> Wave;

struct Sound_Effect {
  Waveform waveform;
  float attack;
  float decay;
  float sustain;
  float release;
  float frequency;
};

struct Effect_Wave {
  float duration;
  float volume;
  Sound_Effect fx;
  float operator()(float t) const;
};

#endif
