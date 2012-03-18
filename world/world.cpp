/* world.cpp

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

#include "world.hpp"
#include <world/effects.hpp>
#include <world/rules.hpp>
#include <world/parts.hpp>
#include <stdexcept>

/// Main container for all of the game state.
class World {
 public:
  static void init();
  static World& get();
  static void clear();

  Terrain get_terrain(Location location);
  void set_terrain(Location location, Terrain cell);

  /// Return the entity whose turn it is to act now.
  ///
  /// Throws Entity_Not_Found if there are no entities that can act.
  Entity active_entity();

  /// Called when done with the current active entity to move to the next one.
  void next_entity();

  // TODO: Use indexed lookup to a static terrain set instead of having
  // individual data here to compress the structure.
  std::map<Location, Terrain> terrain;
  std::map<Location, Portal> portal;

  // Note to optimizers: Heavy-duty component systems want to have parts of
  // one kind in contiguous memory, so that, for example, all physics parts
  // can be processed using fast vectorized code. This simple system does not
  // support that. Shouldn't be a problem unless heavy physics-style stuff is
  // needed.
  std::map<Entity, std::map<Kind, std::unique_ptr<Part>>> entities;

  Entity_Id next_entity_id;
 private:
  World();
  World(const World&);
  World& operator=(const World&);

  static std::unique_ptr<World> s_world;

  Entity previous_entity;
};

void msg(const char* fmt) {
  raw_msg(fmt);
}


std::unique_ptr<World> World::s_world;

World& World::get() {
  if (s_world.get() == nullptr)
    s_world.reset(new World());
  return *s_world.get();
}

Part* find_part(Entity entity, Kind kind) {
  auto& entities = World::get().entities;
  auto iter = entities.find(entity);
  if (iter == entities.end())
    throw Entity_Not_Found();

  auto part_iter = iter->second.find(kind);
  if (part_iter == iter->second.end())
    throw Part_Not_Found();

  return(part_iter->second.get());
}

void add_part(Entity entity, std::unique_ptr<Part> new_part) {
  ASSERT(assoc_contains(World::get().entities, entity));
  World::get().entities[entity][new_part->get_kind()] = std::move(new_part);
}

void World::clear() {
  s_world.reset(nullptr);
}

void clear_world() {
  World::get().clear();
}

World::World()
    : next_entity_id(256) // IDs below this are reserved for fixed stuff.
    , previous_entity(nullptr, -1)
{}

Terrain World::get_terrain(Location location) {
  return assoc_find_or(terrain, location, terrain_void);
}

void World::set_terrain(Location location, Terrain cell) {
  terrain[location] = cell;
}

Entity World::active_entity() {
  auto i = entities.upper_bound(previous_entity);
  if (i != entities.end())
    return i->first;

  // Nothing left after previous_entity, loop to start.
  previous_entity = Entity(nullptr, -1);
  i = entities.upper_bound(previous_entity);
  if (i != entities.end())
    return i->first;

  // No entities, period.
  throw Entity_Not_Found();
}

void World::next_entity() {
  auto i = entities.upper_bound(previous_entity);
  if (i != entities.end())
    previous_entity = i->first;
  else
    previous_entity = Entity(nullptr, -1);

  try {
    start_turn_update(active_entity());
  } catch (Entity_Not_Found &e) {}
}

Terrain get_terrain(Location location) {
  return World::get().get_terrain(location);
}

void _set_terrain(Location location, Terrain cell) {
  World::get().set_terrain(location, cell);
}

Portal get_portal(Location location) {
  auto result = assoc_find(World::get().portal, location);
  if (result) {
    return *result;
  }
  else
    return Portal();
}

void _set_portal(Location location, Portal portal) {
  World::get().portal[location] = portal;
}

void clear_portal(Location location) {
  World::get().portal.erase(location);
}

Entity new_entity(Entity_Id id) {
  auto result = Entity(nullptr, id);
  ASSERT(!entity_exists(result));
  World::get().entities[result] = std::map<Kind, std::unique_ptr<Part>>();
  return result;
}

Entity new_entity() {
  return new_entity(World::get().next_entity_id++);
}

#include <ui/game_screen.hpp>

void delete_entity(Entity entity) {
  // TODO: Notify components of removal

  // XXX HACKHACKHACK
  // TODO: Get rid of this function and this kludge
  static_cast<Game_Screen*>(Game_Loop::get().top_state())->spatial.push(entity);
  World::get().entities.erase(entity);
}

bool entity_exists(Entity entity) {
  return assoc_contains(World::get().entities, entity);
}

Entity active_entity() {
  return World::get().active_entity();
}

void next_entity() {
  World::get().next_entity();
}

template <typename Archive>
void serialize_world(Archive& ar) {
  // TODO: Use the Boost serialization idiom here
}

// XXX: serialize_world, despite being a template function, must be
// implemented on the .cpp, since it relies on the hidden World class
// internals. To prevent linking errors, the serializers used must be defined
// explicitly here:

#define REGISTER_ARCHIVE(A) template void serialize_world<A>(A& ar)

// REGISTER_ARCHIVE(boost::archive::binary_oarchive);
// REGISTER_ARCHIVE(boost::archive::binary_iarchive);

#undef REGISTER_ARCHIVE
