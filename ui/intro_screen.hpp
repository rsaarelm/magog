// Copyright (C) 2012 Risto Saarelma

#ifndef INTRO_SCREEN_HPP
#define INTRO_SCREEN_HPP

#include <util.hpp>

class Intro_Screen : public Game_State {
 public:
  Intro_Screen() {}
  virtual ~Intro_Screen() {}

  virtual void enter() {}
  virtual void exit() {}
  virtual void key_event(int keysym, int printable);
  virtual void update(float interval_seconds) {}
  virtual void draw();

 private:
};

#endif
