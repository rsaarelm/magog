/* imgui_system.cpp

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

#include "imgui_system.hpp"
#include <util/gldraw.hpp>
#include <util/color.hpp>

void Imgui_System::update(int mouse_x, int mouse_y, int mouse_buttons) {
  state.pos = Vec2f(mouse_x, mouse_y);
  state.button = mouse_buttons;
}

bool Imgui_System::button(int id, const char* title, const Rectf& bounds) {
  glBindTexture(GL_TEXTURE_2D, 0);

  bool hit = bounds.contains(state.pos);
  if (hit) {
    glColor4f(0, .50, 0, 1);
  } else {
    glColor4f(0, .25, 0, 1);
  }
  gl_rect(bounds);

  Vec2f dim(fonter.width(title), fonter.height());
  Vec2f centering_tweak = Vec2f(0, -fonter.height() / 4);
  Vec2f pos = bounds.min() + (bounds.dim() - dim) / 2.f + centering_tweak;
  Color(255, 255, 255).gl_color();
  fonter.draw(pos, title);
  return hit && state.button;
}
