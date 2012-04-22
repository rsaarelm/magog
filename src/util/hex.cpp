/* hex.cpp

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

#include "hex.hpp"
#include <util/num.hpp>
#include <util/core.hpp>
#include <boost/range/counting_range.hpp>
#include <boost/range/adaptor/transformed.hpp>
#include <boost/range/join.hpp>
#include <algorithm>

using namespace boost;
using namespace boost::adaptors;
using namespace std;

Shaped_Wall hex_wall(int edge_mask) {
  // 0: Pillar, 1: x-axis wall, 2: y-axis wall, 3: xy-diagonal wall.

  // The values were determined by guesstimating what would look best for each
  // wall neighborhood. This may not be the set that provides the best-looking
  // wall approximations, and there might be a nice concise formula for this.
  const std::array<int, 64> walls{{
      0, 0, 2, 2, 1, 0, 0, 0,
      3, 3, 2, 3, 1, 3, 0, 3,
      2, 0, 2, 2, 0, 0, 2, 0,
      2, 3, 2, 0, 0, 0, 2, 2,
      1, 1, 0, 0, 1, 1, 1, 1,
      1, 0, 0, 0, 1, 0, 0, 1,
      0, 0, 2, 2, 1, 0, 0, 0,
      0, 3, 0, 2, 1, 1, 0, 0}};
  return Shaped_Wall(walls[edge_mask]);
}

int hex_circumference(int radius) {
  if (radius == 0)
    return 1;
  return radius * 6;
}

Vec2i hex_circle_vec(int radius, int index) {
  ASSERT(radius >= 0);

  if (radius == 0)
    return Vec2i(0, 0);

  int sector = mod(index, hex_circumference(radius)) / radius;
  int offset = mod(index, radius);
  return hex_dirs[sector] * radius + offset * hex_dirs[(sector + 2) % 6];
}

Range<Vec2i>::T hex_circle_points(int radius) {
  return counting_range(0, hex_circumference(radius))
      | transformed([=](int i) { return hex_circle_vec(radius, i); });
}

Range<Vec2i>::T hex_area_points(int radius) {
  if (radius == 0)
    return hex_circle_points(0);
  else
    return join(hex_circle_points(radius), hex_area_points(radius - 1));
}

int hex_dist(const Vec2i& vec) {
  if (sign(vec[0]) == sign(vec[1]))
    return max(abs(vec[0]), abs(vec[1]));
  else
    return abs(vec[0]) + abs(vec[1]);
}

bool is_hex_dir(const Vec2i& dir) {
  for (auto& i : hex_dirs) {
    if (dir == i)
      return true;
  }
  return false;
}

int hexadecant(const Vec2f& vec) {
  const float width = pi / 8;
  auto radian = atan2(vec[0], -vec[1]);
  if (radian < 0)
    radian += 2 * pi;
  return floor(radian / width);
}

int vec_to_hex_dir(const Vec2i& vec) {
  switch (hexadecant(vec)) {
  case 14:
  case 15:
    return 0;
  case 0:
  case 1:
  case 2:
  case 3:
    return 1;
  case 4:
  case 5:
    return 2;
  case 6:
  case 7:
    return 3;
  case 8:
  case 9:
  case 10:
  case 11:
    return 4;
  case 12:
  case 13:
    return 5;
  default:
    die("Bad hexadecant");
  }
}

bool on_hex_axis(const Vec2i& vec) {
  return vec[0] == 0 || vec[1] == 0 || vec[0] == vec[1];
}
