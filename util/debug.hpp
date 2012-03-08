// Copyright (C) Risto Saarelma 2012

#ifndef UTIL_DEBUG_HPP
#define UTIL_DEBUG_HPP

#include <stdio.h>
#include <SDL/SDL.h>

class Print_Time {
public:
  Print_Time(const char *msg = "") : msg(msg), ms_ticks(SDL_GetTicks()) {}

  ~Print_Time() {
    printf("%s: took %g seconds.\n", msg, (SDL_GetTicks() - ms_ticks) / 1000.0);
  }
private:
  const char* msg;
  long ms_ticks;
};

#endif
