/* fov_system.hpp

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
#ifndef WORLD_FOV_SYSTEM_HPP
#define WORLD_FOV_SYSTEM_HPP

#include <world/entities_system.hpp>
#include <world/terrain_system.hpp>
#include <world/spatial_system.hpp>
#include <world/location.hpp>
#include <util/vec.hpp>
#include <map>
#include <set>

class Fov_System {
public:
  Fov_System(
    Entities_System& entities,
    Terrain_System& terrain,
    Spatial_System& spatial)
    : entities(entities)
    , terrain(terrain)
    , spatial(spatial) {}

  bool is_seen(Location loc);
  Location view_location(const Vec2i& relative_pos);

  void do_fov(int radius, Location loc, const Vec2i& offset=Vec2i(0, 0));
  void do_fov();

  void move_pos(const Vec2i& delta) { subjective_pos += delta; }
  Vec2i get_pos() const { return subjective_pos; }

  void clear_seen() { visible.clear(); }
private:
  Fov_System(const Fov_System&);
  Fov_System& operator=(const Fov_System&);

  void prune();

  Entities_System& entities;
  Terrain_System& terrain;
  Spatial_System& spatial;

  Vec2i subjective_pos;
  std::map<Vec2i, Plain_Location> view;
  std::set<Plain_Location> visible;
};

#endif
