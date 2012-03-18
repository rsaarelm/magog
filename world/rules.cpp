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
  return Entity(nullptr, 1);
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
