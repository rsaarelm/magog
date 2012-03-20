/* font_data.hpp

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
#ifndef UI_FONT_DATA_HPP
#define UI_FONT_DATA_HPP

#include <util/fonter_system.hpp>
#include <util/surface.hpp>
#include <vector>

const Surface font_sheet {
#include <font_image.hpp>
};

const std::vector<Fonter_System::Font_Data> font_data {
#include <font_data.hpp>
};

const int font_height = 13;

#endif
