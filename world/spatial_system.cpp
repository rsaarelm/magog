/* spatial_system.cpp

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

#include "spatial_system.hpp"

bool Spatial_System::can_pop(Entity entity, Location loc) const {
  // TODO: Use Spatial_System
  return entity.can_pop(loc);
}

void push(Entity entity) {
  // TODO: Use Spatial_System
  entity.push();
}

void pop(Entity entity, Location loc) {
  // TODO: Use Spatial_System
  entity.pop(loc);
}
