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
#include "intro_screen.hpp"
#include "tile_drawable.hpp"
#include <ui/registry.hpp>
#include <world/world.hpp>
#include <world/cavegen.hpp>
#include <GL/gl.h>
#include <util.hpp>
#include <vector>
#include <sstream>
#include <string>

using namespace std;

class DemoThingie : public Drawable {
public:
  DemoThingie() : life(10) { msg("DemoThingie is born"); }
  virtual ~DemoThingie() { msg("DemoThingie perishes"); }

  virtual bool update(float interval_sec) {
    life -= interval_sec;
    return life > 0;
  }

  virtual void draw(const Vec2f& offset) {
    // TODO: Centered draw_text.
    static char buf[256];
    snprintf(buf, sizeof(buf), "DemoThingie represent: %d", static_cast<int>(life));
    Color("white").gl_color();
    draw_text(offset + Vec2f(-text_width(buf) / 2, -font_height()), buf);
  }

  virtual int get_z_layer() const { return 100; }
private:
  float life;
};


Tile_Rect tile_rects[] = {
#include <tile_rect.hpp>
};

uint8_t tiles_png[] = {
#include <tile_atlas.hpp>
};

Surface g_tile_surface;

bool is_wall(const Location& loc) {
  return terrain_data[get_terrain(loc)].kind == wall_terrain;
}

int wall_mask(const Location& loc) {
  int result = 0;
  for (size_t i = 0; i < hex_dirs.size(); i++)
    result += is_wall(loc + hex_dirs[i]) << i;
  return result;
}

static GLuint load_tile_tex() {
  // XXX: Expensive to call this more than once. Should have a media cache if I have more media.
  g_tile_surface.load_image(tiles_png, sizeof(tiles_png));
  return make_texture(g_tile_surface);
}

Actor spawn_infantry(const Location& location) {
  auto actor = new_actor();
  actor.add_part(new Blob_Part(icon_infantry, 3));
  actor.pop(location);
  return actor;
}

Actor spawn_armor(const Location& location) {
  auto actor = new_actor();
  actor.add_part(new Blob_Part(icon_tank, 5));
  actor.pop(location);
  return actor;
}

static unique_ptr<Drawable> tile_drawable(GLuint texture, int index, Color color,
                                          Vec2f offset = Vec2f(0, 0)) {
  return unique_ptr<Drawable>(
    new Tile_Drawable(
      texture,
      color,
      tile_rects[index],
      g_tile_surface.get_dim(),
      offset));
}

void Game_Screen::enter() {
  tiletex = load_tile_tex();

  // TODO: Less verbose data entry.
  actor_drawables.clear();
  actor_drawables.push_back(tile_drawable(tiletex, 8, "#f0f"));
  actor_drawables.push_back(tile_drawable(tiletex, 22, "#0f7"));
  actor_drawables.push_back(tile_drawable(tiletex, 24, "#fd0"));
  actor_drawables.push_back(tile_drawable(tiletex, 27, "#88f", -tile_size));

  terrain_drawables.clear();
  for (int i = 0; i < NUM_TERRAINS; i++)
    terrain_drawables.push_back(
      tile_drawable(
        tiletex,
        terrain_data[i].icon,
        terrain_data[i].color));

  // XXX: Ensure player actor exists. Hacky magic number id.
  new_actor(1);

  // Generate portals for a looping hex area.
  const int r = 16;

  for (auto pos : hex_area_points(r)) {
    if (one_chance_in(8))
      set_terrain(Location{pos, 0}, terrain_sand);
    else
      set_terrain(Location{pos, 0}, terrain_grass);
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
      set_portal(Location{start[sector] + hex_dirs[(sector + 1) % 6] * i, 0}, Portal{offset[sector], 0});

  for (int i = 0; i < 16; i++) {
    // TODO: random location function
    auto loc = Location{Vec2i(rand_int(10), rand_int(10)), 0};
    // TODO: check if loc is occupied
    if (one_chance_in(3))
      spawn_armor(loc);
    else
      spawn_infantry(loc);
  }

  for (auto pos : hex_circle_points(r)) {
    set_terrain(Location{pos, 0}, terrain_floor);
  }
  for (auto pos : hex_circle_points(r+1)) {
    set_terrain(Location{pos, 0}, terrain_void);
  }

  auto player = get_player();
  player.add_part(new Blob_Part(icon_telos, 7));
  player.pop(Location{Vec2i(0, 0), 0});
  do_fov();

  msg_buffer.add_caption("Telos Unit online");
}

void Game_Screen::exit() {
  glDeleteTextures(1, &tiletex);
  World::clear();
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
      world_anims.add(std::unique_ptr<Drawable>(new DemoThingie()), get_player().location());
      break;
    case 'u':
      action_shoot(get_player(), Vec2i(-1, 0));
      next_actor();
      break;
    case 'i':
      action_shoot(get_player(), Vec2i(-1, -1));
      next_actor();
      break;
    case 'o':
      action_shoot(get_player(), Vec2i(0, -1));
      next_actor();
      break;
    case 'l':
      action_shoot(get_player(), Vec2i(1, 0));
      next_actor();
      break;
    case 'k':
      action_shoot(get_player(), Vec2i(1, 1));
      next_actor();
      break;
    case 'j':
      action_shoot(get_player(), Vec2i(0, 1));
      next_actor();
      break;
    case 'b':
      {
        printf("Benchmarking lots of FOV\n");
        double t = Game_Loop::get().get_seconds();
        int n = 1000;
        for (int i = 0; i < n; i++)
          do_fov();
        t = Game_Loop::get().get_seconds() - t;
        printf("Did %d fovs in %f seconds, one took %f seconds.\n", n, t, t/n);
      }
      break;
    default:
      break;
  }
  if (active_actor() == get_player() && ready_to_act(get_player())) {
    if (delta != Vec2i(0, 0)) {
      if (action_walk(get_player(), delta)) {
        do_fov();
        next_actor();
      } else {
        msg_buffer.add_msg("Bump!");
      }
    }
  }
}

void Game_Screen::update(float interval_seconds) {
  msg_buffer.update(interval_seconds);
  world_anims.update(interval_seconds);

  while (!(active_actor() == get_player() && ready_to_act(get_player()))) {
    do_ai();
    if (!get_player().exists()) {
      // TODO: Some kind of message that the player acknowledges here instead of
      // just a crude drop to intro.
      end_game();
      break;
    }
  }
}

void Game_Screen::do_ai() {
  auto mob = active_actor();
  if (ready_to_act(mob))
    action_walk(mob, *rand_choice(hex_dirs));
  next_actor();
}

void Game_Screen::end_game() {
  Game_Loop::get().pop_state();
  Game_Loop::get().push_state(new Intro_Screen);
}

void Game_Screen::draw() {
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
  generate_sprites(sprites);
  for (auto sprite : sprites) {
    auto draw_pos = Vec2f(projection * Vec3f(sprite.pos[0], sprite.pos[1], 1));
    sprite.draw(draw_pos);
  }

  msg_buffer.draw();
}

void Game_Screen::generate_sprites(std::set<Sprite>& output) {
  const int terrain_layer = 1;
  const int actor_layer = 2;

  try {
    auto loc = get_player().location();
    for (int y = -8; y <= 8; y++) {
      for (int x = -8; x <= 8; x++) {
        Vec2i offset(x, y);
        world_anims.collect_sprites(offset, output);
        auto loc = view_space_location(offset);
        if (!loc)
          continue;

        bool in_fov = is_seen(*loc);

        // TODO: Darken terrain out of fov.
        output.insert(Sprite{terrain_layer, offset, *terrain_drawables[get_terrain(*loc)]});

        if (in_fov) {
          for (auto& actor : actors_at(*loc)) {
            auto& blob = actor.as<Blob_Part>();
            output.insert(Sprite{actor_layer, offset, *actor_drawables[blob.icon]});
          }
        }
      }
    }
  } catch (Actor_Exception& e) {
    // No player actor found or no valid Loction component in it.
  }
}
