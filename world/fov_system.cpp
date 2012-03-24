/* fov_system.cpp

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

#include "fov_system.hpp"
#include <world/parts.hpp>
#include <util/hex.hpp>

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
  std::function<void(const Vec2i&, Location)> callback,
  int range,
  Location local_origin,
  Angle begin = Angle{0, 1},
  Angle end = Angle{6, 1}) {
  if (begin.radius > range)
    return;
  Fov_Group group(local_origin, *begin);
  for (auto a = begin; a.is_below(end); ++a) {
    if (Fov_Group(local_origin, *a) != group) {
      process(callback, range, local_origin, a, end);
      if (!group.opaque)
        process(callback, range, local_origin + group.portal, begin.extended(), a.extended());
      return;
    }
    callback(*a, local_origin + *a);
  }
  if (!group.opaque)
    process(callback, range, local_origin + group.portal, begin.extended(), end.extended());
}

Relative_Fov hex_field_of_view(
    int range,
    Location origin) {
  Relative_Fov result;
  result[Vec2i(0, 0)] = origin;
  process([&](const Vec2i& pos, Location loc) { result[pos] = loc; }, range, origin);
  return result;
}


bool Fov_System::is_seen(Location loc) {
  return assoc_contains(visible, loc);
}

Location Fov_System::view_location(const Vec2i& relative_pos) {
  auto iter = view.find(relative_pos + subjective_pos);
  if (iter == view.end())
    return terrain.location();
  else
    return terrain.location(iter->second);
}

void Fov_System::run(int radius, Location origin, Fov_Callback callback) {
  process(callback, radius, origin);
}

void Fov_System::do_fov(int radius, Location origin, const Vec2i& offset) {
  prune();
  auto fov = hex_field_of_view(radius, origin);
  for (auto& pair : fov) {
    auto pos = pair.first + subjective_pos + offset;
    view[pos] = pair.second;
    visible.insert(pair.second);
  }
}

void Fov_System::do_fov() {
  const int radius = 8;

  clear_seen();
  if (entities.as<Blob_Part>(spatial.get_player()).big) {
    // Big entities see with their edge cells too so that they're not completely
    // blind in a forest style terrain.
    for (auto i : hex_dirs)
      do_fov(radius, spatial.location(spatial.get_player()) + i, i);
  }
  do_fov(radius, spatial.location(spatial.get_player()));
}

void Fov_System::prune() {
  // Cut down far-away parts if the storage threatens to become too large.
  const int capacity = 65536;
  const int keep_radius = 48;

  // XXX: This could be a lot more efficient if the underlying structure was a quadtree.
  if (view.size() > capacity) {
    for (auto i = view.begin(); i != view.end(); i++) {
      auto dist = hex_dist(subjective_pos - i->first);
      if (dist > keep_radius) {
        view.erase(i);
      }
    }
  }
}
