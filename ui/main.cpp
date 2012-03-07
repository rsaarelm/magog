// Copyright (C) 2012 Risto Saarelma

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
