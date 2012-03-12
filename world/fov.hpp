/* fov.hpp

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

#ifndef WORLD_FOV_HPP
#define WORLD_FOV_HPP

#include <world/world.hpp>
#include <map>

/// Compute a shadowcasting field of view of radius hex circles around the
/// origin location on a hexagon tile map that may contain portals.
Relative_Fov hex_field_of_view(
    int radius,
    Location origin);

#endif
