/* mapgen_system.hpp

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
#ifndef WORLD_MAPGEN_SYSTEM_HPP
#define WORLD_MAPGEN_SYSTEM_HPP

#include <world/terrain_system.hpp>
#include <world/factory_system.hpp>
#include <util/box.hpp>

class Mapgen_System {
public:
  Mapgen_System(
    Terrain_System& terrain,
    Factory_System& factory)
    : terrain(terrain)
    , factory(factory) {}

  void cave(Plain_Location start, int start_dir6, const Recti& area);

  bool find_portal_enclosure(
    Plain_Location start,
    const Recti& area,
    Plain_Location& loc_out,
    int& dir6_out);
private:
  Mapgen_System(const Mapgen_System&);
  Mapgen_System& operator=(const Mapgen_System&);

  Terrain_System& terrain;
  Factory_System& factory;
};

#endif
