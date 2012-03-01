#include "cavegen.hpp"
#include <xev.hpp>
#include <set>
#include <array>

using namespace std;
using namespace xev;

void dig(const Location& loc) {
  // XXX: Move the hardcoded terrain data somewhere else.
  // TODO: Figure out formatting the walls around the floor.
  set_terrain(loc, Terrain{5, "floor", Color(96, 96, 48)});
}

void generate_cave(const Location& origin, const ARecti& area) {
  set<Vec2i> dug;
  set<Vec2i> edge;

  Vec2i pos = area.min() + area.dim() / 2;
  const float floor_fraction = 0.5;
  size_t n = area.volume() * floor_fraction;

  dig(origin + pos);
  dug.insert(pos);
  for (auto a : hex_dirs) {
    if (dug.find(pos + a) == dug.end()) {
      edge.insert(pos + a);
    }
  }

  while (dug.size() < n && edge.size() > 0) {
    auto pick = rand_choice(edge);
    pos = *pick;

    int n_floor = 0;
    for (auto a : hex_dirs)
      if (dug.find(pos + a) != dug.end())
        n_floor++;

    // Decide whether to dig here. Prefer to dig in closed quarters and
    // destroy singleton pillars.
    if (n_floor < 6 && rand_int(n_floor * n_floor) > 1)
      continue;

    edge.erase(pick);

    dig(origin + pos);
    dug.insert(pos);
    for (auto a : hex_dirs) {
      if (dug.find(pos + a) == dug.end() && area.contains(pos + a)) {
        edge.insert(pos + a);
      }
    }
  }
}
