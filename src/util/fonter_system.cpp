/* fonter_system.cpp

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

#include "fonter_system.hpp"
#include <util/gldraw.hpp>
#include <util/surface.hpp>

Fonter_System::Fonter_System(
  const Surface& font_sheet,
  std::vector<Fonter_System::Font_Data> font_data,
  int font_height,
  int begin_char)
  : tex_dim(font_sheet.get_dim())
  , font_texture(font_sheet)
  , font_data(font_data)
  , font_height(font_height)
  , begin_char(begin_char) {}

int Fonter_System::width(const char* text) {
  int result = 0;
  for (const char* c = text; *c; c++)
    result += font_data[*c - begin_char].char_width;
  return result;
}

int Fonter_System::raw_draw(Vec2f pos, char ch) {
  // Fractions can cause artifacts in font texture drawing. Round to closest
  // integer.
  pos = Vec2f(round(pos[0]), round(pos[1]));

  font_texture.bind();
  pos[1] += height();
  auto& data = font_data[ch - begin_char];

  Vec2f offset = Vec2f(data.x_off, data.y_off) + pos;
  Vec2f origin(data.x0, data.y0);
  Vec2f dim(data.x1 - data.x0, data.y1 - data.y0);

  gl_tex_rect(Rectf(offset, dim), Rectf(origin.elem_div(tex_dim), dim.elem_div(tex_dim)));

  return data.char_width;
}

int Fonter_System::raw_draw(const Vec2f& pos, Align align, const char* text) {
  Vec2f real_pos = pos;

  switch (align) {
  case CENTER:
    real_pos[0] -= width(text) / 2;
    break;
  case RIGHT:
    real_pos[0] -= width(text);
    break;
  default:
    break;
  }

  int result = 0;
  for (const char* c = text; *c; c++)
    result += raw_draw(real_pos + Vec2f(result, 0), *c);
  return result;
}
