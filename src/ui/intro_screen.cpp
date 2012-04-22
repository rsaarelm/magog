/* intro_screen.cpp

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

#include "intro_screen.hpp"
#include <ui/game_screen.hpp>
#include <ui/registry.hpp>
#include <ui/font_data.hpp>
#include <util/game_loop.hpp>
#include <util/core.hpp>
#include <util/sdl_util.hpp>
#include <GL/gl.h>

const char* buildname =
#include <buildname.hpp>
  ;

Intro_Screen::Intro_Screen(File_System& file)
  : file(file)
  , fonter(file, "pf_tempesta_seven_extended_bold.ttf", font_height)
  , imgui(fonter) {}

void Intro_Screen::key_event(int keysym, int printable, int scancode) {
  switch (keysym) {
  case 27: // Escape
    Game_Loop::get().pop_state();
    break;
  case 'n':
    Game_Loop::get().pop_state();
    Game_Loop::get().push_state(new Game_Screen(file));
    break;
  case SDLK_F12:
    screenshot(format("/tmp/%s-", Registry::app_name).c_str());
    break;
  default:
    break;
  }
}

void Intro_Screen::mouse_event(int x, int y, int buttons) {
  imgui.update(x, y, buttons);
}

void Intro_Screen::draw() {
  glClear(GL_COLOR_BUFFER_BIT);

  glMatrixMode(GL_PROJECTION);
  glLoadIdentity();
  auto dim = Game_Loop::get().get_dim();
  glOrtho(0, dim[0], dim[1], 0, -1, 1);

  glMatrixMode(GL_MODELVIEW);
  glLoadIdentity();
  glScalef(4.0, 4.0, 1.0);
  Color(196, 255, 196).gl_color();
  fonter.draw(Vec2f(0, 0), "%s", Registry::app_name);
  glLoadIdentity();

  fonter.draw(
    Vec2f(2, Registry::window_h - fonter.height() - 2),
    "build-%s %s %sbit %s",
    buildname,
    os_name(),
    os_bits(),
    debug_build_name());

  if (imgui.button(GEN_ID, "New Game", Rectf(Vec2f(dim[0]/2, 240), Vec2f(96, 16)))) {
    Game_Loop::get().pop_state();
    Game_Loop::get().push_state(new Game_Screen(file));
  }

  if (imgui.button(GEN_ID, "Exit", Rectf(Vec2f(dim[0]/2, 280), Vec2f(96, 16))))
    Game_Loop::get().quit();

}
