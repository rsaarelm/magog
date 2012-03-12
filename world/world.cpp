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
#include <stdexcept>

void msg(const char* fmt) {
  raw_msg(fmt);
}

Actor get_player() {
  // TODO: Assert that the actor is registered in World.

  // XXX: Fixed ID is problematic if we want to switch the player entity
  // around.
  return Actor(1);
}


std::unique_ptr<World> World::s_world;

World& World::get() {
  if (s_world.get() == nullptr)
    s_world.reset(new World());
  return *s_world.get();
}

void World::clear() {
  s_world.reset(nullptr);
}

World::World()
    : next_actor_id(256) // IDs below this are reserved for fixed stuff.
    , previous_actor(-1)
{}

bool can_enter(Actor actor, Location location) {
  auto kind = terrain_data[get_terrain(location)].kind;
  return kind == open_terrain;
}

bool blocks_shot(Location location) {
  return terrain_data[get_terrain(location)].kind == wall_terrain;
}

bool action_walk(Actor actor, const Vec2i& dir) {
  auto loc = actor.location();
  auto new_loc = loc + dir;
  if (actor.can_pop(new_loc)) {
    actor.push();

    // XXX Hacky. Player is tracked by the view space object.
    if (actor == get_player())
      World::get().view_space.move_pos(dir);

    for (auto a : actors_on(actor.footprint(new_loc))) {
      msg("Crush!");
      delete_actor(a);
    }
    actor.pop(new_loc);
    // Energy cost for movement.
    // TODO: account for terrain differences.
    actor.as<Blob_Part>().energy -= 100;
    return true;
  } else {
    return false;
  }
}

bool action_shoot(Actor actor, const Vec2i& dir) {
  ASSERT(is_hex_dir(dir));
  // TODO: Actors have multiple weapons. (The weapon could be the actor though.)
  const int range = 6; // TODO: Actors have different fire ranges.
  int dist = 0;
  Location loc = actor.location();

  for (loc = loc + dir; dist < range; loc = loc + dir) {
    dist++;
    bool hit_actor = false;
    for (auto& a : actors_at(loc)) {
      if (a != actor) {
        hit_actor = true;
        break;
      }
    }

    if (hit_actor) {
      msg("Zap!");
      damage(loc);
      explosion_fx(loc);
      break;
    }
    if (blocks_shot(loc))
      break;
  }

  beam_fx(actor.location(), dir, dist, Color("pink"));

  auto& blob = actor.as<Blob_Part>();
  // Energy cost for shooting.
  blob.energy -= 100;
}

void damage(Location location) {
  // TODO, lots more detail
  for (auto a : actors_at(location)) {
    delete_actor(a);
  }
}

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

bool blocks_sight(Location location) {
  auto kind = terrain_data[get_terrain(location)].kind;
  return kind == wall_terrain || kind == void_terrain;
}

boost::optional<Location> view_space_location(const Vec2i& relative_pos) {
  auto& view = World::get().view_space;
  return view.at(relative_pos + view.get_pos());
}

void do_fov() {
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

std::vector<Location> area_locations(uint16_t area) {
  ASSERT(area != 0);
  std::vector<Location> result;
  for (auto i = World::get().terrain.upper_bound(Location(area - 1, 0, 0)),
         j = World::get().terrain.lower_bound(Location(area + 1, 0, 0));
       i != j;
       i++) {
    if (i->first.area == area) {
      result.push_back(i->first);
    }
  }
  return result;
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

bool has_actors(Location location) {
  for (auto a : actors_at(location))
    return true;
  return false;
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

void start_turn_update(Actor actor) {
  try {
    auto& blob = actor.as<Blob_Part>();
    blob.energy += blob.power;
  } catch (Part_Not_Found& e) {}
}

bool ready_to_act(Actor actor) {
  try {
    return actor.as<Blob_Part>().energy >= 0;
  } catch (Part_Not_Found& e) {
    return false;
  }
}
