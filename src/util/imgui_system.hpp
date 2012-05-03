/* imgui_system.hpp

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
#ifndef UTIL_IMGUI_SYSTEM_HPP
#define UTIL_IMGUI_SYSTEM_HPP

#include <util/fonter_system.hpp>
#include <util/vec.hpp>
#include <util/box.hpp>
#include <util/core.hpp>

#define GEN_ID (const_hash(__FILE__) + __LINE__)

class Imgui_System {
public:
  Imgui_System(
    Fonter_System& fonter)
  : fonter(fonter) {}

  void update(int mouse_x, int mouse_y, int mouse_buttons);

  bool button(long id, const char* title, const Rectf& bounds);
private:
  Imgui_System(const Imgui_System&);
  Imgui_System& operator=(const Imgui_System&);

  Fonter_System& fonter;

  struct State {
    State() : pos(0, 0), button(0) {}
    Vec2f pos;
    int button;
  };

  State state;
};

#endif
