// Copyright (C) Risto Saarelma 2012

#include "view_space.hpp"
#include <world/fov.hpp>
#include <util/alg.hpp>
#include <util/hex.hpp>

using namespace boost;
using namespace std;

void View_Space::do_fov(int radius, const Location& origin) {
  prune();
  visible.clear();
  auto fov = hex_field_of_view(radius, origin);
  for (auto& pair : fov) {
    auto pos = pair.first + subjective_pos;
    view[pos] = pair.second;
    visible.insert(pair.second);
  }
}

boost::optional<Location> View_Space::at(const Vec2i& pos) const {
  return assoc_find(view, pos);
}

bool View_Space::is_seen(const Location& loc) const {
  return assoc_contains(visible, loc);
}

void View_Space::prune() {
  // Cut down far-away parts if the storage threatens to become too large.
  const int capacity = 65536;
  const int keep_radius = 48;

  // XXX: This could be a lot more efficient if the underlying structure was a quadtree.
  if (view.size() > capacity) {
    for (auto i = view.begin(); i != view.end(); i++) {
      auto dist = hex_dist(subjective_pos - i->first);
      if (dist > keep_radius) {
        view.erase(i);
      }
    }
  }
}
