#ifndef XEV_GAME_LOOP_HPP
#define XEV_GAME_LOOP_HPP

#include <memory>
#include <vector>
#include <functional>
#include <xev/vec.hpp>
#include <xev/game_state.hpp>

namespace xev {

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
 private:
  Game_Loop();

  void update_state_stack();
  bool update_states(float interval);

  static void key_callback(int key, int action);
  static void mouse_pos_callback(int x, int y);
  static void mouse_button_callback(int button, int action);

  std::vector<Game_State*> states;
  std::vector<std::function<void()>> stack_ops;

  float target_fps;
  bool running;

  static std::unique_ptr<Game_Loop> s_instance;
};

}

#endif
