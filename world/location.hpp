/* location.hpp

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

#ifndef WORLD_LOCATION_HPP
#define WORLD_LOCATION_HPP

#include <util.hpp>
#include <boost/optional.hpp>
#include <map>

// By convention, area 0 is no-op.

struct Portal {
  Portal() : area(0), delta_x(0), delta_y(0) {}

  Portal(uint16_t area, int8_t x, int8_t y) : area(area), delta_x(x), delta_y(y) {}

  Portal(uint16_t area, const Vec2i& pos) : area(area), delta_x(pos[0]), delta_y(pos[1]) {}

  bool operator==(const Portal& rhs) const {
    return delta_x == rhs.delta_x && delta_y == rhs.delta_y && area == rhs.area;
  }

  operator bool() const {
    return area != 0 || delta_x != 0 || delta_y != 0;
  }

  uint16_t area;
  int8_t delta_x, delta_y;
};

struct Location {
  Location(uint16_t area, int8_t x, int8_t y) : area(area), x(x), y(y) {}

  Location(uint16_t area, const Vec2i& pos) : area(area), x(pos[0]), y(pos[1]) {}

  Location() : area(0), x(0), y(0) {}

  bool operator<(const Location& rhs) const {
    if (area < rhs.area) return true;
    if (area > rhs.area) return false;

    if (y < rhs.y) return true;
    if (y > rhs.y) return false;

    if (x < rhs.x) return true;
    if (x > rhs.x) return false;

    return false;
  }

  bool operator==(const Location& rhs) const {
    return !(*this < rhs) && !(rhs < *this);
  }

  /// Offset without portaling.
  Location raw_offset(const Vec2i& offset) const {
    return Location(area, x + offset[0], y + offset[1]);
  }

  /// Location through a possible portal in this location.
  Location portaled() const;

  Location operator+(const Vec2i& offset) const {
    // TODO: Make it portal by default.

    //return raw_offset(offset).portaled();
    return raw_offset(offset);
  }

  Location operator+(Portal portal) const {
    return Location(portal.area ? portal.area : area, x + portal.delta_x, y + portal.delta_y);
  }

  // XXX: Deprecated
  Location operator+(boost::optional<Portal> portal) const {
    if (portal)
      return *this + *portal;
    else
      return *this;
  }

  // XXX: Deprecated
  Location offset_and_portal(const Vec2i& offset) const {
    return (*this + offset).portaled();
  }

  size_t hash() const {
    return (((area << 1) ^ y) << 1) ^ x;
  }

  struct Hasher {
    size_t operator()(const Location& location) const { return location.hash(); }
  };

  struct Equator {
    bool operator()(const Location& lhs, const Location& rhs) const { return lhs == rhs; }
  };

  // Fit the whole thing into a 32-bit word.
  uint16_t area;
  int8_t x, y;
};


typedef std::map<Vec2i, Location> Relative_Fov;

typedef std::map<Vec2i, Location> Footprint;

#endif
