// Copyright (C) 2012 Risto Saarelma

#ifndef UTIL_GAME_LOOP_HPP
#define UTIL_GAME_LOOP_HPP

#include <memory>
#include <vector>
#include <functional>
#include "vec.hpp"
#include "game_state.hpp"

/**
 * Class for the top-level game application loop.
 *
 * Use with custom Game_State objects.
 */
class Game_Loop {
 public:
  ~Game_Loop();

  // Game_Loop owns pushed states.
  void push_state(Game_State* state);
  void pop_state();

  void set_state(Game_State* state) { pop_state(); push_state(state); }

  void run();

  void quit();

  Game_Loop& set_target_fps(float target_fps) { target_fps = target_fps; return *this; }

  Game_State* top_state();

  static Game_Loop& get() { return *s_instance; }

  static Game_Loop& init(int w, int h, const char* title);

  Vec2i get_dim() const;

  double get_seconds() const;
 private:
  Game_Loop();

  void update_state_stack();
  bool update_states(float interval);

  std::vector<Game_State*> states;
  std::vector<std::function<void()>> stack_ops;

  float target_fps;
  bool running;

  static std::unique_ptr<Game_Loop> s_instance;
};

#endif
