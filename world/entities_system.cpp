/* entities_system.cpp

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

#include "entities_system.hpp"
#include <util/core.hpp>
#include <util/alg.hpp>

Entities_System::Entities_System()
  : next_entity_id(1024) {
}

Entity Entities_System::create(Entity_Id id) {
  auto result = Entity(this, id);
  ASSERT(!assoc_contains(entities, result));
  entities[result];
  return result;
}

Entity Entities_System::create() {
  return create(next_entity_id++);
}

void Entities_System::destroy(Entity entity) {
  // TODO: Notify components of removal
  // TODO: Systems might want notification too, eg. the spatial index.
  ASSERT(assoc_contains(entities, entity));
  entities.erase(entity);
}

void Entities_System::add(Entity entity, std::unique_ptr<Part> part) {
  ASSERT(assoc_contains(entities, entity));
  Kind kind = part->get_kind();
  entities[entity][kind] = std::move(part);
}

bool Entities_System::has(Entity entity, Kind kind) const {
  //ASSERT(assoc_contains(entities, entity));
  auto iter = entities.find(entity);
  if (iter == entities.end())
    throw Entity_Not_Found();
  return iter->second.find(kind) != iter->second.end();
  //return assoc_contains(entities[entity], kind);
}

Part* Entities_System::get(Entity entity, Kind kind) {
  ASSERT(assoc_contains(entities, entity));

  auto& parts = entities[entity];
  auto part_iter = parts.find(kind);
  if (part_iter == parts.end())
    return nullptr;

  return(part_iter->second.get());
}

// A forward-declarable wrapper for fetching parts
Part* _find_part(Entities_System* entities_system, Entity entity, Kind kind) {
  return entities_system->get(entity, kind);
}
