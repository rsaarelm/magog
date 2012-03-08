// Copyright (C) 2012 Risto Saarelma

#ifndef GAME_SCREEN_HPP
#define GAME_SCREEN_HPP

#include <GL/glew.h>
#include "message_buffer.hpp"
#include "drawable.hpp"
#include "sprite.hpp"
#include "world_space_anims.hpp"
#include <world/world.hpp>
#include <util.hpp>
#include <queue>
#include <vector>
#include <memory>
#include <set>
#include <functional>

class Game_Screen : public Game_State {
 public:
  typedef std::function<bool(float)> Animation;

  Game_Screen()
      : tiletex(0)
      , anim_interval(0.0) {}
  virtual ~Game_Screen() {}

  virtual void enter();
  virtual void exit();
  virtual void key_event(int keycode, int printable);
  virtual void update(float interval_seconds);
  virtual void draw();

  void generate_sprites(std::set<Sprite>& output);

  void do_ai();
  void end_game();

  void add_animation(Animation anim);

  void draw_tile(int idx, const Vec2f& pos);
  void draw_tile(int idx, const Vec2f& pos, const Color& color);

  void draw_anims(float interval_seconds);

  GLuint tiletex;

  float anim_interval;
  std::queue<Animation> animations;

  Message_Buffer msg_buffer;

  World_Space_Anims world_anims;

  std::vector<std::unique_ptr<Drawable>> actor_drawables;
  std::vector<std::unique_ptr<Drawable>> terrain_drawables;
};

#endif
