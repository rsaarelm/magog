#include "world.hpp"
#include "fov.hpp"
#include <stdexcept>

using namespace xev;

void msg(const char* fmt) {
  raw_msg(fmt);
}

Actor get_player() {
  // TODO: Assert that the actor is registered in World.

  // XXX: Fixed ID is problematic if we want to switch the player entity
  // around.
  return Actor(1);
}

Location get_location(Actor actor) {
  return actor.as<Blob_Part>().loc;
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

bool can_enter(Actor actor, const Location& location) {
  // TODO: Proper walkability info in terrain data.
  return get_terrain(location).icon == 5;
}

bool action_walk(Actor actor, const Vec2i& dir) {
  auto loc = get_location(actor);
  auto new_loc = loc + dir + get_portal(loc + dir);
  if (can_enter(actor, new_loc)) {
    for (auto a : actors_at(new_loc)) {
      // TODO: Msg writing, inform that crushination takes place here.
      msg("Crush!");
      delete_actor(a);
    }
    auto& blob = actor.as<Blob_Part>();
    blob.loc = new_loc;
    // Energy cost for movement.
    // TODO: account for terrain differences.
    blob.energy -= 100;
    return true;
  } else {
    return false;
  }
}

Terrain World::get_terrain(const Location& location) {
  static Terrain solid_terrain {1, "wall", Color(196, 196, 196)};

  return assoc_find_or(terrain, location, solid_terrain);
}

void World::set_terrain(const Location& location, const Terrain& cell) {
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


void clear_seen() {
  World::get().seen.clear();
}

void mark_seen(const Location& location) {
  World::get().seen.insert(location);
  World::get().explored.insert(location);
}

bool is_seen(const Location& location) {
  return World::get().seen.count(location) > 0;
}

bool is_explored(const Location& location) {
  return World::get().explored.count(location) > 0;
}

bool blocks_sight(const Location& location) {
  // TODO: Do this properly.
  return get_terrain(location).icon != 5;
}

Relative_Fov do_fov(Actor actor) {
  clear_seen();
  return hex_field_of_view(8, get_location(actor));
}

Terrain get_terrain(const Location& location) {
  return World::get().get_terrain(location);
}

void set_terrain(const Location& location, const Terrain& cell) {
  World::get().set_terrain(location, cell);
}

boost::optional<Portal> get_portal(const Location& location) {
  return assoc_find(World::get().portal, location);
}

void set_portal(const Location& location, const Portal& portal) {
  World::get().portal[location] = portal;
}

void clear_portal(const Location& location) {
  World::get().portal.erase(location);
}

std::vector<Actor> all_actors() {
  std::vector<Actor> result;
  for (auto& i : World::get().actors)
    result.push_back(i.first);
  return result;
}

std::vector<Actor> actors_at(const Location& location) {
  std::vector<Actor> result;
  for (auto& i : all_actors()) {
    if (get_location(i) == location)
      result.push_back(i);
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
