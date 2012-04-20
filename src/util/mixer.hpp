/* mixer.hpp

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
#ifndef UTIL_MIXER_HPP
#define UTIL_MIXER_HPP

#include <util/audio.hpp>
#include <SDL/SDL.h>
#include <list>

const int sampling_rate = 11025;

class Mixer {
public:

  Mixer();
  void init();

  void add_wave(Wave wave, float duration_sec);

  void start();
  void stop();
private:
  typedef int8_t Sample;

  friend void mixer_dispatch(void *userdata, uint8_t* stream, int len);

  void generate(int8_t* stream, int len);

  long current_time;

  struct Wave_Record { Wave wave; long start_t; long end_t; };

  std::list<Wave_Record> waves;
};

void add_wave(Wave wave, float duration_sec);
void add_wave(const Effect_Wave& effect);

#endif
