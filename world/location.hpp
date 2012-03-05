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

#endif
