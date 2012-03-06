#ifndef UTIL_GEOM_UTIL_HPP
#define UTIL_GEOM_UTIL_HPP

/** \file geom_util.hpp
 * Geometric utilities.
 */

#include <functional>
#include "vec.hpp"

void line(
    const Vec2i& p0,
    const Vec2i& p1,
    std::function<void(const Vec<int, 2>&)> fn);

void filled_triangle(
    const Vec2i& p0,
    const Vec2i& p1,
    const Vec2i& p2,
    std::function<void(const Vec2i&)> fn);

#endif
