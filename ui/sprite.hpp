/* sprite.hpp

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

#ifndef UI_SPRITE_HPP
#define UI_SPRITE_HPP

#include <util/vec.hpp>
#include <memory>

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
    if (drawable < rhs.drawable) return true;

    return false;
  }

  /// Call the `draw` method for the sprite's `Drawable` object.
  inline void draw(const Vec2f& offset) {
    drawable->draw(offset);
  }

  int z_layer;
  Vec2i pos;
  std::shared_ptr<Drawable> drawable;
};

#endif
