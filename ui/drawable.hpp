// Copyright (C) 2012 Risto Saarelma

#ifndef UI_DRAWABLE_HPP
#define UI_DRAWABLE_HPP

#include <util/vec.hpp>

class Drawable {
public:
  virtual ~Drawable() {}

  /// Update the Drawable's state, return whether the Drawable is still alive
  /// after this.
  virtual bool update(float interval_sec) { return true; }

  virtual void draw(const Vec2f& offset) = 0;

  virtual int get_z_layer() const { return 0; }
};

#endif
