// Copyright (C) 2012 Risto Saarelma

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
