/* font.cpp

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

#include "font.hpp"
#include "surface.hpp"
#include "core.hpp"
#include "gldraw.hpp"
#include <cstdarg>
#include <GL/gl.h>

static GLuint g_font_tex = 0;

static Surface font_image;

struct Char_Data {
  int x0, y0, x1, y1; ///< The rectangle points on the font texture
  float x_off, y_off; ///< Rendering offsets
  float char_width;
};

const int g_font_height = 13;
const int begin_char = 32;
const int num_chars = 96;

static Char_Data g_font_data[] = {
#include <font_data.hpp>
};

static uint8_t g_font_image[] = {
#include <font_image.hpp>
};

void init_font() {
  font_image.load_image(g_font_image, sizeof(g_font_image));
  g_font_tex = font_image.make_texture();
}

int draw_char(Vec2f pos, char ch) {
  if (!g_font_tex) die("Font not initialized");

  pos[1] += g_font_height;

  glBindTexture(GL_TEXTURE_2D, g_font_tex);
  auto chdata = g_font_data[ch - begin_char];

  Vec2f offset = Vec2f(chdata.x_off, chdata.y_off) + pos;
  Vec2f origin(chdata.x0, chdata.y0);
  Vec2f dim(chdata.x1 - chdata.x0, chdata.y1 - chdata.y0);
  Vec2f tex_scale = font_image.get_dim();

  gl_tex_rect(ARectf(offset, dim), ARectf(origin.elem_div(tex_scale), dim.elem_div(tex_scale)));

  return chdata.char_width;
}

int text_width(const char* text) {
  int result = 0;
  for (const char* c = text; *c; c++)
    result += g_font_data[*c - begin_char].char_width;
  return result;
}

int font_height() {
  return g_font_height;
}

int draw_text_raw(const Vec2f& pos, const char* text) {
  int result = 0;
  for (const char* c = text; *c; c++)
    result += draw_char(pos + Vec2f(result, 0), *c);
  return result;
}

// XXX: Not thread safe, using a huge static buffer to print the text in
// because too lazy to do a proper buffer.
int draw_text(const Vec2f& pos, const char* fmt, ...) {
  static char buffer[16384];
  va_list ap;
  va_start(ap, fmt);
  vsnprintf(buffer, sizeof(buffer), fmt, ap);
  return draw_text_raw(pos, buffer);
}
