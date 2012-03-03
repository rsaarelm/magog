#ifndef UTIL_TRANSFORM_HPP
#define UTIL_TRANSFORM_HPP

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
