// Copyright (C) 2012 Risto Saarelma

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
