/* cycler_system.cpp

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

#include "cycler_system.hpp"

void Cycler_System::run() {
  for (;;) {
    try {
      Entity next = entities.entity_after(current_entity);

      if (next < current_entity) {
        // Rollover.
        spatial.destroy_pushed();
        current_entity = 0;

        switch (state) {
        case state_starting:
        case state_no_player:
          state = state_no_player;
          return;
        case state_had_player:
          // Don't return at rollover if already returned at player.
          state = state_no_player;
          break;
        }
      }

      current_entity = next;

      if (action.is_dead(current_entity))
        continue;

      action.start_turn_update(current_entity);

      if (action.is_player(current_entity)) {
        // Player entity encountered, note this in state.
        state = state_had_player;
        if (action.is_ready(current_entity)) {
          // Player entity is ready to act, let the toplevel do its stuff.
          return;
        }
      } else {
        action.update(current_entity);
      }
    } catch (Entity_Exception& e) {
      // No entities.
      return;
    }
  }
}

Entity Cycler_System::current_player() const {
  if (action.is_dead(current_entity))
    return Entity(0);
  return current_entity;
}
