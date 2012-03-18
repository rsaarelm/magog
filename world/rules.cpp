/* rules.cpp

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

#include "rules.hpp"
#include <world/world.hpp>
#include <world/effects.hpp>
#include <world/parts.hpp>
#include <util/num.hpp>

Entity get_player() {
  // TODO: Assert that the entity is registered in World.

  // XXX: Fixed ID is problematic if we want to switch the player entity
  // around.
  return Entity(1);
}

/// Add results of four dice which can give -1, 0 or 1 with equal
/// probabilities. Result is distributed in a crude approximation of a normal
/// distribution.
int fudge_roll() {
  int result = 0;
  for (int i = 0; i < 4; i++)
    result += rand_int(3) - 1;
  return result;
}

bool blocks_shot(Location location) {
  auto kind = terrain_data[get_terrain(location)].kind;
  return kind == wall_terrain || kind == curtain_terrain;
}

bool blocks_sight(Location location) {
  auto kind = terrain_data[get_terrain(location)].kind;
  return kind == wall_terrain || kind == void_terrain || kind == curtain_terrain;
}

bool can_crush(Entity entity, Entity crushee) {
  return entity.as<Blob_Part>().big && !crushee.as<Blob_Part>().big;
}

bool blocks_movement(Entity entity) {
  return entity.has<Blob_Part>();
}

bool action_walk(Entity entity, const Vec2i& dir) {
  auto loc = entity.location();
  auto new_loc = loc + dir;
  if (entity.can_pop(new_loc)) {
    for (auto a : entities_on(entity.footprint(new_loc))) {
      if (a == entity) continue;
      // Uncrushable entities in the target area, abort movement.
      if (blocks_movement(a) && !can_crush(entity, a))
        return false;
    }

    entity.push();

    // XXX Hacky. Player is tracked by the view space object.
    if (entity == get_player())
      move_view_pos(dir);

    for (auto a : entities_on(entity.footprint(new_loc))) {
      if (blocks_movement(a)) {
        // Crushing damages you.
        damage(entity, a.as<Blob_Part>().armor / 2);
        msg("Crush!");
        delete_entity(a);
      }
    }
    entity.pop(new_loc);
    // Energy cost for movement.
    // TODO: account for terrain differences.
    entity.as<Blob_Part>().energy -= 100;
    return true;
  } else {
    return false;
  }
}

bool action_shoot(Entity entity, const Vec2i& dir) {
  ASSERT(is_hex_dir(dir));
  // TODO: Entities have multiple weapons. (The weapon could be the entity though.)
  const int range = 6; // TODO: Entities have different fire ranges.
  int dist = 0;
  Location loc = entity.location();

  for (loc = loc + dir; dist < range; loc = loc + dir) {
    dist++;

    bool hit_entity = false;
    for (auto& a : entities_at(loc)) {
      if (a != entity) {
        hit_entity = true;
        break;
      }
    }

    if (hit_entity) {
      msg("Zap!");
      damage(loc, entity.as<Blob_Part>().damage);
      break;
    }
    if (blocks_shot(loc))
      break;
  }

  beam_fx(entity.location(), dir, dist, Color("pink"));

  auto& blob = entity.as<Blob_Part>();
  // Energy cost for shooting.
  blob.energy -= 100;
}

void damage(Location location, int amount) {
  for (auto a : entities_at(location))
    damage(a, amount);
}

void damage(Entity entity, int amount) {
  if (entity.has<Blob_Part>()) {
    auto& blob = entity.as<Blob_Part>();
    blob.armor -= amount;
    if (blob.armor <= 0) {
      explosion_fx(entity.location());
      delete_entity(entity);
    }
  }
}

bool has_entities(Location location) {
  for (auto a : entities_at(location))
    return true;
  return false;
}

void start_turn_update(Entity entity) {
  try {
    auto& blob = entity.as<Blob_Part>();
    blob.energy += blob.power;
  } catch (Part_Not_Found& e) {}
}

bool ready_to_act(Entity entity) {
  try {
    return entity.as<Blob_Part>().energy >= 0;
  } catch (Part_Not_Found& e) {
    return false;
  }
}
