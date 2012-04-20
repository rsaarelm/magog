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
#include <world/spatial_index.hpp>
#include <vector>
#include <set>

class Spatial_System {
public:
  Spatial_System(
    Entities_System& entities,
    Terrain_System& terrain);

  bool is_open(
    Plain_Location loc,
    std::function<bool(Entity)> is_blocking_pred=[](Entity) { return true; }) const;

  bool can_pop(Entity entity, Location loc) const;
  void push(Entity entity);
  void pop(Entity entity);
  void pop(Entity entity, Location loc);

  Location location(Entity entity) const;
  Footprint footprint(Entity entity, Location center) const;
  Footprint footprint(Entity entity) const;

  std::vector<Entity> entities_at(Location location) const;
  std::vector<std::pair<Vec2i, Entity>> entities_with_offsets_at(Location location) const;
  std::vector<Entity> entities_on(const Footprint& footprint) const;

  void destroy_pushed();
private:
  Spatial_System(const Spatial_System&);
  Spatial_System& operator=(const Spatial_System&);

  Entities_System& entities;
  Terrain_System& terrain;

  Spatial_Index<Entity> index;
  std::set<Entity> pushed;
};

#endif
