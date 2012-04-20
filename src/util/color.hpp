/* color.hpp

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

#ifndef UTIL_COLOR_HPP
#define UTIL_COLOR_HPP

#include "vec.hpp"
#include <GL/gl.h>
#include <stdexcept>

/// Color type.
struct Color {
  typedef uint8_t Color_Elt;
  typedef Vec<Color_Elt, 4> Color_Vec;

  Color() : r(0), g(0), b(0), a(0) {}

  Color(Color_Elt r, Color_Elt g, Color_Elt b, Color_Elt a=255)
      : r(r), g(g), b(b), a(a) {}

  /**
   * Initialize a color using a string description
   *
   * The string can be of the form "#RGB" or "#RRGGBB", where R, G and B are
   * hexadecimal color values. The single-digit versions are expanded into
   * double-digit ones by repeating the digit, so that "#73b" becomes
   * "#7733bb".
   *
   * The string can also be lowercase space-separated X11 color name, eg.
   * "alice blue" or "cornsilk".
   */
  Color(const char* desc);

  Color monochrome() const {
    Color_Elt i = .2989 * r + .5870 * g + .1140 * b;
    return Color(i, i, i, a);
  }

  void gl_color() const {
    glColor4ub(r, g, b, a);
  }

  /// Cast the color value into the corresponding 4-element vector.
  const Color_Vec& as_vec() const { return *reinterpret_cast<const Color_Vec*>(this); }

  /// Cast the color value into the corresponding 4-element vector.
  Color_Vec& as_vec() { return *reinterpret_cast<Color_Vec*>(this); }

  Color_Elt r, g, b, a;
};

const Color& as_color(const Color::Color_Vec& vec);

Color lerp(float f, const Color& c1, const Color& c2);

#endif
