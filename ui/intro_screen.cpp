// Copyright (C) 2012 Risto Saarelma

#include "intro_screen.hpp"
#include "telos.hpp"
#include <GL/glew.h>
#include <GL/glfw.h>
#include <util.hpp>
#include "game_screen.hpp"

void Intro_Screen::key_event(int keysym, int printable) {
  switch (keysym) {
    case GLFW_KEY_ESC:
      Game_Loop::get().pop_state();
      break;
    case 'N':
      Game_Loop::get().pop_state();
      Game_Loop::get().push_state(new Game_Screen);
      break;
    default:
      break;
  }
}

void Intro_Screen::draw() {
  glClear(GL_COLOR_BUFFER_BIT);

  glMatrixMode(GL_PROJECTION);
  glLoadIdentity();
  auto dim = Game_Loop::get().get_dim();
  glOrtho(0, dim[0], dim[1], 0, -1, 1);

  glMatrixMode(GL_MODELVIEW);
  glScalef(4.0, 4.0, 1.0);
  Color(196, 255, 196).gl_color();
  draw_text(Vec2f(0, 0), "TELOS v%s", VERSION);
  glLoadIdentity();

  if (im_button(GEN_ID, "New Game", ARectf(Vec2f(dim[0]/2, 240), Vec2f(96, 16)))) {
    Game_Loop::get().pop_state();
    Game_Loop::get().push_state(new Game_Screen);
  }

  if (im_button(GEN_ID, "Exit", ARectf(Vec2f(dim[0]/2, 280), Vec2f(96, 16))))
    Game_Loop::get().quit();

}
