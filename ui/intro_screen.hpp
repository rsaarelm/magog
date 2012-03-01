#ifndef INTRO_SCREEN_HPP
#define INTRO_SCREEN_HPP

#include <xev.hpp>

class Intro_Screen : public xev::Game_State {
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
