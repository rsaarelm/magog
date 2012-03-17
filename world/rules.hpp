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

#include <world/actor.hpp>
#include <world/location.hpp>

Actor get_player();

bool blocks_shot(Location location);
bool blocks_sight(Location location);

bool blocks_movement(Actor actor);

bool action_walk(Actor actor, const Vec2i& dir);
bool action_melee(Actor actor, const Vec2i& dir);
bool action_bump(Actor actor, const Vec2i& dir);
bool action_shoot(Actor actor, const Vec2i& dir);

void damage(Location location, int amount);
void damage(Actor actor, int amount);

bool has_actors(Location location);

void start_turn_update(Actor actor);
bool ready_to_act(Actor actor);

#endif
