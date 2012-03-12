/* view_space.cpp

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

#include "view_space.hpp"
#include <world/fov.hpp>
#include <util/alg.hpp>
#include <util/hex.hpp>

using namespace boost;
using namespace std;

void View_Space::do_fov(int radius, Location origin) {
  prune();
  visible.clear();
  auto fov = hex_field_of_view(radius, origin);
  for (auto& pair : fov) {
    auto pos = pair.first + subjective_pos;
    view[pos] = pair.second;
    visible.insert(pair.second);
  }
}

boost::optional<Location> View_Space::at(const Vec2i& pos) const {
  return assoc_find(view, pos);
}

bool View_Space::is_seen(Location loc) const {
  return assoc_contains(visible, loc);
}

void View_Space::prune() {
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
