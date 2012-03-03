#ifndef XEV_GEOM_UTIL_HPP
#define XEV_GEOM_UTIL_HPP

#include <functional>
#include "vec.hpp"

namespace xev {

void line(
    const Vec2i& p0,
    const Vec2i& p1,
    std::function<void(const Vec<int, 2>&)> fn);

void filled_triangle(
    const Vec2i& p0,
    const Vec2i& p1,
    const Vec2i& p2,
    std::function<void(const Vec2i&)> fn);
}

#endif
