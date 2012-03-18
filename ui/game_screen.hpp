/* game_screen.hpp

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

#ifndef GAME_SCREEN_HPP
#define GAME_SCREEN_HPP

#include <GL/glew.h>
#include "message_buffer.hpp"
#include "drawable.hpp"
#include "sprite.hpp"
#include "world_space_anims.hpp"
#include <world/world.hpp>
#include <world/entities_system.hpp>
#include <world/fov_system.hpp>
#include <world/action_system.hpp>
#include <util.hpp>
#include <queue>
#include <vector>
#include <memory>
#include <set>
#include <functional>

const Vec2f tile_size(16, 16);

const Mtx<float, 2, 2> tile_projection{
  tile_size[0],    -tile_size[0],
  tile_size[1] / 2, tile_size[1] / 2};

class Game_Screen : public Game_State {
 public:
  Game_Screen()
      : tiletex(0)
      , entities()
      , fov(entities)
      , world_anims(fov)
      , action(entities, fov) {}
  virtual ~Game_Screen() {}

  virtual void enter();
  virtual void exit();
  virtual void key_event(int keycode, int printable);
  virtual void update(float interval_seconds);
  virtual void draw();

  void generate_sprites(std::set<Sprite>& output);

  void do_ai();
  void end_game();

  void draw_tile(int idx, const Vec2f& pos);
  void draw_tile(int idx, const Vec2f& pos, const Color& color);

  GLuint tiletex;

  Message_Buffer msg_buffer;

  std::vector<std::shared_ptr<Drawable>> entity_drawables;

  Entities_System entities;
  Fov_System fov;
  World_Space_Anims world_anims;
  Action_System action;
};

#endif
