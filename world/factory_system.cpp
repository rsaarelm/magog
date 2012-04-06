/* factory_system.cpp

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

#include "factory_system.hpp"
#include <world/parts.hpp>
#include <world/footprint.hpp>
#include <util/num.hpp>

Entity Factory_System::build(Spec spec, Entity entity) {
  if (!entity) {
    entity = entities.create();
  }

  switch (spec) {
  case spec_player:
    entities.add(entity, std::unique_ptr<Part>(new Blob_Part(icon_player, 7, 10, 5)));
    return entity;
  case spec_dreg:
    entities.add(entity, std::unique_ptr<Part>(new Blob_Part(icon_dreg, 3, 6, 2)));
    return entity;
  case spec_thrall:
    entities.add(entity, std::unique_ptr<Part>(new Blob_Part(icon_thrall, 5, 8, 4)));
    return entity;
  default:
    throw Spec_Exception();
  }
}

Footprint Factory_System::footprint(Spec spec, Location center) const {
  // TODO: Make this data-driven
  return small_footprint(center);
}

bool Factory_System::can_spawn(Spec spec, Location loc) const {
  for (auto& pair : footprint(spec, loc)) {
    if (!spatial.is_open(pair.second))
      return false;
  }
  return true;
}

Location Factory_System::random_spawn_point(Spec spec, Area_Index area) const {
  // XXX: Very slow.
  auto locations = terrain.area_locations(area);
  for (int ntries = 256; ntries; ntries--) {
    auto loc = rand_choice(locations);
    if (can_spawn(spec, *loc))
      return *loc;
  }
  throw Spawn_Point_Exception();
}

Entity Factory_System::spawn(Spec spec, Location loc, Entity entity) {
  Entity result = build(spec, entity);
  spatial.pop(result, loc);
  return result;
}
