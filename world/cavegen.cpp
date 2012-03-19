/* cavegen.cpp

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

#include "cavegen.hpp"
#include <world/terrain.hpp>
#include <util.hpp>
#include <set>
#include <array>

using namespace std;

void dig(Location loc) {
  // XXX: Move the hardcoded terrain data somewhere else.
  // TODO: Figure out formatting the walls around the floor.

  // TODO: Reactivate for systems code when cavegen is in use again.
  //set_terrain(loc, terrain_floor);
}

void generate_cave(Location origin, const Recti& area) {
  set<Vec2i> dug;
  set<Vec2i> edge;

  Vec2i pos = area.min() + area.dim() / 2;
  const float floor_fraction = 0.5;
  size_t n = area.volume() * floor_fraction;

  dig(origin + pos);
  dug.insert(pos);
  for (auto a : hex_dirs) {
    if (dug.find(pos + a) == dug.end()) {
      edge.insert(pos + a);
    }
  }

  while (dug.size() < n && edge.size() > 0) {
    auto pick = rand_choice(edge);
    pos = *pick;

    int n_floor = 0;
    for (auto a : hex_dirs)
      if (dug.find(pos + a) != dug.end())
        n_floor++;

    // Decide whether to dig here. Prefer to dig in closed quarters and
    // destroy singleton pillars.
    if (n_floor < 6 && rand_int(n_floor * n_floor) > 1)
      continue;

    edge.erase(pick);

    dig(origin + pos);
    dug.insert(pos);
    for (auto a : hex_dirs) {
      if (dug.find(pos + a) == dug.end() && area.contains(pos + a)) {
        edge.insert(pos + a);
      }
    }
  }
}
