/* terrain.hpp

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

#ifndef WORLD_TERRAIN_HPP
#define WORLD_TERRAIN_HPP

#include <util/color.hpp>

enum Terrain_Kind : uint8_t {
  void_terrain,
  open_terrain,
  wall_terrain,
  water_terrain,
  curtain_terrain,
};

struct Terrain_Data {
  int icon;
  Color color;
  Terrain_Kind kind;
};

// Specify terrain enum and data using X-macros
// (http://en.wikibooks.org/wiki/C_Programming/Preprocessor#X-Macros)

#define TERRAIN_TABLE \
  X(terrain_void,         8, "magenta",      void_terrain)          \
  X(terrain_grass,        1, "olive drab",   open_terrain)          \
  X(terrain_sand,         1, "khaki",        open_terrain)          \
  X(terrain_floor,        1, "dim gray",     open_terrain)          \
  X(terrain_water,        2, "royal blue",   water_terrain)         \
  X(terrain_wall_center, 14, "gray",         wall_terrain)          \
  X(terrain_wall_x,      15, "gray",         wall_terrain)          \
  X(terrain_wall_y,      16, "gray",         wall_terrain)          \
  X(terrain_wall_xy,     17, "gray",         wall_terrain)          \
  X(terrain_forest,       7, "forest green", curtain_terrain)

#define X(a, b, c, d) a,
enum Terrain : uint8_t {
  TERRAIN_TABLE
  NUM_TERRAINS
};
#undef X

#define X(a, b, c, d) {b, c, d},
const Terrain_Data terrain_data[] = {
  TERRAIN_TABLE
};
#undef X

#undef TERRAIN_TABLE

#endif
