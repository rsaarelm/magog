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
#include <world/actor.hpp>
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


/// A proto-kind as a precursor to a full-fledged parts system.
class Blob_Part : public Part {
 public:
  friend class Actor;
  static Kind s_get_kind() { return Blob_Kind; }

  Blob_Part() {}
  Blob_Part(Actor_Icon icon, int power, bool big=false)
    : icon(icon), power(power), energy(0), big(big) {}
  ~Blob_Part() {}

  virtual Kind get_kind() { return s_get_kind(); }
  Actor_Icon icon;
  int power;
  int energy;
  bool big; // XXX: Very crude, should have a more complex size system.
 private:
  Location loc;
  Blob_Part(const Blob_Part&);
  Blob_Part& operator=(const Blob_Part&);
};

Part* find_part(Actor actor, Kind kind);
void add_part(Actor actor, std::unique_ptr<Part> new_part);

template<class T>
T& Actor::as() const {
  Part* part = find_part(*this, T::s_get_kind());

  T* result = dynamic_cast<T*>(part);
  // If kind doesn't match to the actual object, there's been data corruption.
  ASSERT(result != nullptr);
  return *result;
}

void clear_world();

Spatial_Index<Actor>& get_spatial_index();

// TODO variadics.
void msg(const char* fmt);

bool is_seen(Location location);
boost::optional<Location> view_space_location(const Vec2i& relative_pos);
void do_fov();

Terrain get_terrain(Location location);
void set_terrain(Location location, Terrain cell);

Portal get_portal(Location location);
void set_portal(Location location, Portal portal);
void clear_portal(Location location);

std::pair<std::map<Location, Terrain>::const_iterator,
          std::map<Location, Terrain>::const_iterator>
area_locations(uint16_t area);

// XXX: Return type should be considered just some iterable type, the exact
// form may change.
std::vector<Actor> all_actors();
std::vector<Actor> actors_at(Location location);
std::vector<std::pair<Vec2i, Actor>> actors_with_offsets_at(Location location);
std::vector<Actor> actors_on(const Footprint& footprint);

Actor new_actor(Actor_Id id);
Actor new_actor();
void delete_actor(Actor actor);
bool actor_exists(Actor actor);
Actor active_actor();
void next_actor();
Actor actor_after(Actor actor);

void move_view_pos(const Vec2i& offset);

#endif
