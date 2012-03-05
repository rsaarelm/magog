#include "tile_drawable.hpp"
#include <util/gldraw.hpp>

void Tile_Drawable::draw(const Vec2f& offset) {
  glBindTexture(GL_TEXTURE_2D, texture);
  color.gl_color();
  gl_tex_rect(ARectf(offset, size), tex_rect);
}
