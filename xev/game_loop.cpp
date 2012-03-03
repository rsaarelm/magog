#include "game_loop.hpp"
#include "game_state.hpp"
#include "imgui.hpp"
#include "util.hpp"
#include <cstdlib>
#include <algorithm>
#include <iostream>
#include <GL/glew.h>
#include <GL/glfw.h>

namespace xev {

std::unique_ptr<Game_Loop> Game_Loop::s_instance;

Game_Loop::Game_Loop()
    : target_fps(60)
    , running(false)
{}

Game_Loop::~Game_Loop() {
  for (auto state : states) {
    delete state;
  }
}

void Game_Loop::push_state(Game_State* state) {
  stack_ops.push_back([&, state]() {
      states.push_back(state);
      state->enter();
    });
}

void Game_Loop::pop_state() {
  stack_ops.push_back([&]() {
      states.back()->exit();
      delete states.back();
      states.pop_back();
    });
}

void Game_Loop::update_state_stack() {
  for (auto op : stack_ops)
    op();
  stack_ops.clear();
}

void init_gl() {
#if 0
  GLenum err = glewInit();
  if (GLEW_OK != err) {
    die("GLEW init failed: %d", err);
  }
  if (!GLEW_VERSION_2_0) {
    die("OpenGL 2.0 not available\n");
  }
#endif
  glClearColor(.05, .1, .1, 1);
  glEnable(GL_TEXTURE_2D);
  glEnable(GL_BLEND);
  glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
}

Game_Loop& Game_Loop::init(int w, int h, const char* title) {
  ASSERT(s_instance == nullptr);
  s_instance = std::unique_ptr<Game_Loop>(new Game_Loop);

  if (!glfwInit())
    die("Unable to init GLFW");
  if (!glfwOpenWindow(w, h, 0, 0, 0, 0, 0, 0, GLFW_WINDOW))
    die("Unable to open GLFW window");
  glfwSetWindowTitle(title);
  glfwSetKeyCallback(Game_Loop::key_callback);
  glfwSetMouseButtonCallback(Game_Loop::mouse_button_callback);
  glfwSetMousePosCallback(Game_Loop::mouse_pos_callback);
  glfwEnable(GLFW_KEY_REPEAT);
  init_gl();
  return get();
}

Vec2i Game_Loop::get_dim() const {
  Vec2i result;
  glfwGetWindowSize(&result[0], &result[1]);
  return result;
}

bool Game_Loop::update_states(float interval) {
  update_state_stack();

  if (states.empty()) {
    return false;
  } else {
    for (auto state : states)
      state->update(interval);
    return true;
  }
}

void Game_Loop::run() {
  const float interval = 1.0 / target_fps;
  double time = glfwGetTime();
  running = true;
  update_state_stack();
  while (running) {
    double current_time = glfwGetTime();

    // Failsafe in case updates keep taking more time than interval and the
    // loop keeps falling back.
    int max_updates = 16;
    if (current_time - time >= interval) {
      while (current_time - time >= interval) {
        running = update_states(interval);
        if (!running)
          break;
        time += interval;
        if (max_updates-- <= 0) {
          // Forget about catching up.
          time = current_time;
          break;
        }
      }

      glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
      auto dim = get_dim();
      glViewport(0, 0, dim[0], dim[1]);
      for (auto state : states)
        state->draw();
      glfwSwapBuffers();
    } else {
      // Don't busy wait.
      glfwSleep(0.01);
    }
    if (!glfwGetWindowParam(GLFW_OPENED))
      quit();
  }
  glfwTerminate();
}

void Game_Loop::quit() {
  if (!running) return;
  running = false;
  printf("Quitting..\n");
  for (size_t i = 0; i < states.size(); i++)
    pop_state();
}

Game_State* Game_Loop::top_state() {
  if (states.size() > 0)
    return states.back();
  else
    return nullptr;
}

void Game_Loop::key_callback(int key, int action) {
  Game_State *top;
  if ((top = get().top_state()))
    top->key_event(key * (action == GLFW_PRESS ? 1 : -1), -1);
}

static int glfw_mouse_button_state() {
  return (glfwGetMouseButton(GLFW_MOUSE_BUTTON_LEFT) == GLFW_PRESS) +
      ((glfwGetMouseButton(GLFW_MOUSE_BUTTON_RIGHT) == GLFW_PRESS) << 1) +
      ((glfwGetMouseButton(GLFW_MOUSE_BUTTON_MIDDLE) == GLFW_PRESS) << 2);
}

void Game_Loop::mouse_pos_callback(int x, int y) {
  int buttons = glfw_mouse_button_state();
  Game_State *top;
  if ((top = get().top_state()))
    top->mouse_event(x, y, buttons);
  imgui_state.pos = Vec2f(x, y);
  imgui_state.button = buttons;
}

void Game_Loop::mouse_button_callback(int button, int action) {
  int buttons = glfw_mouse_button_state();
  int x, y;
  glfwGetMousePos(&x, &y);
  Game_State *top;
  if ((top = get().top_state()))
    top->mouse_event(x, y, buttons);
  imgui_state.pos = Vec2f(x, y);
  imgui_state.button = buttons;
}

}
