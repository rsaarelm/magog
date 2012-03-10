/* debug.hpp

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

#ifndef UTIL_DEBUG_HPP
#define UTIL_DEBUG_HPP

#include <stdio.h>
#include <SDL/SDL.h>

class Print_Time {
public:
  Print_Time(const char *msg = "") : msg(msg), ms_ticks(SDL_GetTicks()) {}

  ~Print_Time() {
    printf("%s: took %g seconds.\n", msg, (SDL_GetTicks() - ms_ticks) / 1000.0);
  }
private:
  const char* msg;
  long ms_ticks;
};

#endif
