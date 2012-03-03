#include "geom_util.hpp"
#include <algorithm>

using namespace std;

void line(
    const Vec2i& p0,
    const Vec2i& p1,
    std::function<void(const Vec2i&)> fn) {
  // XXX: The naive, floating-point solution. Feel free to replace with Bresenham or something.
  auto d = p1 - p0;
  int n = abs(d[0]) > abs(d[1]) ? abs(d[0]) : abs(d[1]);
  Vec2f step(d[0], d[1]), pos(p0[0], p0[1]);
  step /= n;
  for (int i = 0; i <= n; i++) {
    fn(Vec2i(pos[0], pos[1]));
    pos += step;
  }
}

void filled_triangle(
    const Vec2i& p0,
    const Vec2i& p1,
    const Vec2i& p2,
    std::function<void(const Vec2i&)> fn) {
  // XXX: Naive implementation.
  float x0 = p0[0], x1 = p1[0], x2 = p2[0];
  float y0 = p0[1], y1 = p1[1], y2 = p2[1];
  int minx = min(p0[0], min(p1[0], p2[0]));
  int miny = min(p0[1], min(p1[1], p2[1]));
  int maxx = max(p0[0], max(p1[0], p2[0]));
  int maxy = max(p0[1], max(p1[1], p2[1]));
  for (int y = miny; y < maxy; y++) {
    for (int x = minx; x < maxx; x++) {
      if ((x0 - x1) * (y - y0) - (y0 - y1) * (x - x0) > 0 &&
          (x1 - x2) * (y - y1) - (y1 - y2) * (x - x1) > 0 &&
          (x2 - x0) * (y - y2) - (y2 - y0) * (x - x2) > 0)
        fn(Vec2i(x, y));
    }
  }
}
