/* action_system.hpp

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
#ifndef WORLD_ACTION_SYSTEM_HPP
#define WORLD_ACTION_SYSTEM_HPP

#include <world/entities_system.hpp>
#include <world/terrain_system.hpp>
#include <world/spatial_system.hpp>
#include <world/fov_system.hpp>
#include <world/fx_system.hpp>
#include <world/parts.hpp>
#include <util/vec.hpp>

class Action_System {
public:
  Action_System(
    Entities_System& entities,
    Terrain_System& terrain,
    Spatial_System& spatial,
    Fov_System& fov,
    Fx_System& fx)
  : entities(entities)
  , terrain(terrain)
  , spatial(spatial)
  , fov(fov)
  , fx(fx) {}

  bool walk(Entity entity, const Vec2i& dir);

  // Return whether an attack was made, not whether it was successful.
  bool melee(Entity entity, const Vec2i& dir);

  bool bump(Entity entity, const Vec2i& dir);
  bool shoot(Entity entity, const Vec2i& dir);

  void wait(Entity entity);

  void damage(Location location, int amount);
  void damage(Entity entity, int amount);

  bool is_ready(Entity entity);

  bool can_crush(Entity entity, Entity crushee);
  bool blocks_movement(Entity entity);

  void start_turn_update(Entity entity);

  bool is_player(Entity entity);
  bool is_enemy_of(Entity a, Entity b);

  Entity mob_at(Location location);

  void update(Entity entity);

  void kill(Entity entity);
  bool is_dead(Entity entity) const;

  int count_aligned(Faction faction) const;
private:
  Entities_System& entities;
  Terrain_System& terrain;
  Spatial_System& spatial;
  Fov_System& fov;
  Fx_System& fx;
};

#endif
