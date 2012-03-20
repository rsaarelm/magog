/* hex.hpp

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

#ifndef UTIL_HEX_HPP
#define UTIL_HEX_HPP

/** \file hex.hpp
 * Utilities for hexagonal tiles.
 */

#include "vec.hpp"
#include "alg.hpp"
#include <array>

/**
 * The 6 hex directions, in canonical order.
 *
 * The canonical order starts at the point towards (-1, -1) and proceeds
 * clockwise from there.
 */
const std::array<const Vec2i, 6> hex_dirs{{{-1, -1}, {0, -1}, {1, 0}, {1, 1}, {0, 1}, {-1, 0}}};

enum Shaped_Wall {
  center_wall = 0,
  x_wall = 1,
  y_wall = 2,
  xy_wall = 3
};

/**
 * Pick the nicest shaped wall tile for the given set of connecting walls.
 *
 * The shaped walls and their corresponding  are
 *
 * * 0: center_wall, can represent a free-standing pillar, a corner or a junction,
 * * 1: x_wall, a wall along the x-axis,
 * * 2: y_wall, a wall along the y-axis,
 * * 3: xy_wall, a wall along the x=y line, the third axis of the hex coordinates.
 *
 * The `edge_mask` parameter is a 6-bit integer where the bits correspond to
 * the six surrounding hexes of the center hex, starting from the one at (-1,
 * -1) and moving clockwise:
 *
 *         0
 *       5   1
 *         o
 *       4   2
 *      y  3  x
 *     /       \
 *
 * The values correspond to the following surrounding wall layouts:
 *
 *     00  .    01  #    02  .    03  #    04  .    05  #    06  .    07  #
 *       .   .    .   .    .   #    .   #    .   .    .   .    .   #    .   #
 *         *        *        *        *        *        *        *        *
 *       .   .    .   .    .   .    .   .    .   #    .   #    .   #    .   #
 *         .        .        .        .        .        .        .        .
 *
 *     08  .    09  #    10  .    11  #    12  .    13  #    14  .    15  #
 *       .   .    .   .    .   #    .   #    .   .    .   .    .   #    .   #
 *         *        *        *        *        *        *        *        *
 *       .   .    .   .    .   .    .   .    .   #    .   #    .   #    .   #
 *         #        #        #        #        #        #        #        #
 *
 *     16  .    17  #    18  .    19  #    20  .    21  #    22  .    23  #
 *       .   .    .   .    .   #    .   #    .   .    .   .    .   #    .   #
 *         *        *        *        *        *        *        *        *
 *       #   .    #   .    #   .    #   .    #   #    #   #    #   #    #   #
 *         .        .        .        .        .        .        .        .
 *
 *     24  .    25  #    26  .    27  #    28  .    29  #    30  .    31  #
 *       .   .    .   .    .   #    .   #    .   .    .   .    .   #    .   #
 *         *        *        *        *        *        *        *        *
 *       #   .    #   .    #   .    #   .    #   #    #   #    #   #    #   #
 *         #        #        #        #        #        #        #        #
 *
 *     32  .    33  #    34  .    35  #    36  .    37  #    38  .    39  #
 *       #   .    #   .    #   #    #   #    #   .    #   .    #   #    #   #
 *         *        *        *        *        *        *        *        *
 *       .   .    .   .    .   .    .   .    .   #    .   #    .   #    .   #
 *         .        .        .        .        .        .        .        .
 *
 *     40  .    41  #    42  .    43  #    44  .    45  #    46  .    47  #
 *       #   .    #   .    #   #    #   #    #   .    #   .    #   #    #   #
 *         *        *        *        *        *        *        *        *
 *       .   .    .   .    .   .    .   .    .   #    .   #    .   #    .   #
 *         #        #        #        #        #        #        #        #
 *
 *     48  .    49  #    50  .    51  #    52  .    53  #    54  .    55  #
 *       #   .    #   .    #   #    #   #    #   .    #   .    #   #    #   #
 *         *        *        *        *        *        *        *        *
 *       #   .    #   .    #   .    #   .    #   #    #   #    #   #    #   #
 *         .        .        .        .        .        .        .        .
 *
 *     56  .    57  #    58  .    59  #    60  .    61  #    62  .    63  #
 *       #   .    #   .    #   #    #   #    #   .    #   .    #   #    #   #
 *         *        *        *        *        *        *        *        *
 *       #   .    #   .    #   .    #   .    #   #    #   #    #   #    #   #
 *         #        #        #        #        #        #        #        #
 *
 * The rule to determine the nicest center wall tile is an empirical look-up
 * table based on eyeballing what seems to work visually in practice.
 */
Shaped_Wall hex_wall(int edge_mask);

/**
 * Return a vector to a point on a hexagonal circle.
 *
 * `hex_circle_vec(r, i)` returns a vector pointing to one of the points on the
 * hexagonal circle of hex tiles at distance `r` from the origin. The point is
 * indexed by `i` starting at the point in the direction of vector (-1, -1) and
 * moving clockwise.
 */
Vec2i hex_circle_vec(int radius, int index);

/// Return a sequence of all hex circle points for a given radius.
Range<Vec2i>::T hex_circle_points(int radius);

/// Return all the points in a hex disk with the given radius.
Range<Vec2i>::T hex_area_points(int radius);

/**
 * Return the hex distance from origin to a point.
 *
 * The hex distance of two points is the minimum number of hex grid moves needed
 * to move from one point to the other.
 */
int hex_dist(const Vec2i& vec);

bool is_hex_dir(const Vec2i& dir);

int vec_to_hex_dir(const Vec2i& vec);

#endif
