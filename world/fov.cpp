/* fov.cpp

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

#include "fov.hpp"
#include <world/rules.hpp>
#include <util.hpp>
#include <array>
#include <cmath>

using namespace std;

struct Fov_Group {
  bool opaque;
  Portal portal;

  Fov_Group(Location origin, const Vec2i& offset)
    : opaque((origin + offset).blocks_sight())
    , portal(origin.raw_offset(offset).get_portal()) {}

  bool operator!=(const Fov_Group& rhs) {
    return rhs.opaque != opaque || rhs.portal != portal;
  }
};

struct Angle {
  float pos;
  int radius;

  int winding_index() const {
    return floor(pos + 0.5);
  }

  int end_index() const {
    return ceil(pos + 0.5);
  }

  bool is_below(const Angle& end_angle) const {
    return winding_index() < end_angle.end_index();
  }

  Vec2i operator*() const {
    // XXX: Could cache this.
    return hex_circle_vec(radius, winding_index());
  }

  Angle& operator++() {
    pos += 0.5;
    pos = floor(pos);
    pos += 0.5;
    return *this;
  }

  Angle extended() const {
    return Angle{pos * (radius + 1) / radius, radius + 1};
  }
};

void process(
    Relative_Fov& rfov,
    int range,
    Location local_origin,
    Angle begin = Angle{0, 1},
    Angle end = Angle{6, 1}) {
  if (begin.radius > range)
    return;
  Fov_Group group(local_origin, *begin);
  for (auto a = begin; a.is_below(end); ++a) {
    if (Fov_Group(local_origin, *a) != group) {
      if (!group.opaque)
        process(rfov, range, local_origin + group.portal, begin.extended(), a.extended());
      process(rfov, range, local_origin, a, end);
      return;
    }
    rfov[*a] = local_origin + *a;
  }
  if (!group.opaque)
    process(rfov, range, local_origin + group.portal, begin.extended(), end.extended());
}

Relative_Fov hex_field_of_view(
    int range,
    Location origin) {
  Relative_Fov result;
  result[Vec2i(0, 0)] = origin;
  process(result, range, origin);
  return result;
}
