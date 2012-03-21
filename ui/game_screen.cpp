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
#include <util/game_loop.hpp>

using namespace std;

class DemoThingie : public Drawable {
public:
  DemoThingie(Fonter_System& fonter) : fonter(fonter), life(10) { }
  virtual ~DemoThingie() { }

  virtual bool update(float interval_sec) {
    life -= interval_sec;
    return life > 0;
  }

  virtual void draw(const Vec2f& offset) {
    // TODO: Centered draw_text.
    static char buf[256];
    snprintf(buf, sizeof(buf), "DemoThingie represent: %d", static_cast<int>(life));
    Color("white").gl_color();
    fonter.draw(offset + Vec2f(-fonter.width(buf) / 2, -fonter.height()), buf);
  }

  virtual int get_z_layer() const { return 100; }
private:
  Fonter_System& fonter;
  float life;
};

Entity Game_Screen::spawn_infantry(Location location) {
  auto entity = entities.create();
  entities.add(entity, unique_ptr<Part>(new Blob_Part(icon_infantry, 3, 6, 2)));
  if (spatial.can_pop(entity, location))
    spatial.pop(entity, location);
  else
    entities.destroy(entity);
  return entity;
}

Entity Game_Screen::spawn_armor(Location location) {
  auto entity = entities.create();
  entities.add(entity, unique_ptr<Part>(new Blob_Part(icon_tank, 5, 8, 4)));
  if (spatial.can_pop(entity, location))
    spatial.pop(entity, location);
  else
    entities.destroy(entity);
  return entity;
}

void Game_Screen::enter() {
  // XXX: Ensure player entity exists. Hacky magic number id.
  entities.create(1);

  // Generate portals for a looping hex area.
  const int r = 16;

  for (auto pos : hex_area_points(r)) {
    int n = rand_int(100);
    if (n < 3)
      terrain.set(Plain_Location(1, pos), terrain_wall_center);
    else if (n < 6)
      terrain.set(Plain_Location(1, pos), terrain_water);
    else if (n < 12)
      terrain.set(Plain_Location(1, pos), terrain_forest);
    else if (n < 20)
      terrain.set(Plain_Location(1, pos), terrain_sand);
    else
      terrain.set(Plain_Location(1, pos), terrain_grass);
  }


  const Vec2i start[]{
    {-(r+1), -1},
    {-(r+1), -(r+1)},
    {0, -r},
    {r, 0},
    {r-1, r},
    {-1, r}
  };

  const Vec2i offset[]{
    {2*r, r},
    {r, 2*r},
    {-r, r},
    {-2*r, -r},
    {-r, -2*r},
    {r, -r}
  };

  for (int sector = 0; sector < 6; sector++)
    for (int i = 0; i < r + (sector % 2); i++)
      terrain.set_portal(
        {1, start[sector] + hex_dirs[(sector + 1) % 6] * i}, Portal(0, offset[sector]));

  for (int i = 0; i < 16; i++) {
    // TODO: random location function
    auto loc = terrain.location(1, Vec2i(rand_int(10), rand_int(10)));
    // TODO: check if loc is occupied
    if (one_chance_in(3))
      spawn_armor(loc);
    else
      spawn_infantry(loc);
  }

  for (auto pos : hex_circle_points(r)) {
    terrain.set({1, pos}, terrain_floor);
  }
  for (auto pos : hex_circle_points(r+1)) {
    terrain.set({1, pos}, terrain_void);
  }

  auto player = spatial.get_player();
  entities.add(player, unique_ptr<Part>(new Blob_Part(icon_telos, 7, 40, 10, true)));

  auto locations = terrain.area_locations(1);
  int n_tries = 1024;
  for (; n_tries; n_tries--) {
    auto loc = rand_choice(locations);
    if (spatial.can_pop(player, *loc)) {
      spatial.pop(player, *loc);
      break;
    }
  }
  if (n_tries == 0) {
    die("Couldn't find spawn point");
  }
  fov.do_fov();

  hud.add_caption("Telos Unit online");
}

void Game_Screen::exit() {
}

int from_colemak(int keysym) {
  static const char* keymap =
      " !\"#$%&'()*+,-./0123456789Pp<=>?@ABCGKETHLYNUMJ:RQSDFIVWXOZ[\\]^_`abcgkethlynumj;rqsdfivwxoz{|}~";
  if (keysym >= 32 && keysym < 127)
    return keymap[keysym - 32];
  else
    return keysym;
}

void Game_Screen::key_event(int keysym, int printable) {
  Vec2i delta(0, 0);
  if (Registry::using_colemak)
    keysym = from_colemak(keysym);
  switch (keysym) {
    case 27: // Escape
      end_game();
      break;
    case 'q': delta = Vec2i(-1, 0); break;
    case 'w': delta = Vec2i(-1, -1); break;
    case 'e': delta = Vec2i(0, -1); break;
    case 'a': delta = Vec2i(0, 1); break;
    case 's': delta = Vec2i(1, 1); break;
    case 'd': delta = Vec2i(1, 0); break;
    case '1':
      sprite.add(std::shared_ptr<Drawable>(new DemoThingie(fonter)), spatial.location(spatial.get_player()));
      break;
    case 'u':
      action.shoot(spatial.get_player(), Vec2i(-1, 0));
      action.next_entity();
      break;
    case 'i':
      action.shoot(spatial.get_player(), Vec2i(-1, -1));
      action.next_entity();
      break;
    case 'o':
      action.shoot(spatial.get_player(), Vec2i(0, -1));
      action.next_entity();
      break;
    case 'l':
      action.shoot(spatial.get_player(), Vec2i(1, 0));
      action.next_entity();
      break;
    case 'k':
      action.shoot(spatial.get_player(), Vec2i(1, 1));
      action.next_entity();
      break;
    case 'j':
      action.shoot(spatial.get_player(), Vec2i(0, 1));
      action.next_entity();
      break;
    case 'b':
      {
        printf("Benchmarking lots of FOV\n");
        double t = Game_Loop::get().get_seconds();
        int n = 100;
        for (int i = 0; i < n; i++)
          fov.do_fov();
        t = Game_Loop::get().get_seconds() - t;
        printf("Did %d fovs in %f seconds, one took %f seconds.\n", n, t, t/n);
      }
      break;
    default:
      break;
  }
  if (action.active_entity() == spatial.get_player() && action.is_ready(spatial.get_player())) {
    if (delta != Vec2i(0, 0)) {
      if (action.walk(spatial.get_player(), delta)) {
        fov.do_fov();
        action.next_entity();
      } else {
        hud.add_msg("Bump!");
      }
    }
  }
}

void Game_Screen::update(float interval_seconds) {
  hud.update(interval_seconds);
  sprite.update(interval_seconds);

  while (!(action.active_entity() == spatial.get_player() && action.is_ready(spatial.get_player()))) {
    do_ai();
    if (!entities.exists(spatial.get_player())) {
      // TODO: Some kind of message that the player acknowledges here instead of
      // just a crude drop to intro.
      end_game();
      break;
    }
  }
}

void Game_Screen::do_ai() {
  auto mob = action.active_entity();
  if (action.is_ready(mob)) {
    auto& dir = *rand_choice(hex_dirs);
    // Stupid random fire
    if (one_chance_in(3))
      action.shoot(mob, dir);
    else
      action.walk(mob, dir);
  }
  action.next_entity();
}

void Game_Screen::end_game() {
  Game_Loop::get().pop_state();
  Game_Loop::get().push_state(new Intro_Screen);
}

void Game_Screen::draw() {
  // No player, no show.
  // TODO Make this more decoupled from a locked player so we don't need hacks
  // like this.
  if (!entities.exists(spatial.get_player()))
    return;

  glMatrixMode(GL_PROJECTION);
  glLoadIdentity();
  auto dim = Game_Loop::get().get_dim();
  glOrtho(0, dim[0], dim[1], 0, -1, 1);

  glMatrixMode(GL_MODELVIEW);
  glLoadIdentity();

  Mtx<float, 3, 3> projection{
    16, -16, static_cast<float>(dim[0]/2),
    8,   8,  static_cast<float>(dim[1]/3),
    0,   0,  1};
  glClear(GL_COLOR_BUFFER_BIT);

  set<Sprite> sprites;
  display.world_sprites(sprites);
  for (auto sprite : sprites) {
    auto draw_pos = Vec2f(projection * Vec3f(sprite.pos[0], sprite.pos[1], 1));
    sprite.draw(draw_pos);
  }

  hud.draw();

}
