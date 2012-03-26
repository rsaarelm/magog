/* gldraw.hpp

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

#ifndef UTIL_GLDRAW_HPP
#define UTIL_GLDRAW_HPP

/** \file gldraw.hpp
 * OpenGL drawing utilities.
 */

#include <util/box.hpp>
#include <GL/gl.h>

class Surface;

GLuint make_texture(const Surface& surface);

void gl_rect(const Rectf& box);

void gl_tex_rect(const Rectf& box, const Rectf& texcoords);

#endif
