/* transform.hpp

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

#ifndef UTIL_TRANSFORM_HPP
#define UTIL_TRANSFORM_HPP

/** \file transform.hpp
 * 3D graphics transformations for OpenGL.
 */

#include "vec.hpp"
#include "mtx.hpp"

template<class C, int N> class Vec;
template<class C, int C, int R> class Mtx;

typedef Mtx<float, 4, 4> Gl_Matrix;

typedef Vec<float, 4> Quaternion;

Gl_Matrix frustum(
    float l, float r,
    float b, float t,
    float n, float f);

Gl_Matrix ortho(
    float l, float r,
    float b, float t,
    float n, float f);

Gl_Matrix perspective(
    float v_fov, float aspect,
    float z_near, float z_far);

Gl_Matrix translation(const Vec3f& delta);

Gl_Matrix rotation(const Vec3f& axis, float angle);

Gl_Matrix rotation(const Quaternion& q);

#endif
