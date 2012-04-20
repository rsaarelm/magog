/* cycler_system.hpp

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
#ifndef WORLD_CYCLER_SYSTEM_HPP
#define WORLD_CYCLER_SYSTEM_HPP

#include <world/entities_system.hpp>
#include <world/action_system.hpp>

class Cycler_System {
public:
  Cycler_System(
    Entities_System& entities,
    Spatial_System& spatial,
    Action_System& action)
    : entities(entities)
    , spatial(spatial)
    , action(action)
    , state(state_starting)
    , current_entity(0) {}

  /// Runs one update cycle, returns when it encounters a player entity or
  /// goes through an entire entity cycle without seeing any player entities.
  void run();

  /// Returns the current player entity or 0, if at the beginning of a cycle.
  Entity current_player() const;

private:
  Entities_System& entities;
  Spatial_System& spatial;
  Action_System& action;

  enum State {
    state_starting,
    state_had_player,
    state_no_player
  };

  State state;
  Entity current_entity;
};

#endif
