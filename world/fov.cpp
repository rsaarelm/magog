#include "fov.hpp"
#include <xev/util.hpp>
#include <array>
#include <cmath>

using namespace xev;
using namespace std;
using namespace boost;

struct Fov_Group {
  bool opaque;
  optional<Portal> portal;

  Fov_Group(const Location& location)
      : opaque(blocks_sight(location + get_portal(location)))
      , portal(get_portal(location)) {}

  bool operator!=(const Fov_Group& rhs) {
    return rhs.opaque != opaque || rhs.portal != portal;
  }
};

struct Angle {
  float pos;
  int radius;

  int winding_index() const {
    return floor(pos + 0.5);
  }

  int end_index() const {
    return ceil(pos + 0.5);
  }

  bool is_below(const Angle& end_angle) const {
    return winding_index() < end_angle.end_index();
  }

  Vec2i operator*() const {
    // XXX: Could cache this.
    return hex_circle_vec(radius, winding_index());
  }

  Angle& operator++() {
    pos += 0.5;
    pos = floor(pos);
    pos += 0.5;
    return *this;
  }

  Angle extended() const {
    return Angle{pos * (radius + 1) / radius, radius + 1};
  }
};

void mark(Relative_Fov& rfov, const Vec2i& relative_pos, const Location& loc) {
  mark_seen(loc);
  rfov[relative_pos] = loc;
}

void process(
    Relative_Fov& rfov,
    int range,
    const Location& local_origin,
    Angle begin = Angle{0, 1},
    Angle end = Angle{6, 1}) {
  if (begin.radius > range)
    return;
  Fov_Group group(local_origin + *begin);
  for (auto a = begin; a.is_below(end); ++a) {
    if (Fov_Group(local_origin + *a) != group) {
      if (!group.opaque)
        process(rfov, range, local_origin + group.portal, begin.extended(), a.extended());
      process(rfov, range, local_origin, a, end);
      return;
    }
    mark(rfov, *a, local_origin + *a + group.portal);
  }
  if (!group.opaque)
    process(rfov, range, local_origin + group.portal, begin.extended(), end.extended());
}

Relative_Fov hex_field_of_view(
    int range,
    const Location& origin) {
  Relative_Fov result;
  mark(result, Vec2i(0, 0), origin);
  process(result, range, origin);
  return result;
}
