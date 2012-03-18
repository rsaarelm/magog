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
#include <world/rules.hpp>
#include <world/world.hpp>
#include <world/parts.hpp>
#include <world/effects.hpp>

bool Action_System::walk(Entity entity, const Vec2i& dir) {
  auto loc = entity.location();
  auto new_loc = loc + dir;
  if (spatial.can_pop(entity, new_loc)) {
    for (auto a : spatial.entities_on(spatial.footprint(entity, new_loc))) {
      if (a == entity) continue;
      // Uncrushable entities in the target area, abort movement.
      if (blocks_movement(a) && !can_crush(entity, a))
        return false;
    }

    spatial.push(entity);

    // XXX Hacky. Player is tracked by the view space object.
    if (entity == get_player())
      fov.move_view_pos(dir);

    for (auto a : spatial.entities_on(spatial.footprint(entity, new_loc))) {
      if (blocks_movement(a)) {
        // Crushing damages you.
        damage(entity, a.as<Blob_Part>().armor / 2);
        msg("Crush!");
        delete_entity(a);
      }
    }
    spatial.pop(entity, new_loc);
    // Energy cost for movement.
    // TODO: account for terrain differences.
    entity.as<Blob_Part>().energy -= 100;
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
  Location loc = entity.location();

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
      msg("Zap!");
      damage(loc, entity.as<Blob_Part>().damage);
      break;
    }
    if (terrain.blocks_shot(loc))
      break;
  }

  beam_fx(entity.location(), dir, dist, Color("pink"));

  auto& blob = entity.as<Blob_Part>();
  // Energy cost for shooting.
  blob.energy -= 100;
}

void Action_System::damage(Location location, int amount) {
  for (auto a : spatial.entities_at(location))
    damage(a, amount);
}

void Action_System::damage(Entity entity, int amount) {
  if (entity.has<Blob_Part>()) {
    auto& blob = entity.as<Blob_Part>();
    blob.armor -= amount;
    if (blob.armor <= 0) {
      explosion_fx(entity.location());
      delete_entity(entity);
    }
  }
}

bool Action_System::is_ready(Entity entity) {
  try {
    return entity.as<Blob_Part>().energy >= 0;
  } catch (Part_Not_Found& e) {
    return false;
  }
}
