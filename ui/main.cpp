/* main.cpp

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

#include "telos.hpp"
#include <GL/glew.h>
#include <util.hpp>
#include <util/winmain.hpp>
#include "intro_screen.hpp"

int main(int argc, char* argv[])
{
  Game_Loop& game = Game_Loop::init(800, 600, "Telos");

  init_font();
  game.push_state(new Intro_Screen);
  game.run();
  return 0;
}
