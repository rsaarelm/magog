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
#include <world/fov.hpp>
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

  /// Return the actor whose turn it is to act now.
  ///
  /// Throws Actor_Not_Found if there are no actors that can act.
  Actor active_actor();

  /// Called when done with the current active actor to move to the next one.
  void next_actor();

  // TODO: Use indexed lookup to a static terrain set instead of having
  // individual data here to compress the structure.
  std::map<Location, Terrain> terrain;
  std::map<Location, Portal> portal;

  // Note to optimizers: Heavy-duty component systems want to have parts of
  // one kind in contiguous memory, so that, for example, all physics parts
  // can be processed using fast vectorized code. This simple system does not
  // support that. Shouldn't be a problem unless heavy physics-style stuff is
  // needed.
  std::map<Actor, std::map<Kind, std::unique_ptr<Part>>> actors;

  Actor_Id next_actor_id;
  View_Space view_space;

  Spatial_Index<Actor> spatial_index;
 private:
  World();
  World(const World&);
  World& operator=(const World&);

  static std::unique_ptr<World> s_world;

  Actor previous_actor;
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

Part* find_part(Actor actor, Kind kind) {
  auto& actors = World::get().actors;
  auto iter = actors.find(actor);
  if (iter == actors.end())
    throw Actor_Not_Found();

  auto part_iter = iter->second.find(kind);
  if (part_iter == iter->second.end())
    throw Part_Not_Found();

  return(part_iter->second.get());
}

void add_part(Actor actor, std::unique_ptr<Part> new_part) {
  ASSERT(assoc_contains(World::get().actors, actor));
  World::get().actors[actor][new_part->get_kind()] = std::move(new_part);
}

Spatial_Index<Actor>& get_spatial_index() {
  return World::get().spatial_index;
}

void World::clear() {
  s_world.reset(nullptr);
}

void clear_world() {
  World::get().clear();
}

World::World()
    : next_actor_id(256) // IDs below this are reserved for fixed stuff.
    , previous_actor(-1)
{}

Terrain World::get_terrain(Location location) {
  return assoc_find_or(terrain, location, terrain_void);
}

void World::set_terrain(Location location, Terrain cell) {
  terrain[location] = cell;
}

Actor World::active_actor() {
  auto i = actors.upper_bound(previous_actor);
  if (i != actors.end())
    return i->first;

  // Nothing left after previous_actor, loop to start.
  previous_actor = Actor(-1);
  i = actors.upper_bound(previous_actor);
  if (i != actors.end())
    return i->first;

  // No actors, period.
  throw Actor_Not_Found();
}

void World::next_actor() {
  auto i = actors.upper_bound(previous_actor);
  if (i != actors.end())
    previous_actor = i->first;
  else
    previous_actor = Actor(-1);

  try {
    start_turn_update(active_actor());
  } catch (Actor_Not_Found &e) {}
}


bool is_seen(Location location) {
  return World::get().view_space.is_seen(location) > 0;
}

boost::optional<Location> view_space_location(const Vec2i& relative_pos) {
  auto& view = World::get().view_space;
  return view.at(relative_pos + view.get_pos());
}

void do_fov() {
  auto& view = World::get().view_space;

  view.clear_seen();
  if (get_player().as<Blob_Part>().big) {
    // Big actors see with their edge cells too so that they're not completely
    // blind in a forest style terrain.
    for (auto i : hex_dirs)
      World::get().view_space.do_fov(8, get_player().location() + i, i);
  }
  World::get().view_space.do_fov(8, get_player().location());
}

Terrain get_terrain(Location location) {
  return World::get().get_terrain(location);
}

void set_terrain(Location location, Terrain cell) {
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

void set_portal(Location location, Portal portal) {
  World::get().portal[location] = portal;
}

void clear_portal(Location location) {
  World::get().portal.erase(location);
}

std::pair<std::map<Location, Terrain>::const_iterator,
          std::map<Location, Terrain>::const_iterator>
area_locations(uint16_t area) {
  ASSERT(area != 0);
  auto i = World::get().terrain.upper_bound(Location(area - 1, 0, 0)),
    j = World::get().terrain.lower_bound(Location(area + 1, 0, 0));

  while (i->first.area < area) ++i;
  while (j->first.area > area) --j;
  ++j;
  return std::make_pair(i, j);
}

std::vector<Actor> all_actors() {
  std::vector<Actor> result;
  for (auto& i : World::get().actors)
    result.push_back(i.first);
  return result;
}

std::vector<Actor> actors_at(Location location) {
  std::vector<Actor> result;
  auto range = World::get().spatial_index.equal_range(location);
  for (auto i = range.first; i != range.second; ++i) {
    result.push_back(i->second.second);
  }
  return result;
}

std::vector<std::pair<Vec2i, Actor>> actors_with_offsets_at(Location location) {
  std::vector<std::pair<Vec2i, Actor>> result;
  auto range = World::get().spatial_index.equal_range(location);
  for (auto i = range.first; i != range.second; ++i) {
    result.push_back(i->second);
  }
  return result;
}

std::vector<Actor> actors_on(const Footprint& footprint) {
  std::vector<Actor> result;
  for (auto& pair : footprint) {
    auto range = World::get().spatial_index.equal_range(pair.second);
    for (auto i = range.first; i != range.second; ++i) {
      result.push_back(i->second.second);
    }
  }
  return result;
}

Actor new_actor(Actor_Id id) {
  auto result = Actor{id};
  ASSERT(!actor_exists(result));
  World::get().actors[result] = std::map<Kind, std::unique_ptr<Part>>();
  return result;
}

Actor new_actor() {
  return new_actor(World::get().next_actor_id++);
}

void delete_actor(Actor actor) {
  // TODO: Notify components of removal
  actor.push();
  World::get().actors.erase(actor);
}

bool actor_exists(Actor actor) {
  return assoc_contains(World::get().actors, actor);
}

Actor active_actor() {
  return World::get().active_actor();
}

void next_actor() {
  World::get().next_actor();
}

void move_view_pos(const Vec2i& offset) {
  World::get().view_space.move_pos(offset);
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
