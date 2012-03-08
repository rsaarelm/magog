// Copyright (C) Risto Saarelma 2012

#ifndef WORLD_VIEW_SPACE_HPP
#define WORLD_VIEW_SPACE_HPP

#include <world/location.hpp>
#include <util/vec.hpp>
#include <boost/optional.hpp>
#include <map>
#include <set>

class View_Space {
public:
  void move_pos(const Vec2i& delta) { subjective_pos += delta; }
  Vec2i get_pos() const { return subjective_pos; }
  void do_fov(int radius, const Location& loc);

  boost::optional<Location> at(const Vec2i& pos) const;
  bool is_seen(const Location& loc) const;

  View_Space() {}
private:
  View_Space(const View_Space&);
  View_Space& operator=(const View_Space&);

  void prune();

  Vec2i subjective_pos;
  std::map<Vec2i, Location> view;
  std::set<Location> visible;
};

#endif
