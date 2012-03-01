#ifndef XEV_COLOR_HPP
#define XEV_COLOR_HPP

#include <xev/vec.hpp>
#include <GL/glew.h>
#include <stdexcept>

namespace xev {

struct Color {
  typedef unsigned char Color_Elt;
  typedef Vec<Color_Elt, 4> Color_Vec;

  Color() : r(0), g(0), b(0), a(0) {}

  Color(Color_Elt r, Color_Elt g, Color_Elt b, Color_Elt a=255)
      : r(r), g(g), b(b), a(a) {}

  Color(const char* desc);

  void gl_color() const {
    glColor4ub(r, g, b, a);
  }

  const Color_Vec& as_vec() const { return *reinterpret_cast<const Color_Vec*>(this); }

  Color_Vec& as_vec() { return *reinterpret_cast<Color_Vec*>(this); }

  Color_Elt r, g, b, a;
};

}

#endif
