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
#include <array>

enum Terrain_Flag : uint8_t {
  wallform_flag    = 1 << 1,
  block_move_flag  = 1 << 2,
  block_shot_flag  = 1 << 3,
  block_sight_flag = 1 << 4,
  void_flag        = 1 << 5,
};

enum Terrain_Kind : uint8_t {
  open_terrain = 0,
  void_terrain = block_move_flag | block_shot_flag | block_sight_flag | void_flag,
  wall_terrain = wallform_flag | block_move_flag | block_shot_flag | block_sight_flag,
  block_terrain = block_move_flag | block_shot_flag | block_sight_flag,
  water_terrain  = block_move_flag, // XXX May be made different from window_terrain later
  window_terrain  = block_move_flag,
  curtain_terrain = block_shot_flag | block_sight_flag,
};

struct Terrain_Data {
  const char* icon_set;
  int icon;
  Color color;
  Terrain_Kind kind;
};

// Specify terrain enum and data using X-macros
// (http://en.wikibooks.org/wiki/C_Programming/Preprocessor#X-Macros)

#define TERRAIN_TABLE \
  X(terrain_void,       "terrain",   8, "magenta",        void_terrain)          \
  X(terrain_grass,      "terrain",   1, "olive drab",     open_terrain)          \
  X(terrain_sand,       "terrain",   1, "khaki",          open_terrain)          \
  X(terrain_floor,      "terrain",   1, "dim gray",       open_terrain)          \
  X(terrain_water,      "terrain",   2, "royal blue",     water_terrain)         \
  X(terrain_tree,       "terrain",   7, "forest green",   window_terrain)        \
  X(terrain_menhir,     "terrain",   3, "gray",           block_terrain)         \
  X(terrain_wall,       "wall",      0, "gray",           wall_terrain)          \
  X(terrain_cave_wall,  "wall",      4, "dark goldenrod", wall_terrain)          \
  X(terrain_slope_n,    "slope",     0, "gray",           open_terrain)          \
  X(terrain_slope_ne,   "slope",     1, "gray",           open_terrain)          \
  X(terrain_slope_se,   "slope",     2, "gray",           open_terrain)          \
  X(terrain_slope_s,    "slope",     3, "gray",           open_terrain)          \
  X(terrain_slope_sw,   "slope",     4, "gray",           open_terrain)          \
  X(terrain_slope_nw,   "slope",     5, "gray",           open_terrain)          \

#define X(a, b, c, d, e) a,
enum Terrain : uint8_t {
  TERRAIN_TABLE
  NUM_TERRAINS
};
#undef X

#define X(a, b, c, d, e) {b, c, d, e},
const Terrain_Data terrain_data[] = {
  TERRAIN_TABLE
};
#undef X

#undef TERRAIN_TABLE

Terrain slope_terrain(int dir6);

#endif
