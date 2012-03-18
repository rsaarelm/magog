/* world.hpp

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

#ifndef WORLD_WORLD_HPP
#define WORLD_WORLD_HPP

/// \file world.hpp \brief Core interface to the game state

#include <util.hpp>
#include <world/entity.hpp>
#include <world/location.hpp>
#include <world/terrain.hpp>
#include <world/view_space.hpp>
#include <world/spatial_index.hpp>
#include <boost/optional.hpp>
#include <exception>
#include <map>
#include <set>
#include <vector>
#include <string>

Part* find_part(Entity entity, Kind kind);
void add_part(Entity entity, std::unique_ptr<Part> new_part);

void clear_world();

Spatial_Index<Entity>& get_spatial_index();

// TODO variadics.
void msg(const char* fmt);

// TODO REMOVE
Terrain get_terrain(Location location);

// TODO REMOVE
void _set_terrain(Location location, Terrain cell);

// TODO REMOVE
Portal get_portal(Location location);
// TODO REMOVE
void _set_portal(Location location, Portal portal);

std::pair<std::map<Location, Terrain>::const_iterator,
          std::map<Location, Terrain>::const_iterator>
area_locations(uint16_t area);

// XXX: Return type should be considered just some iterable type, the exact
// form may change.
std::vector<Entity> all_entities();
std::vector<Entity> entities_at(Location location);
std::vector<std::pair<Vec2i, Entity>> entities_with_offsets_at(Location location);
std::vector<Entity> entities_on(const Footprint& footprint);

Entity new_entity(Entity_Id id);
Entity new_entity();
void delete_entity(Entity entity);
bool entity_exists(Entity entity);
Entity active_entity();
void next_entity();
Entity entity_after(Entity entity);

template<typename Archive>
void serialize_world(Archive& ar);

#endif
