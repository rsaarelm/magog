/* intro_screen.hpp

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

#ifndef INTRO_SCREEN_HPP
#define INTRO_SCREEN_HPP

#include <util/game_state.hpp>

class Intro_Screen : public Game_State {
 public:
  Intro_Screen() {}
  virtual ~Intro_Screen() {}

  virtual void enter() {}
  virtual void exit() {}
  virtual void key_event(int keysym, int printable);
  virtual void update(float interval_seconds) {}
  virtual void draw();

 private:
};

#endif
