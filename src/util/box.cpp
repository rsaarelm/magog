/* box.cpp

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

#include "box.hpp"

std::vector<Vec2i> points(const Recti& rect) {
  std::vector<Vec2i> result;
  for (int i = 0; i < rect.dim()[1]; i++)
    for (int j = 0; j < rect.dim()[0]; j++)
      result.push_back(Vec2i(j, i) + rect.min());
  return result;
}

