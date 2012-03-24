/* footprint.cpp

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

#include "footprint.hpp"
#include <util/hex.hpp>

Footprint small_footprint(Location center) {
  Footprint result;
  result[Vec2i(0, 0)] = center;
  return result;
}

Footprint large_footprint(Location center) {
  Footprint result;
  result[Vec2i(0, 0)] = center;
  for (auto& i : hex_dirs) {
    result[i] = center + i;
  }
  return result;
}
