/* terrain.cpp

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

#include "terrain.hpp"
#include <array>

Terrain slope_terrain(int dir6) {
  static const std::array<Terrain, 6> slopes{{
    terrain_slope_n,
    terrain_slope_ne,
    terrain_slope_se,
    terrain_slope_s,
    terrain_slope_sw,
    terrain_slope_nw}};
  return slopes.at(dir6);
}
