/* config.cpp

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

#include "config.hpp"
#include <string>
#include <boost/program_options.hpp>
#include <util.hpp>

using namespace std;
using namespace boost::program_options;

Key_Bindings g_keybindings;

#define KEYBIND(name, default_val, desc) (#name, value<string>(&g_keybindings.##name)->default_value(default_val), desc)

void parse_command_line(int argc, char* argv[]) {
  try {
    options_description desc("Options");
    desc.add_options()
        ("move_n", value<string>()->default_value("e"), "Key to move north")
        ("move_ne", value<string>()->default_value("r"), "Key to move northeast")
        ("move_se", value<string>()->default_value("f"), "Key to move southeast")
        ("move_s", value<string>()->default_value("d"), "Key to move south")
        ("move_sw", value<string>()->default_value("s"), "Key to move southwest")
        ("move_nw", value<string>()->default_value("w"), "Key to move northwest")
        ("shoot_n", value<string>()->default_value("i"), "Key to shoot north")
        ("shoot_ne", value<string>()->default_value("o"), "Key to shoot northeast")
        ("shoot_se", value<string>()->default_value("l"), "Key to shoot southeast")
        ("shoot_s", value<string>()->default_value("k"), "Key to shoot south")
        ("shoot_sw", value<string>()->default_value("j"), "Key to shoot southwest")
        ("shoot_nw", value<string>()->default_value("u"), "Key to shoot northwest")
        ;
    variables_map vm;
    store(parse_command_line(argc, argv, desc), vm);
    notify(vm);
  } catch (exception& e) {
    die(e.what());
  }

}
