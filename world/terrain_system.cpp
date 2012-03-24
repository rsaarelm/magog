/* terrain_system.cpp

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

#include "terrain_system.hpp"
#include <world/location.hpp>
#include <util/alg.hpp>
#include <util/hex.hpp>

Location Terrain_System::location(Area_Index area, const Vec2i& pos) {
  return Location(*this, area, pos[0], pos[1]);
}

Location Terrain_System::location(Plain_Location loc) {
  return Location(*this, loc);
}

Location Terrain_System::location() {
  return Location(*this);
}

Terrain Terrain_System::get(Plain_Location loc) const {
  return assoc_find_or(terrain, loc, terrain_void);
}

void Terrain_System::set(Plain_Location loc, Terrain ter) {
  terrain[loc] = ter;
}

void Terrain_System::clear(Plain_Location loc) {
  terrain.erase(loc);
}

Portal Terrain_System::get_portal(Plain_Location loc) const {
  return assoc_find_or(portals, loc, Portal());
}

void Terrain_System::set_portal(Plain_Location loc, Portal portal) {
  portals[loc] = portal;
}

void Terrain_System::clear_portal(Plain_Location loc) {
  portals.erase(loc);
}

bool Terrain_System::blocks_shot(Plain_Location loc) {
  auto kind = terrain_data[get(loc)].kind;
  return kind == wall_terrain || kind == curtain_terrain;
}

bool Terrain_System::blocks_sight(Plain_Location loc) {
  auto kind = terrain_data[get(loc)].kind;
  return kind == wall_terrain || kind == void_terrain || kind == curtain_terrain;
}

std::vector<Location> Terrain_System::area_locations(Area_Index area) {
  ASSERT(area != 0);
  std::vector<Location> result;
  auto i = terrain.upper_bound({static_cast<Area_Index>(area - 1), 0, 0}),
    j = terrain.lower_bound({static_cast<Area_Index>(area + 1), 0, 0});

  while (i->first.area < area) ++i;
  while (j->first.area > area) --j;
  ++j;
  while (i != j) {
    result.push_back(location(i->first));
    ++i;
  }
  return result;
}

bool Terrain_System::is_wall(Plain_Location loc) const {
  return terrain_data[get(loc)].kind == wall_terrain;
}

int Terrain_System::wall_mask(Location loc) const {
  int result = 0;
  for (size_t i = 0; i < hex_dirs.size(); i++)
    result += is_wall(loc + hex_dirs[i]) << i;
  return result;
}
