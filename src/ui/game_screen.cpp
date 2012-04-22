/* game_screen.cpp

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

#include "game_screen.hpp"
#include <ui/intro_screen.hpp>
#include <ui/registry.hpp>
#include <world/parts.hpp>
#include <util/hex.hpp>
#include <util/num.hpp>
#include <util/sdl_util.hpp>
#include <util/game_loop.hpp>

using namespace std;

void Game_Screen::enter() {
  const int chunk_w = 32;
  const int chunk_h = 32;
  static const char chunk[chunk_h][chunk_w + 1] = {
    "###........#,,,~~,,,,#..........",
    "#.#........#,,,,~~,,,#..........",
    "#A#........#,,,,~~,,,#..........",
    "...........#,,,,~~,,,#..........",
    "...........#,,,~~,,,,#..........",
    "...........#,,,~~,,,,#..........",
    "...........#,,,,~~,,,#..........",
    "...........#,,,~~,,,,#..........",
    "...........#,,~~~,,,,#..........",
    "#..#########,,,~~,,,,###########",
    ",,,,,,,,T,,,,,,,~~~~,,,,,,,,,...",
    ",,,,,T,,,,,,,,,,,~~~~~~~~~......",
    ",,,,,,,,,T,,,,,,,,,~~~~~~~......",
    ",.,,,,,,,,,,,,,,,,,,,,,,,~~~....",
    ",,,,,,,,,TT,,,,I,,,,,,,,,~,~~~..",
    "~~~~,,,,TTT,,,I..I,,,,,,,,,~~~~~",
    "~~~~~,,,,T,,,,,,,..,,,,,,,,,,,~~",
    "____~~,,,,,T,,,I,.I.,,,,,,,,,,,,",
    ",,__~~,,T,,,,,,,,I,,.,,,,,,,,,,,",
    ",_,__~~,,,,,,,,,,,,,,,,,,,,,,,,,",
    ",,,,,_~~~~~,,,,,,,,,,,,,,,,,,,,,",
    ",,,,,,,~~,~~~,,,,,,,,,,,,,,,,,,,",
    ",,,,,,,,,,~~~~,,,,,,,,,,,,,,,,,,",
    ",,,,,,,,,,,,,~~~,,,,,,,,,,,,,,,,",
    "############,,,~,,,,,###########",
    "...........#,,,~~,,,,#..........",
    "...........#,,,~~,,,,#..........",
    "...........#,,,~~,,,,#..........",
    "...........#,,,,~~,,,#..........",
    "...........#,,,,~~,,,#..........",
    "...........#,,,~~~,,,#..........",
    "...........#,,,~~,,,,#..........",
  };

  for (int y = 0; y < chunk_h; y++) {
    for (int x = 0; x < chunk_w; x++) {
      auto loc = Plain_Location(1, x, y);
      switch (chunk[y][x]) {
      case ',':
        terrain.set(loc, terrain_grass);
        break;
      case '.':
        terrain.set(loc, terrain_floor);
        break;
      case '_':
        terrain.set(loc, terrain_sand);
        break;
      case '~':
        terrain.set(loc, terrain_water);
        break;
      case 'I':
        terrain.set(loc, terrain_menhir);
        break;
      case 'T':
        terrain.set(loc, terrain_tree);
        break;
      case '#':
        terrain.set(loc, terrain_wall);
        break;
      case '<':
        terrain.set(loc, terrain_slope_nw);
        break;
      case 'P':
        terrain.set(loc, terrain_slope_n);
        break;
      case 'A':
        terrain.set(loc, terrain_slope_ne);
        break;
      case '>':
        terrain.set(loc, terrain_slope_se);
        break;
      case 'J':
        terrain.set(loc, terrain_slope_s);
        break;
      case 'V':
        terrain.set(loc, terrain_slope_sw);
        break;
      default:
        break;
      }
    }
  }

  // Loop portals
  for (int y = 0; y < chunk_h; y++) {
    terrain.set_portal({1, {-1, y}}, Portal(0, {chunk_w, 0}));
    terrain.set_portal({1, {chunk_w, y}}, Portal(0, {-chunk_w, 0}));
  }

  for (int x = 0; x < chunk_w; x++) {
    terrain.set_portal({1, {x, -1}}, Portal(0, {0, chunk_h}));
    terrain.set_portal({1, {x, chunk_h}}, Portal(0, {0, -chunk_h}));
  }

  terrain.set_portal({1, {-1, -1}}, Portal(0, {chunk_h, chunk_w}));
  terrain.set_portal({1, {chunk_w, chunk_h}}, Portal(0, {-chunk_h, -chunk_w}));

  // Downstairs
  terrain.set_portal({1, {1, 1}}, Portal(2, {-1, -1}));
  mapgen.cave(terrain.location(2, {0, 0}), 1, Recti({-16, -16}, {32, 32}));
  terrain.set_portal({2, {0, 1}}, Portal(1, {1, 1}));

  // Entity spawns
  Entity player =
    factory.spawn(spec_player, terrain.location(1, Vec2i(16, 16)));
  entities.as<Blob_Part>(player).faction = player_faction;

  for (int i = 0; i < 16; i++) {
    auto spec = one_chance_in(3) ? spec_thrall : spec_dreg;
    factory.spawn(spec, factory.random_spawn_point(spec, 2));
  }

  fov.do_fov(player);

  // Prime the cycler.
  cycler.run();
}

void Game_Screen::exit() {
}

const char* colemak_map = " !\"#$%&'()*+,-./0123456789Pp<=>?@ABCGKETHLYNUMJ:RQSDFIVWXOZ[\\]^_`abcgkethlynumj;rqsdfivwxoz{|}~";
const char* dvorak_map = " !Q#$%&q()*}w'e[0123456789ZzW]E{@ANIHDYUJGCVPMLSRXO:KF><BT?/\\=^\"`anihdyujgcvpmlsrxo;kf.,bt/_|+~";

int remap_key(int keysym, const char* keymap) {
  if (keysym >= 32 && keysym < 127)
    return keymap[keysym - 32];
  else
    return keysym;
}

void Game_Screen::key_event(int keysym, int printable, int scancode) {
  Vec2i delta(0, 0);

  switch (Registry::keyboard_layout) {
  case colemak:
    keysym = remap_key(keysym, colemak_map);
    break;
  case dvorak:
    keysym = remap_key(keysym, dvorak_map);
    break;
  default:
    break;
  }

  switch (keysym) {
  case 27: // Escape
    end_game();
    break;
  break;
  default:
    break;
  }

  Entity player = cycler.current_player();
  if (!player)
    return;

  ASSERT(action.is_ready(player));

  switch (Registry::use_scancodes ? keysym_for_scancode(scancode) : keysym) {
  case 'q': delta = Vec2i(-1, 0); break;
  case 'w': delta = Vec2i(-1, -1); break;
  case 'e': delta = Vec2i(0, -1); break;
  case 'a': delta = Vec2i(0, 1); break;
  case 's': delta = Vec2i(1, 1); break;
  case 'd': delta = Vec2i(1, 0); break;
  case 'u':
    action.shoot(player, Vec2i(-1, 0));
    end_turn();
    break;
  case 'i':
    action.shoot(player, Vec2i(-1, -1));
    end_turn();
    break;
  case 'o':
    action.shoot(player, Vec2i(0, -1));
    end_turn();
    break;
  case 'l':
    action.shoot(player, Vec2i(1, 0));
    end_turn();
    break;
  case 'k':
    action.shoot(player, Vec2i(1, 1));
    end_turn();
    break;
  case 'j':
    action.shoot(player, Vec2i(0, 1));
    end_turn();
    break;
  }

  switch (keysym) {
  case ' ':
    action.wait(player);
    end_turn();
    break;
  case SDLK_F12:
    screenshot(format("/tmp/%s-", Registry::app_name).c_str());
    break;
  default:
    break;
  }
  if (delta != Vec2i(0, 0)) {
    if (action.bump(player, delta)) {
      fov.do_fov(player);
      end_turn();
    } else {
      hud.add_msg("Bump!");
    }
  }
}

void Game_Screen::update(float interval_seconds) {
  hud.update(interval_seconds);
  sprite.update(interval_seconds);

  if (!cycler.current_player())
    end_turn();
}

void Game_Screen::end_game() {
  Game_Loop::get().pop_state();
  Game_Loop::get().push_state(new Intro_Screen(file));
}

void Game_Screen::draw() {
  Entity player = cycler.current_player();
  display.draw(player, Rectf(Vec2f(0, 0), Vec2f(Registry::window_w, Registry::window_h)));
  hud.draw(player);
}

void Game_Screen::end_turn() {
  cycler.run();
  if (state == state_playing) {
    const float time_util_return_to_intro = 7;
    int n_player = action.count_aligned(player_faction);
    int n_enemy = action.count_aligned(npc_faction);
    if (n_player == 0) {
      state = state_lost;
      hud.add_caption("You have died");
      hud.add_caption("Press ESC to exit");
      hud.add_event(time_util_return_to_intro, [&] { end_game(); });
    } else if (n_enemy == 0) {
      state = state_won;
      hud.add_caption("They are dead");
      hud.add_caption("Press ESC to exit");
      hud.add_event(time_util_return_to_intro, [&] { end_game(); });
    }
  }
}
