#ifndef UTIL_GLDRAW_HPP
#define UTIL_GLDRAW_HPP

/** \file gldraw.hpp
 * OpenGL drawing utilities.
 */

#include "axis_box.hpp"

void gl_rect(const ARectf& box);

void gl_tex_rect(const ARectf& box, const ARectf& texcoords);

#endif
