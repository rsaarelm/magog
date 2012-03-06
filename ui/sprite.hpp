#ifndef UI_SPRITE_HPP
#define UI_SPRITE_HPP

#include <util/vec.hpp>

class Drawable;

/**
 * A lightweight handle for `Drawable` objects.
 *
 * Sprites are used to collate the drawables that are displayed during one
 * frame. They are intended to be inserted into a sorted and
 * duplicate-removing container, such as a `std::set`. The insertion will
 * remove duplicate draws where the same (by memory address identity)
 * `Drawable` is commanded to be drawn several times in the same place, while
 * allowing the same `Drawable` to be drawn into several different positions
 * for the same frame. The sort stage will also provide a draw order for the
 * sprites as they are sorted according to their `z_layer` field.
 */
struct Sprite {
  inline bool operator<(const Sprite& rhs) const {
    // Function to tell sprites apart, with a bonus for doing draw layers.

    // XXX: A generalized heterogeneous lexicographical compare function would
    // be nice here.

    // First sort by layer.
    if (z_layer < rhs.z_layer) return true;
    if (rhs.z_layer < z_layer) return false;

    // Then distinguish by position.
    if (pos < rhs.pos) return true;
    if (rhs.pos < pos) return false;

    // Finally use memory address of drawable as tie-breaker.
    if (reinterpret_cast<size_t>(&drawable) < reinterpret_cast<size_t>(&rhs.drawable)) return true;

    return false;
  }

  /// Call the `draw` method for the sprite's `Drawable` object.
  inline void draw(const Vec2f& offset) {
    drawable.draw(offset);
  }

  int z_layer;
  Vec2i pos;
  Drawable& drawable;
};

#endif
