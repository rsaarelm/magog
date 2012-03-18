/* rules.hpp

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
#ifndef WORLD_RULES_HPP
#define WORLD_RULES_HPP

/// \file rules.hpp \brief Various operations on game state

#include <world/entity.hpp>
#include <world/location.hpp>

Entity get_player();

bool blocks_shot(Location location);
bool blocks_sight(Location location);

bool blocks_movement(Entity entity);

bool action_walk(Entity entity, const Vec2i& dir);
bool action_melee(Entity entity, const Vec2i& dir);
bool action_bump(Entity entity, const Vec2i& dir);
bool action_shoot(Entity entity, const Vec2i& dir);

void damage(Location location, int amount);
void damage(Entity entity, int amount);

bool has_entities(Location location);

void start_turn_update(Entity entity);
bool ready_to_act(Entity entity);

#endif
