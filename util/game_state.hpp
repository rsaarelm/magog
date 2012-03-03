#ifndef UTIL_GAME_STATE_HPP
#define UTIL_GAME_STATE_HPP

class Game_State {
 public:
  Game_State() {}

  virtual ~Game_State() {}

  virtual void enter() {}
  virtual void exit() {}

  // Key release events get negative keycodes with the absolute value of the
  // released key's keycode.
  virtual void key_event(int keycode, int printable) {}
  virtual void mouse_event(int x, int y, int buttons) {}
  virtual void resize_event(int width, int height) {}

  virtual void update(float interval_seconds) = 0;
  virtual void draw() = 0;
 private:
};

#endif
