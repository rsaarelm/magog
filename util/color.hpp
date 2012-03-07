// Copyright (C) 2012 Risto Saarelma

#ifndef UTIL_COLOR_HPP
#define UTIL_COLOR_HPP

#include "vec.hpp"
#include <GL/glew.h>
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

  void gl_color() const {
    glColor4ub(r, g, b, a);
  }

  /// Cast the color value into the corresponding 4-element vector.
  const Color_Vec& as_vec() const { return *reinterpret_cast<const Color_Vec*>(this); }

  /// Cast the color value into the corresponding 4-element vector.
  Color_Vec& as_vec() { return *reinterpret_cast<Color_Vec*>(this); }

  Color_Elt r, g, b, a;
};

#endif
