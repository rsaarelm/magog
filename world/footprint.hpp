/* footprint.hpp

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
#ifndef WORLD_FOOTPRINT_HPP
#define WORLD_FOOTPRINT_HPP

#include <world/location.hpp>

/// A single-cell entity's footprint.
Footprint small_footprint(Location center);

/// A 7-cell large entity's footprint.
Footprint large_footprint(Location center);

#endif
