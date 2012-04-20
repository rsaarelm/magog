/* game_state.hpp

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

#ifndef UTIL_GAME_STATE_HPP
#define UTIL_GAME_STATE_HPP

/**
 * Interface for describing a specific state of a game application, such as
 * the intro screen or the main gameplay stage.
 */
class Game_State {
 public:
  Game_State() {}

  virtual ~Game_State() {}

  virtual void enter() {}
  virtual void exit() {}

  // Key release events get negative keycodes with the absolute value of the
  // released key's keycode.
  virtual void key_event(int keycode, int printable, int scancode) {}
  virtual void mouse_event(int x, int y, int buttons) {}
  virtual void resize_event(int width, int height) {}

  virtual void update(float interval_seconds) = 0;
  virtual void draw() = 0;
 private:
};

#endif
