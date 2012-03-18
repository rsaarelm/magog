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
#include <world/rules.hpp>
#include <world/cavegen.hpp>
#include <world/parts.hpp>
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

#if 0
// TODO: Move to Terrain_System0

bool is_wall(Location loc) {
  return terrain_data[terrain.get(loc)].kind == wall_terrain;
}

int wall_mask(Location loc) {
  int result = 0;
  for (size_t i = 0; i < hex_dirs.size(); i++)
    result += is_wall(loc + hex_dirs[i]) << i;
  return result;
}
#endif

static GLuint load_tile_tex() {
  // XXX: Expensive to call this more than once. Should have a media cache if I have more media.
  g_tile_surface.load_image(tiles_png, sizeof(tiles_png));
  return make_texture(g_tile_surface);
}

Entity Game_Screen::spawn_infantry(Location location) {
  auto entity = new_entity();
  entity.add_part(new Blob_Part(icon_infantry, 3, 6, 2));
  if (spatial.can_pop(entity, location))
    spatial.pop(entity, location);
  else
    delete_entity(entity);
  return entity;
}

Entity Game_Screen::spawn_armor(Location location) {
  auto entity = new_entity();
  entity.add_part(new Blob_Part(icon_tank, 5, 8, 4));
  if (spatial.can_pop(entity, location))
    spatial.pop(entity, location);
  else
    delete_entity(entity);
  return entity;
}

static shared_ptr<Drawable> tile_drawable(GLuint texture, int index, Color color,
                                          Vec2f offset = Vec2f(0, 0)) {
  return shared_ptr<Drawable>(
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
  entity_drawables.clear();
  entity_drawables.push_back(tile_drawable(tiletex, 8, "#f0f"));
  entity_drawables.push_back(tile_drawable(tiletex, 22, "#0f7"));
  entity_drawables.push_back(tile_drawable(tiletex, 24, "#fd0"));
  entity_drawables.push_back(tile_drawable(tiletex, 27, "#88f", -tile_size));

  // XXX: Ensure player entity exists. Hacky magic number id.
  new_entity(1);

  // Generate portals for a looping hex area.
  const int r = 16;

  for (auto pos : hex_area_points(r)) {
    int n = rand_int(100);
    if (n < 3)
      terrain.set(Location(1, pos), terrain_wall_center);
    else if (n < 6)
      terrain.set(Location(1, pos), terrain_water);
    else if (n < 12)
      terrain.set(Location(1, pos), terrain_forest);
    else if (n < 20)
      terrain.set(Location(1, pos), terrain_sand);
    else
      terrain.set(Location(1, pos), terrain_grass);
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
        Location(1, start[sector] + hex_dirs[(sector + 1) % 6] * i), Portal(0, offset[sector]));

  for (int i = 0; i < 16; i++) {
    // TODO: random location function
    auto loc = Location(1, Vec2i(rand_int(10), rand_int(10)));
    // TODO: check if loc is occupied
    if (one_chance_in(3))
      spawn_armor(loc);
    else
      spawn_infantry(loc);
  }

  for (auto pos : hex_circle_points(r)) {
    terrain.set(Location(1, pos), terrain_floor);
  }
  for (auto pos : hex_circle_points(r+1)) {
    terrain.set(Location(1, pos), terrain_void);
  }

  auto player = get_player();
  player.add_part(new Blob_Part(icon_telos, 7, 40, 10, true));

  auto locations = terrain.area_locations(1);
  int n_tries = 1024;
  for (; n_tries; n_tries--) {
    auto loc = rand_choice(locations.first, locations.second);
    if (spatial.can_pop(player, loc->first)) {
      spatial.pop(player, loc->first);
      break;
    }
  }
  if (n_tries == 0) {
    die("Couldn't find spawn point");
  }
  fov.do_fov();

  msg_buffer.add_caption("Telos Unit online");
}

void Game_Screen::exit() {
  glDeleteTextures(1, &tiletex);
  clear_world();
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
      sprite.add(std::shared_ptr<Drawable>(new DemoThingie()), get_player().location());
      break;
    case 'u':
      action.shoot(get_player(), Vec2i(-1, 0));
      next_entity();
      break;
    case 'i':
      action.shoot(get_player(), Vec2i(-1, -1));
      next_entity();
      break;
    case 'o':
      action.shoot(get_player(), Vec2i(0, -1));
      next_entity();
      break;
    case 'l':
      action.shoot(get_player(), Vec2i(1, 0));
      next_entity();
      break;
    case 'k':
      action.shoot(get_player(), Vec2i(1, 1));
      next_entity();
      break;
    case 'j':
      action.shoot(get_player(), Vec2i(0, 1));
      next_entity();
      break;
    case 'b':
      {
        printf("Benchmarking lots of FOV\n");
        double t = Game_Loop::get().get_seconds();
        int n = 1000;
        for (int i = 0; i < n; i++)
          fov.do_fov();
        t = Game_Loop::get().get_seconds() - t;
        printf("Did %d fovs in %f seconds, one took %f seconds.\n", n, t, t/n);
      }
      break;
    default:
      break;
  }
  if (active_entity() == get_player() && action.is_ready(get_player())) {
    if (delta != Vec2i(0, 0)) {
      if (action.walk(get_player(), delta)) {
        fov.do_fov();
        next_entity();
      } else {
        msg_buffer.add_msg("Bump!");
      }
    }
  }
}

void Game_Screen::update(float interval_seconds) {
  msg_buffer.update(interval_seconds);
  sprite.update(interval_seconds);

  while (!(active_entity() == get_player() && action.is_ready(get_player()))) {
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
  auto mob = active_entity();
  if (action.is_ready(mob)) {
    auto& dir = *rand_choice(hex_dirs);
    // Stupid random fire
    if (one_chance_in(3))
      action.shoot(mob, dir);
    else
      action.walk(mob, dir);
  }
  next_entity();
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

  Color("beige").gl_color();
  draw_text({0, Registry::window_h - 20.0f}, "Armor level: %d", get_player().as<Blob_Part>().armor);
}

void Game_Screen::generate_sprites(std::set<Sprite>& output) {
  const int terrain_layer = 1;
  const int entity_layer = 2;

  try {
    auto loc = get_player().location();
    for (int y = -8; y <= 8; y++) {
      for (int x = -8; x <= 8; x++) {
        Vec2i offset(x, y);
        sprite.collect_sprites(offset, output);
        auto loc = fov.view_location(offset);
        if (loc.is_null())
          continue;

        bool in_fov = fov.is_seen(loc);

        // TODO: Darken terrain out of fov.
        auto ter = terrain_data[terrain.get(loc)];
        auto color = ter.color;
        if (!in_fov)
          color = lerp(0.5, Color("black"), color.monochrome());
        auto terrain_tile = tile_drawable(
          tiletex,
          ter.icon,
          color);
        output.insert(Sprite{terrain_layer, offset, std::move(terrain_tile)});

        if (in_fov) {
          for (auto& pair : spatial.entities_with_offsets_at(loc)) {
            Entity& entity = pair.second;
            auto& blob = entity.as<Blob_Part>();
            output.insert(Sprite{entity_layer, offset + pair.first, entity_drawables[blob.icon]});
          }
        }
      }
    }
  } catch (Entity_Exception& e) {
    // No player entity found or no valid Loction component in it.
  }
}
