/* mapgen_system.cpp

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

#include "mapgen_system.hpp"
#include <util/hex.hpp>
#include <util/num.hpp>

using namespace std;

// TODO: Data driven floor and wall tile.

struct Digger {
  Digger(Location origin, Terrain_System& terrain, const Recti& area)
    : origin(origin)
    , terrain(terrain)
    , area(area) {}

  bool dig(const Vec2i& pos) {
    if (forbidden.find(pos) != forbidden.end())
      return false;
    if (!area.contains(pos))
      return false;

    terrain.set(origin + pos, terrain_floor);

    dug.insert(pos);
    edge.erase(pos);

    for (auto a : hex_dirs) {
      auto neighbor = pos + a;
      if (dug.find(neighbor) == dug.end() &&
          forbidden.find(neighbor) == forbidden.end()) {
        if (area.contains(neighbor))
          edge.insert(neighbor);
        else
          forbidden.insert(neighbor); // Ensure that out-of-area edges get filled.
      }
    }

    return true;
  }

  void dig_entrance(const Vec2i& pos, int dir6) {
    Vec2i portal_pos = pos - hex_dirs[dir6];
    // Make a blocked portal enclosure around the portal.
    for (int i = 0; i < 6; i++) {
      if (i != dir6)
        forbidden.insert(portal_pos + hex_dirs[i]);
    }
    forbidden.insert(portal_pos);
    dig(pos);
  }

  void fill_edges() {
    for (auto pos : edge)
      terrain.set(origin + pos, terrain_cave_wall);

    for (auto pos : forbidden)
      terrain.set(origin + pos, terrain_cave_wall);
  }

  int count_neighbor_floors(const Vec2i& pos) {
    int result = 0;
    for (auto a : hex_dirs)
      if (dug.find(pos + a) != dug.end())
        result++;
    return result;
  }

  Location origin;
  Terrain_System& terrain;
  Recti area;
  set<Vec2i> dug;
  set<Vec2i> edge;
  set<Vec2i> forbidden;
};

void Mapgen_System::cave(Plain_Location start, int start_dir6, const Recti& area) {
  const float floor_fraction = 0.5;
  size_t n = area.volume() * floor_fraction;

  Digger state(terrain.location(start), terrain, area);

  state.dig_entrance(Vec2i(0, 0), start_dir6);

  while (state.dug.size() < n && state.edge.size() > 0) {
    auto pick = rand_choice(state.edge);
    Vec2i pos = *pick;

    int n_floor = state.count_neighbor_floors(pos);

    // Decide whether to dig here. Prefer to dig in closed quarters and
    // destroy singleton pillars.
    if (n_floor < 6 && rand_int(n_floor * n_floor) > 1)
      continue;

    state.dig(pos);
  }

  state.fill_edges();
}
