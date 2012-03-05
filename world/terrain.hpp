#ifndef WORLD_TERRAIN_HPP
#define WORLD_TERRAIN_HPP

#include <util/color.hpp>

enum Terrain_Kind : uint8_t {
  void_terrain,
  open_terrain,
  wall_terrain,
  water_terrain,
};

struct Terrain_Data {
  int icon;
  Color color;
  Terrain_Kind kind;
};

// Specify terrain enum and data using X-macros
// (http://en.wikibooks.org/wiki/C_Programming/Preprocessor#X-Macros)

#define TERRAIN_TABLE \
  X(terrain_void,         8, "magenta",     void_terrain)          \
  X(terrain_grass,        5, "olive drab",  open_terrain)          \
  X(terrain_sand,         5, "khaki",       open_terrain)          \
  X(terrain_floor,        5, "dim gray",    open_terrain)          \
  X(terrain_water,        6, "royal blue",  water_terrain)         \
  X(terrain_wall_center,  1, "gray",        wall_terrain)          \
  X(terrain_wall_x,       2, "gray",        wall_terrain)          \
  X(terrain_wall_y,       3, "gray",        wall_terrain)          \
  X(terrain_wall_xy,      4, "gray",        wall_terrain)

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
