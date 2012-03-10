/* tile_drawable.hpp

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

#ifndef UI_TILE_DRAWABLE_HPP
#define UI_TILE_DRAWABLE_HPP

#include "drawable.hpp"
#include <util/color.hpp>
#include <util/axis_box.hpp>
#include <GL/gl.h>

class Tile_Drawable : public Drawable {
public:
  Tile_Drawable(GLuint texture, const ARectf& tex_rect, const Vec2f& size, Color color)
    : texture(texture), tex_rect(tex_rect), size(size), color(color) {}

  virtual void draw(const Vec2f& offset);
private:
  GLuint texture;
  ARectf tex_rect;
  Vec2f size;
  Color color;
};

#endif
