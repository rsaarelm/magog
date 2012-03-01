#include "hex.hpp"
#include <xev/num.hpp>
#include <xev/util.hpp>
#include <boost/range/counting_range.hpp>
#include <boost/range/adaptor/transformed.hpp>
#include <boost/range/join.hpp>
#include <algorithm>

using namespace boost;
using namespace boost::adaptors;
using namespace std;

namespace xev {

const std::array<const Vec2i, 6> hex_dirs{{{-1, -1}, {0, -1}, {1, 0}, {1, 1}, {0, 1}, {-1, 0}}};

int hex_wall(int edge_mask) {
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
  return walls[edge_mask];
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

}
