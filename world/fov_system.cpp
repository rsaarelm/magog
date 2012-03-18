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
#include <world/fov.hpp>
#include <world/parts.hpp>
#include <world/rules.hpp>

bool Fov_System::is_seen(Location loc) {
  return assoc_contains(visible, loc);
}

Location Fov_System::view_location(const Vec2i& relative_pos) {
  auto iter = view.find(relative_pos + subjective_pos);
  if (iter == view.end())
    return Location();
  else
    return iter->second;
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
  if (get_player().as<Blob_Part>().big) {
    // Big entities see with their edge cells too so that they're not completely
    // blind in a forest style terrain.
    for (auto i : hex_dirs)
      do_fov(radius, get_player().location() + i, i);
  }
  do_fov(radius, get_player().location());
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
