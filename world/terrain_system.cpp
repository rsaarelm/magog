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
#include <world/world.hpp>

Location Terrain_System::loc(uint16_t area, const Vec2i& pos) {
  return Location(this, area, pos[0], pos[1]);
}

Terrain Terrain_System::get(Location loc) const {
  // TODO: Use Terrain_System storage
  return get_terrain(loc);
}

void Terrain_System::set(Location loc, Terrain terrain) {
  // TODO
  _set_terrain(loc, terrain);
}

void Terrain_System::clear(Location loc) {
  // TODO
}

Portal Terrain_System::get_portal(Location loc) const {
  // TODO
  return ::get_portal(loc);
}

void Terrain_System::set_portal(Location loc, Portal portal) {
  // TODO
  _set_portal(loc, portal);
}

void Terrain_System::clear_portal(Location loc) {
  portals.erase(loc);
}

bool Terrain_System::blocks_shot(Location loc) {
  auto kind = terrain_data[get(loc)].kind;
  return kind == wall_terrain || kind == curtain_terrain;
}

bool Terrain_System::blocks_sight(Location loc) {
  auto kind = terrain_data[get_terrain(loc)].kind;
  return kind == wall_terrain || kind == void_terrain || kind == curtain_terrain;
}
