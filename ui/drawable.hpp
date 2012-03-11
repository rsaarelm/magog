/* drawable.hpp

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

#ifndef UI_DRAWABLE_HPP
#define UI_DRAWABLE_HPP

#include <util/vec.hpp>
#include <world/location.hpp>
#include <map>

class Drawable {
public:
  virtual ~Drawable() {}

  /// Update the Drawable's state, return whether the Drawable is still alive
  /// after this.
  virtual bool update(float interval_sec) { return true; }

  virtual void draw(const Vec2f& offset) = 0;

  virtual int get_z_layer() const { return 0; }

  virtual Footprint footprint(const Location& start) const {
    Footprint result;
    result[Vec2i(0, 0)] = start;
    return result;
  }
};

#endif
