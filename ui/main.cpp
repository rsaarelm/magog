#include "telos.hpp"
#include <GL/glew.h>
#include <util.hpp>
#include <util/winmain.hpp>
#include "intro_screen.hpp"
#include "config.hpp"

int main(int argc, char* argv[])
{
  parse_command_line(argc, argv);
  Game_Loop& game = Game_Loop::init(800, 600, "Telos");

  init_font(_fontdata_font);
  game.push_state(new Intro_Screen);
  game.run();
  return 0;
}
