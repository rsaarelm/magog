/* entity.cpp

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

#include "entity.hpp"
#include <world/world.hpp>
#include <world/parts.hpp>

/// Add a part to the Entity. Ownership of the part will move to Entity.
void Entity::add_part(Part* new_part) {
  ::add_part(*this, std::unique_ptr<Part>(new_part));
}

bool Entity::exists() const {
  return entity_exists(*this);
}

Location Entity::location() const {
  return as<Blob_Part>().loc;
}

void Entity::_push() {
  auto& index = get_spatial_index();
  if (index.has(*this))
    index.remove(*this);
}

bool Entity::_can_pop(Location location) const {
  for (auto& pair : footprint(location)) {
    auto& loc = pair.second;
    auto kind = terrain_data[get_terrain(loc)].kind;
    if (!(kind == open_terrain || kind == curtain_terrain))
      return false;
    // TODO: Handle entity collisions.
  }
  return true;
}

void Entity::_pop() {
  ASSERT(!get_spatial_index().has(*this));
  get_spatial_index().add(*this, footprint());
}

void Entity::_pop(Location location) {
  as<Blob_Part>().loc = location;
  _pop();
}

Footprint Entity::footprint(Location center) const {
  Footprint result;
  result[Vec2i(0, 0)] = center;
  if (as<Blob_Part>().big) {
    for (auto& i : hex_dirs) {
      result[i] = center + i;
    }
  }
  return result;
}

Footprint Entity::footprint() const {
  return footprint(as<Blob_Part>().loc);
}
