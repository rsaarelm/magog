/* config.hpp

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

#ifndef CONFIG_HPP
#define CONFIG_HPP

#include <string>

struct Key_Bindings {
  std::string move_n;
  std::string move_ne;
  std::string move_se;
  std::string move_s;
  std::string move_sw;
  std::string move_nw;
  std::string shoot_n;
  std::string shoot_ne;
  std::string shoot_se;
  std::string shoot_s;
  std::string shoot_sw;
  std::string shoot_nw;
};

extern Key_Bindings g_keybindings;

void parse_command_line(int argc, char* argv[]);

#endif
