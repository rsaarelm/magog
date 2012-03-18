/* fov_system.cpp

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

#include "fov_system.hpp"
#include <world/fov.hpp>
#include <world/parts.hpp>
#include <world/rules.hpp>

bool Fov_System::is_seen(Location loc) {
  return view_space.is_seen(loc);
}

Location Fov_System::view_location(const Vec2i& relative_pos) {
  return view_space.at(relative_pos + view_space.get_pos());
}

void Fov_System::do_fov() {
  const int radius = 8;

  view_space.clear_seen();
  if (get_player().as<Blob_Part>().big) {
    // Big entities see with their edge cells too so that they're not completely
    // blind in a forest style terrain.
    for (auto i : hex_dirs)
      view_space.do_fov(radius, get_player().location() + i, i);
  }
  view_space.do_fov(radius, get_player().location());
}

void Fov_System::move_view_pos(const Vec2i& offset) {
  view_space.move_pos(offset);
}
