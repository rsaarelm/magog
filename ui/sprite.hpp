#ifndef UI_SPRITE_HPP
#define UI_SPRITE_HPP

#include <util/vec.hpp>

class Drawable;

struct Sprite {
  inline bool operator<(const Sprite& rhs) const {
    // Function to tell sprites apart, with a bonus for doing draw layers.

    // XXX: A generalized heterogeneous lexicographical compare function would
    // be nice here.

    // First sort by layer.
    if (z_layer < rhs.z_layer) return true;
    if (z_layer > rhs.z_layer) return false;

    // Then distinguish by position.
    if (pos < rhs.pos) return true;
    if (pos > rhs.pos) return false;

    // Finally use memory address of drawable as tie-breaker.
    if (reinterpret_cast<size_t>(&drawable) < reinterpret_cast<size_t>(&rhs.drawable)) return true;

    return false;
  }

  inline void draw(const Vec2f& offset) {
    drawable.draw(offset);
  }

  int z_layer;
  Vec2i pos;
  Drawable& drawable;
};

#endif
