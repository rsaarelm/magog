/* game_loop.cpp

   Copyright (C) 2012 Risto Saarelma

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "game_loop.hpp"
#include <util/game_state.hpp>
#include <util/core.hpp>
#include <cstdlib>
#include <algorithm>
#include <iostream>
#include <GL/glew.h>
#include <SDL/SDL.h>

std::unique_ptr<Game_Loop> Game_Loop::s_instance;

Game_Loop::Game_Loop()
    : target_fps(60)
    , running(false)
{}

Game_Loop::~Game_Loop() {
}

void Game_Loop::push_state(Game_State* state) {
  stack_ops.push_back([&, state]() {
      states.push_back(std::unique_ptr<Game_State>(state));
      top_state()->enter();
    });
}

void Game_Loop::pop_state() {
  stack_ops.push_back([&]() {
      states.back()->exit();
      states.pop_back();
    });
}

void Game_Loop::update_state_stack() {
  for (auto op : stack_ops)
    op();
  stack_ops.clear();
}

void init_gl() {
  GLenum err = glewInit();
  if (GLEW_OK != err) {
    die("GLEW init failed: %d", err);
  }
  if (!GLEW_VERSION_2_0) {
    die("OpenGL 2.0 not available.\nMake sure your video drivers are up to date.");
  }

  glClearColor(.05, .1, .1, 1);
  glEnable(GL_TEXTURE_2D);
  glEnable(GL_BLEND);
  glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
}

Game_Loop& Game_Loop::init(int w, int h, const char* title) {
  ASSERT(s_instance == nullptr);
  s_instance = std::unique_ptr<Game_Loop>(new Game_Loop);

  if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO))
    die("Unable to init SDL: %s", SDL_GetError());

  if (SDL_SetVideoMode(w, h, 0, SDL_OPENGL) == nullptr)
    die("Unable to open SDL window: %s", SDL_GetError());

  SDL_WM_SetCaption(title, title);

  SDL_EnableUNICODE(1);
  SDL_EnableKeyRepeat(SDL_DEFAULT_REPEAT_DELAY, SDL_DEFAULT_REPEAT_INTERVAL);

  s_instance->mixer.init();

#if !defined(NDEBUG) && defined(__WIN32__)
  // Windows builds aren't console apps, so make them output info to a log
  // file.
  freopen("stdout.txt", "w", stdout);
  freopen("stderr.txt", "w", stderr);
#endif

  init_gl();
  return get();
}

Vec2i Game_Loop::get_dim() const {
  Vec2i result;
  auto surface = SDL_GetVideoSurface();
  result[0] = surface->w;
  result[1] = surface->h;
  return result;
}

double Game_Loop::get_seconds() const {
  return SDL_GetTicks() / 1000.0;
}

bool Game_Loop::update_states(float interval) {
  update_state_stack();

  if (states.empty()) {
    return false;
  } else {
    for (auto& state : states)
      state->update(interval);
    return true;
  }
}

static int mouse_button_mask() {
  int x, y;
  return SDL_GetMouseState(&x, &y);
}

void Game_Loop::run() {
  const float interval = 1.0 / target_fps;
  double time = SDL_GetTicks() / 1000.0;
  running = true;
  update_state_stack();
  while (running) {
    double current_time = SDL_GetTicks() / 1000.0;

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
      for (auto& state : states)
        state->draw();
      SDL_GL_SwapBuffers();
    } else {
      // Don't busy wait.
      SDL_Delay(10);
    }
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
      Game_State *top = top_state();
      switch (event.type) {
      case SDL_KEYDOWN:
        if (top)
          top->key_event(event.key.keysym.sym, event.key.keysym.unicode);
        break;
      case SDL_KEYUP:
        if (top)
          top->key_event(-event.key.keysym.sym, -1);
        break;
      case SDL_MOUSEBUTTONDOWN:
      case SDL_MOUSEBUTTONUP:
        if (top)
          top->mouse_event(event.button.x, event.button.y, mouse_button_mask());
        break;
      case SDL_MOUSEMOTION:
        if (top)
          top->mouse_event(event.motion.x, event.motion.y, mouse_button_mask());
        break;
      case SDL_QUIT:
        quit();
        break;
      }
    }
  }
  SDL_Quit();
}

void Game_Loop::quit() {
  if (!running) return;
  running = false;
  for (size_t i = 0; i < states.size(); i++)
    pop_state();
}

Game_State* Game_Loop::top_state() {
  if (states.size() > 0)
    return states.back().get();
  else
    return nullptr;
}
