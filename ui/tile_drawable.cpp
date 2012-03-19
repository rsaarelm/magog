/* tile_drawable.cpp

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

#include "tile_drawable.hpp"
#include <util/gldraw.hpp>

Tile_Drawable::Tile_Drawable(
  GLuint texture, const Color& color, const Tile_Rect& tile_rect,
  const Vec2i& texture_dim,
  const Vec2f& offset)
  : texture(texture), color(color), offset_(offset) {
  Vec2f dim = texture_dim;
  Vec2f p0(tile_rect.x0, tile_rect.y0);
  Vec2f p1(tile_rect.x1, tile_rect.y1);

  texture_coords = Rectf(p0.elem_div(dim), (p1 - p0).elem_div(dim));
  draw_box = Rectf(Vec2f(tile_rect.x_off, tile_rect.y_off), p1 - p0);
}

void Tile_Drawable::draw(const Vec2f& offset) {
  glBindTexture(GL_TEXTURE_2D, texture);
  color.gl_color();
  gl_tex_rect(draw_box + offset + offset_, texture_coords);
}
