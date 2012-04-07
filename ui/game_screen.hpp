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

#include <GL/gl.h>
#include <ui/font_data.hpp>
#include <ui/hud_system.hpp>
#include <ui/ui_fx_system.hpp>
#include <ui/sprite_system.hpp>
#include <ui/display_system.hpp>
#include <world/entities_system.hpp>
#include <world/terrain_system.hpp>
#include <world/spatial_system.hpp>
#include <world/factory_system.hpp>
#include <world/mapgen_system.hpp>
#include <world/fov_system.hpp>
#include <world/action_system.hpp>
#include <world/cycler_system.hpp>
#include <util/game_state.hpp>
#include <util/fonter_system.hpp>

class Game_Screen : public Game_State {
 public:
  Game_Screen()
    : fonter(font_sheet, font_data, font_height)
    , entities()
    , terrain()
    , spatial(entities, terrain)
    , factory(entities, terrain, spatial)
    , mapgen(terrain, factory)
    , fov(entities, terrain, spatial)
    , sprite(fov)
    , hud(fonter, entities, spatial)
    , fx(sprite, hud)
    , display(entities, terrain, spatial, fov, sprite)
    , action(entities, terrain, spatial, fov, fx)
    , cycler(entities, spatial, action)
    , state(state_playing) {}
  virtual ~Game_Screen() {}

  virtual void enter();
  virtual void exit();
  virtual void key_event(int keycode, int printable, int scancode);
  virtual void update(float interval_seconds);
  virtual void draw();

  void end_game();

  void draw_tile(int idx, const Vec2f& pos);
  void draw_tile(int idx, const Vec2f& pos, const Color& color);

  Fonter_System fonter;

  Entities_System entities;
  Terrain_System terrain;
  Spatial_System spatial;
  Factory_System factory;
  Mapgen_System mapgen;
  Fov_System fov;
  Sprite_System sprite;
  Hud_System hud;
  Ui_Fx_System fx;
  Display_System display;
  Action_System action;
  Cycler_System cycler;

private:
  enum State {
    state_playing,
    state_won,
    state_lost
  };

  void end_turn();

  State state;
};

#endif
