/* spatial_system.cpp

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

#include "spatial_system.hpp"
#include <world/parts.hpp>

bool Spatial_System::can_pop(Entity entity, Location loc) const {
  for (auto& pair : footprint(entity, loc)) {
    auto& foot_loc = pair.second;
    auto kind = terrain_data[terrain.get(foot_loc)].kind;
    if (!(kind == open_terrain || kind == curtain_terrain))
      return false;
    // TODO: Handle entity collisions.
  }
  return true;
}

void Spatial_System::push(Entity entity) {
  if (index.has(entity))
    index.remove(entity);
}

void Spatial_System::pop(Entity entity) {
  ASSERT(!index.has(entity));
  index.add(entity, footprint(entity));
}

void Spatial_System::pop(Entity entity, Location loc) {
  entities.as<Blob_Part>(entity).loc = loc;
  pop(entity);
}

Location Spatial_System::location(Entity entity) const {
  return entities.as<Blob_Part>(entity).loc;
}

Footprint Spatial_System::footprint(Entity entity, Location center) const {
  Footprint result;
  result[Vec2i(0, 0)] = center;
  if (entities.as<Blob_Part>(entity).big) {
    for (auto& i : hex_dirs) {
      result[i] = center + i;
    }
  }
  return result;
}

Footprint Spatial_System::footprint(Entity entity) const {
  return footprint(entity, location(entity));
}

std::vector<Entity> Spatial_System::entities_at(Location location) {
  std::vector<Entity> result;
  auto range = index.equal_range(location);
  for (auto i = range.first; i != range.second; ++i) {
    result.push_back(i->second.second);
  }
  return result;
}

std::vector<std::pair<Vec2i, Entity>> Spatial_System::entities_with_offsets_at(
  Location location) {
  std::vector<std::pair<Vec2i, Entity>> result;
  auto range = index.equal_range(location);
  for (auto i = range.first; i != range.second; ++i) {
    result.push_back(i->second);
  }
  return result;
}

std::vector<Entity> Spatial_System::entities_on(const Footprint& footprint) {
  std::vector<Entity> result;
  for (auto& pair : footprint) {
    auto range = index.equal_range(pair.second);
    for (auto i = range.first; i != range.second; ++i) {
      result.push_back(i->second.second);
    }
  }
  return result;
}
