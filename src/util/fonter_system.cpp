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

#define STB_TRUETYPE_IMPLEMENTATION
#include <contrib/stb/stb_truetype.h>

Fonter_System::Fonter_System(
  File_System& file,
  const char* ttf_file,
  int font_height,
  int first_char,
  int num_chars)
  : file(file)
  , font_height(font_height)
  , first_char(first_char) {
  load_font(ttf_file, font_height, first_char, num_chars);
}

int Fonter_System::width(const char* text) {
  int result = 0;
  for (const char* c = text; *c; c++)
    result += font_data[*c - first_char].char_width;
  return result;
}

int Fonter_System::raw_draw(Vec2f pos, char ch) {
  // Fractions can cause artifacts in font texture drawing. Round to closest
  // integer.
  pos = Vec2f(round(pos[0]), round(pos[1]));

  font_texture.bind();
  pos[1] += height();
  auto& data = font_data[ch - first_char];

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

void Fonter_System::load_font(const char* filename, int height, int first, int num) {
  std::vector<uint8_t> ttf_file = file.read(filename);
  std::vector<stbtt_bakedchar> chardata;
  chardata.resize(num);

  int pixel_estimate = height * height * num;
  // Get the smallest power-of-two square texture size that fits the
  // guesstimated number of pixels.
  int dim = 1;
  while (dim * dim < pixel_estimate)
    dim <<= 1;

  std::vector<uint8_t> pixels;
  int error = 0;

  for (int i = 0; i < 2; i++) {
    pixels.resize(dim * dim);
    error = stbtt_BakeFontBitmap(
      ttf_file.data(), 0, height,
      pixels.data(), dim, dim,
      first, num, chardata.data());
    if (error <= 0)
      // Canvas too small, try doubling.
      dim <<= 1;
    else
      break;
  }

  // XXX: Should throw exception here, since trying to parse data from the
  // outside.
  ASSERT(error >= 0);

  Surface surf(dim, dim);
  uint8_t* dst = surf.data();
  uint8_t* src = pixels.data();

  for (int i = 0; i < dim*dim; i++) {
    // 8-bit to 32-bit.
    *dst++ = *src;
    *dst++ = *src;
    *dst++ = *src;
    *dst++ = *src++;
  }

  font_texture = Gl_Texture(surf);
  tex_dim = Vec2i(dim, dim);

  for (auto& a : chardata)
    font_data.push_back(Font_Data{a.x0, a.y0, a.x1, a.y1, a.xoff, a.yoff, a.xadvance});
}
