/* factory_system.hpp

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
#ifndef WORLD_FACTORY_SYSTEM_HPP
#define WORLD_FACTORY_SYSTEM_HPP

#include <world/location.hpp>
#include <world/entities_system.hpp>
#include <world/spatial_system.hpp>
#include <stdexcept>

/// A spec is a type for specifying the entity created by the factory.
/// It may end up very complex, but will be a simple enum for now.
enum Spec {
  spec_player = 1,
  spec_dreg,
  spec_thrall,
  NUM_ENTITY_TYPES
};

class Spec_Exception : public std::exception {
 public:
  virtual const char* what() const throw() {
    return "Invalid spec";
  }
};

class Spawn_Point_Exception : public std::exception {
public:
  virtual const char* what() const throw() {
    return "Couldn't find spawn point";
  }
};

class Factory_System {
public:
  Factory_System(
    Entities_System& entities,
    Terrain_System& terrain,
    Spatial_System& spatial)
  : entities(entities)
  , terrain(terrain)
  , spatial(spatial) {}

  Entity build(Spec spec, Entity entity=0);

  Footprint footprint(Spec spec, Location center) const;

  bool can_spawn(Spec spec, Location loc) const;

  Location random_spawn_point(Spec spec, Area_Index area) const;

  Entity spawn(Spec spec, Location loc, Entity entity=0);
private:

  Entities_System& entities;
  Terrain_System& terrain;
  Spatial_System& spatial;
};

#endif
