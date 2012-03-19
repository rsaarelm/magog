/* location.cpp

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

#include "location.hpp"
#include <world/terrain_system.hpp>

#include <ui/game_screen.hpp>
#include <util/game_loop.hpp>

Location Location::portaled() const {
  Terrain_System* ter = terrain;
  // XXX HACKHACKHACK FIXME
  if (ter == nullptr) {
    ter = &(dynamic_cast<Game_Screen*>(Game_Loop::get().top_state())->terrain);
  }
  return *this + ter->get_portal(*this);
//  ASSERT(terrain != nullptr);
//  return *this + terrain->get_portal(*this);
}

bool Location::blocks_sight() const {
  ASSERT(terrain != nullptr);
  return terrain->blocks_sight(*this);
}

Portal Location::get_portal() const {
  ASSERT(terrain != nullptr);
  return terrain->get_portal(*this);
}
