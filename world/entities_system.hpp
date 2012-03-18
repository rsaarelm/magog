/* entities_system.hpp

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
#ifndef WORLD_ENTITIES_SYSTEM_HPP
#define WORLD_ENTITIES_SYSTEM_HPP

#include <world/entity.hpp>
#include <map>
#include <memory>

class Entities_System {
public:
  Entities_System();

  Entity create(Entity_Id id);
  Entity create();
  void destroy(Entity entity);

  void add(Entity entity, std::unique_ptr<Part> part);
  bool has(Entity entity, Kind kind) const;
  Part* get(Entity entity, Kind kind);

private:
  Entity_Id next_entity_id;
  std::map<Entity, std::map<Kind, std::unique_ptr<Part>>> entities;
};

#endif
