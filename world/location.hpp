#ifndef WORLD_LOCATION_HPP
#define WORLD_LOCATION_HPP

#include <world/actor.hpp>
#include <util.hpp>
#include <boost/optional.hpp>
#include <map>

struct Portal {
  Vec2i delta;
  Actor area;

  bool operator==(const Portal& rhs) const {
    return delta == rhs.delta && area == rhs.area;
  }
};

struct Location {
  Vec2i pos;
  Actor area;

  bool operator<(const Location& rhs) const {
    if (area < rhs.area)
      return true;
    else if (area == rhs.area) {
      return pos < rhs.pos;
    }
    return false;
  }

  bool operator==(const Location& rhs) const {
    return !(*this < rhs) && !(rhs < *this);
  }

  Location operator+(const Vec2i& offset) const {
    return Location{pos + offset, area};
  }

  Location operator+(const Portal& portal) const {
    return Location{pos + portal.delta, portal.area};
  }

  Location operator+(const boost::optional<Portal>& portal) const {
    if (portal)
      return Location{pos + portal->delta, portal->area};
    else
      return Location(*this);
  }

  size_t hash() const {
    return (((pos[0] << 1) ^ pos[1]) << 1) ^ area.hash();
  }

  struct Hasher {
    size_t operator()(const Location& location) const { return location.hash(); }
  };

  struct Equator {
    bool operator()(const Location& lhs, const Location& rhs) const { return lhs == rhs; }
  };
};


typedef std::map<Vec2i, Location> Relative_Fov;


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
};
#undef X

#define X(a, b, c, d) {b, c, d},
const Terrain_Data terrain_data[] = {
  TERRAIN_TABLE
};
#undef X

#undef TERRAIN_TABLE

#endif
