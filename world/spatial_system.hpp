/* spatial_system.hpp

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
#ifndef WORLD_SPATIAL_SYSTEM_HPP
#define WORLD_SPATIAL_SYSTEM_HPP

#include <world/entities_system.hpp>
#include <world/terrain_system.hpp>

class Spatial_System {
public:
  Spatial_System(
    Entities_System& entities,
    Terrain_System& terrain)
    : entities(entities)
    , terrain(terrain) {}

  bool can_pop(Entity entity, Location loc) const;
  void push(Entity entity);
  void pop(Entity entity, Location loc);

  Footprint footprint(Entity entity) const;
private:
  Entities_System& entities;
  Terrain_System& terrain;
};

#endif
