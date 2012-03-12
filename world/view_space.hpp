/* view_space.hpp

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
  void do_fov(int radius, Location loc, const Vec2i& offset=Vec2i(0, 0));

  boost::optional<Location> at(const Vec2i& pos) const;
  bool is_seen(Location loc) const;

  void clear_seen();

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
