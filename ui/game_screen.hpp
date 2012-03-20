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
#include <ui/message_buffer.hpp>
#include <ui/drawable.hpp>
#include <ui/ui_fx_system.hpp>
#include <ui/sprite.hpp>
#include <ui/sprite_system.hpp>
#include <ui/font_data.hpp>
#include <ui/display_system.hpp>
#include <world/entities_system.hpp>
#include <world/terrain_system.hpp>
#include <world/spatial_system.hpp>
#include <world/fov_system.hpp>
#include <world/action_system.hpp>
#include <util/game_state.hpp>
#include <util/fonter_system.hpp>
#include <vector>
#include <memory>
#include <set>
#include <functional>

class Game_Screen : public Game_State {
 public:
  Game_Screen()
    : fonter(font_sheet, font_data, font_height)
    , msg_buffer(fonter)
    , entities()
    , terrain()
    , spatial(entities, terrain)
    , fov(entities, terrain, spatial)
    , sprite(fov)
    , fx(sprite)
    , display(entities, terrain, spatial, fov, sprite)
    , action(entities, terrain, spatial, fov, fx) {}
  virtual ~Game_Screen() {}

  virtual void enter();
  virtual void exit();
  virtual void key_event(int keycode, int printable);
  virtual void update(float interval_seconds);
  virtual void draw();

  void do_ai();
  void end_game();

  void draw_tile(int idx, const Vec2f& pos);
  void draw_tile(int idx, const Vec2f& pos, const Color& color);

  Fonter_System fonter;
  Message_Buffer msg_buffer;

  Entities_System entities;
  Terrain_System terrain;
  Spatial_System spatial;
  Fov_System fov;
  Sprite_System sprite;
  Ui_Fx_System fx;
  Display_System display;
  Action_System action;

private:
  Entity spawn_infantry(Location location);
  Entity spawn_armor(Location location);
};

#endif
