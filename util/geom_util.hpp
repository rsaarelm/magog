/* geom_util.hpp

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
