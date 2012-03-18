/* fov_system.hpp

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
#ifndef WORLD_FOV_SYSTEM_HPP
#define WORLD_FOV_SYSTEM_HPP

#include <world/entities_system.hpp>
#include <world/view_space.hpp>
#include <world/location.hpp>

class Fov_System {
public:
  Fov_System(Entities_System& entities)
  : entities(entities) {}

  bool is_seen(Location loc);
  Location view_location(const Vec2i& relative_pos);
  void do_fov();
  void move_view_pos(const Vec2i& offset);
private:
  Entities_System& entities;

  View_Space view_space;
};

#endif
