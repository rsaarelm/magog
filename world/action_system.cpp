/* action_system.cpp

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

#include "action_system.hpp"
#include <util/hex.hpp>
#include <util/num.hpp>

bool Action_System::walk(Entity entity, const Vec2i& dir) {
  auto loc = spatial.location(entity);
  auto new_loc = loc + dir;
  if (spatial.can_pop(entity, new_loc)) {
    for (auto a : spatial.entities_on(spatial.footprint(entity, new_loc))) {
      if (a == entity) continue;
      // Uncrushable entities in the target area, abort movement.
      if (blocks_movement(a) && !can_crush(entity, a))
        return false;
    }

    spatial.push(entity);
    // Energy cost for movement.
    // TODO: account for terrain differences.
    auto& blob = entities.as<Blob_Part>(entity);
    blob.energy -= 100;
    blob.base_facing = vec_to_hex_dir(dir);

    // XXX Hacky. Player is tracked by the view space object. This needs to be
    // changed if we want to support multiple FOVs.
    if (is_player(entity))
      fov.move_pos(dir);

    spatial.pop(entity, new_loc);

    for (auto a : spatial.entities_on(spatial.footprint(entity, new_loc))) {
      if (a != entity && blocks_movement(a)) {
        // Crushing damages you.
        damage(entity, entities.as<Blob_Part>(a).armor);
        fx.msg("Unit crushed");
        fx.explosion(spatial.location(a), 10, Color("red"));
        kill(a);
      }
    }

    return true;
  } else {
    return false;
  }
}

bool Action_System::shoot(Entity entity, const Vec2i& dir) {
  ASSERT(is_hex_dir(dir));
  // TODO: Entities have multiple weapons. (The weapon could be the entity though.)
  const int range = 6; // TODO: Entities have different fire ranges.
  int dist = 0;
  Location start_loc = spatial.location(entity);

  auto& blob = entities.as<Blob_Part>(entity);

  if (blob.big)
    start_loc = start_loc + dir;

  Location loc = start_loc;

  for (loc = loc + dir; dist < range; loc = loc + dir) {
    dist++;

    bool hit_entity = false;
    for (auto& a : spatial.entities_at(loc)) {
      if (a != entity) {
        hit_entity = true;
        break;
      }
    }

    if (hit_entity) {
      damage(loc, entities.as<Blob_Part>(entity).damage);
      break;
    }
    if (terrain.blocks_shot(loc))
      break;
  }

  fx.beam(start_loc, dir, dist, Color("pink"));

  // Energy cost for shooting.
  blob.energy -= 100;

  blob.turret_facing = vec_to_hex_dir(dir);
}

void Action_System::wait(Entity entity) {
  entities.as<Blob_Part>(entity).energy -= 100;
}

void Action_System::damage(Location location, int amount) {
  for (auto a : spatial.entities_at(location))
    damage(a, amount);
}

void Action_System::damage(Entity entity, int amount) {
  if (entities.has(entity, Blob_Kind)) {
    auto& blob = entities.as<Blob_Part>(entity);
    blob.armor -= amount;
    if (blob.armor <= 0) {
      fx.explosion(spatial.location(entity), 10, Color("red"));
      fx.msg("Unit destroyed");
      kill(entity);
    }
  }
}

bool Action_System::is_ready(Entity entity) {
  try {
    if (is_dead(entity))
      return false;

    return entities.as<Blob_Part>(entity).energy >= 0;
  } catch (Part_Not_Found& e) {
    return false;
  }
}

bool Action_System::can_crush(Entity entity, Entity crushee) {
  return entities.as<Blob_Part>(entity).big &&
    !entities.as<Blob_Part>(crushee).big;
}

bool Action_System::blocks_movement(Entity entity) {
  return entities.has(entity, Blob_Kind);
}

void Action_System::start_turn_update(Entity entity) {
  try {
    auto& blob = entities.as<Blob_Part>(entity);
    blob.energy += blob.power;
  } catch (Part_Not_Found& e) {}
}

bool Action_System::is_player(Entity entity) {
  return entities.as<Blob_Part>(entity).faction == player_faction;
}

bool Action_System::is_enemy_of(Entity a, Entity b) {
  return entities.as<Blob_Part>(a).faction != entities.as<Blob_Part>(b).faction;
}

void Action_System::update(Entity entity) {
  // Brain-dead AI
  if (is_ready(entity)) {
    const int fov_radius = 5;

    Entity enemy = 0;
    Vec2i enemy_dir;
    bool can_fire = false;

    // XXX: Expensive. Should cache results instead of running this every frame.
    fov.run(
      fov_radius, spatial.location(entity),
      [&](const Vec2i& offset, Location loc) {
        for (auto& e : spatial.entities_at(loc)) {
          if (is_enemy_of(entity, e)) {
            if (!enemy) {
              enemy = e;
              enemy_dir = hex_dirs[vec_to_hex_dir(offset)];
            }
            if (on_hex_axis(offset) && !can_fire) {
              can_fire = true;
              enemy_dir = hex_dirs[vec_to_hex_dir(offset)];
            }
          }
        }
      });

    auto& random_dir = *rand_choice(hex_dirs);

    // TODO: Melee combat
    if (enemy) {
      if (can_fire) {
        //shoot(entity, enemy_dir);
      } else {
        // TODO: Get into firing pos AI
        walk(entity, random_dir);
      }
    } else {
      if (one_chance_in(3))
        walk(entity, random_dir);
      else
        wait(entity);
    }
  }
}

void Action_System::kill(Entity entity) {
  spatial.push(entity);
  entities.as<Blob_Part>(entity).is_dead = true;
}

bool Action_System::is_dead(Entity entity) const {
  if (!entities.exists(entity))
    return true;
  return entities.as<Blob_Part>(entity).is_dead;
}

int Action_System::count_aligned(Faction faction) const {
  int result = 0;
  for (auto& i : entities.all()) {
    if (entities.as<Blob_Part>(i).faction == faction)
      result++;
  }
  return result;
}
