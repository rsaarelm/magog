#ifndef UI_DRAWABLE_HPP
#define UI_DRAWABLE_HPP

#include <util/vec.hpp>

class Drawable {
public:
  virtual ~Drawable() {}

  void draw(const Vec2f& offset) = 0;
};

#endif
