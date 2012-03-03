#ifndef WORLD_WORLD_HPP
#define WORLD_WORLD_HPP

#include <util.hpp>
#include <world/actor.hpp>
#include <world/location.hpp>
#include <boost/optional.hpp>
#include <exception>
#include <map>
#include <set>
#include <vector>
#include <string>


/// A proto-kind as a precursor to a full-fledged parts system.
class Blob_Part : public Part {
 public:
  static Kind s_get_kind() { return Blob_Kind; }

  Blob_Part() {}
  Blob_Part(const Location& loc, int icon, const Color& color, int power)
      : loc(loc), icon(icon), color(color), power(power), energy(0) {}
  ~Blob_Part() {}

  virtual Kind get_kind() { return s_get_kind(); }
  Location loc;
  int icon;
  Color color;
  int power;
  int energy;
 private:
  Blob_Part(const Blob_Part&);
};

/// Main container for all of the game state.
class World {
 public:
  static void init();
  static World& get();
  static void clear();

  Terrain get_terrain(const Location& location);
  void set_terrain(const Location& location, const Terrain& cell);

  /// Return the actor whose turn it is to act now.
  ///
  /// Throws Actor_Not_Found if there are no actors that can act.
  Actor active_actor();

  /// Called when done with the current active actor to move to the next one.
  void next_actor();

  // Public because I'm lazy.
  std::set<Location> explored;
  std::set<Location> seen;

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
 private:
  World();
  World(const World&);
  World& operator=(const World&);

  static std::unique_ptr<World> s_world;

  Actor previous_actor;
};


template<class T>
T& Actor::as() {
  Kind kind = T::s_get_kind();

  auto iter = World::get().actors.find(*this);
  if (iter == World::get().actors.end())
    throw Actor_Not_Found();

  auto part_iter = iter->second.find(kind);
  if (part_iter == iter->second.end())
    throw Part_Not_Found();

  Part* part = part_iter->second.get();
  T* result = dynamic_cast<T*>(part);
  // If kind doesn't match to the actual object, there's been data corruption.
  ASSERT(result != nullptr);
  return *result;
}

// Implement this on UI.
void raw_msg(std::string string);

// TODO variadics.
void msg(const char* fmt);

Actor get_player();
Location get_location(Actor actor);

bool can_enter(Actor actor, const Location& location);

bool action_walk(Actor actor, const Vec2i& dir);
bool action_melee(Actor actor, const Vec2i& dir);
bool action_bump(Actor actor, const Vec2i& dir);

void clear_seen();
void mark_seen(const Location& location);
bool is_seen(const Location& location);
bool is_explored(const Location& location);
bool blocks_sight(const Location& location);

Relative_Fov do_fov(Actor actor);

Terrain get_terrain(const Location& location);
void set_terrain(const Location& location, const Terrain& cell);

boost::optional<Portal> get_portal(const Location& location);
void set_portal(const Location& location, const Portal& portal);
void clear_portal(const Location& location);

// XXX: Return type should be considered just some iterable type, the exact
// form may change.
std::vector<Actor> all_actors();
std::vector<Actor> actors_at(const Location& location);

Actor new_actor(Actor_Id id);
Actor new_actor();
void delete_actor(Actor actor);
bool actor_exists(Actor actor);
Actor active_actor();
void next_actor();

void start_turn_update(Actor actor);

bool ready_to_act(Actor actor);

#endif
