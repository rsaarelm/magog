/* display_system.cpp

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

#include "display_system.hpp"
#include <world/parts.hpp>
#include <ui/tile_drawable.hpp>
#include <util/surface.hpp>
#include <util/hex.hpp>
#include <set>
#include <algorithm>

Display_System::Display_System(
  File_System& file,
  Entities_System& entities,
  Terrain_System& terrain,
  Spatial_System& spatial,
  Fov_System& fov,
  Sprite_System& sprite)
  : entities(entities)
  , terrain(terrain)
  , spatial(spatial)
  , fov(fov)
  , sprite(sprite)
  , atlas(file, "tiles/") {
  // TODO: Less verbose data entry.
  // TODO: Make the match to icon enum more obvious.
  entity_drawables.push_back(tile_drawable("terrain", 8, "#f0f"));  // invalid
  entity_drawables.push_back(tile_drawable("creatures", 1, "#a70")); // dreg
  entity_drawables.push_back(tile_drawable("creatures", 2, "#088")); // thrall
  entity_drawables.push_back(tile_drawable("creatures", 0, "#ccc")); // player
}

void Display_System::draw(Entity player, const Rectf& screen_rect) {
  // XXX: Currently fov is hardcoded to a single player entity, so the player
  // parameter gets no use. In future, there may be support for multiple fovs.
  glMatrixMode(GL_PROJECTION);
  glLoadIdentity();
  auto dim = screen_rect.dim();
  glOrtho(0, dim[0], dim[1], 0, -1, 1);

  Vec2f offset = (dim - tile_size) * .5f;

  glMatrixMode(GL_MODELVIEW);
  glLoadIdentity();

  Mtx<float, 3, 3> projection{
    16, -16, offset[0],
    8,   8,  offset[1],
    0,   0,  1};

  auto inv_projection = inverse(projection);

  std::vector<Vec2f> fov_vertices;
  for (auto& vtx : screen_rect.vertices())
    fov_vertices.push_back((inv_projection * vtx.homogenize()).dehomogenize());

  Rectf fov_sub_rect = Rectf::smallest_containing(fov_vertices.begin(), fov_vertices.end());
  Vec2i fov_min(floor(fov_sub_rect.min()[0]), floor(fov_sub_rect.min()[1]));
  Vec2i fov_max(ceil(fov_sub_rect.max()[0]), ceil(fov_sub_rect.max()[1]));

  glClear(GL_COLOR_BUFFER_BIT);

  std::set<Sprite> sprites;
  world_sprites(Recti(fov_min, fov_max - fov_min), sprites);
  for (auto sprite : sprites) {
    auto draw_pos = Vec2f(projection * Vec3f(sprite.pos[0], sprite.pos[1], 1));
    sprite.draw(draw_pos);
  }
}

void Display_System::world_sprites(const Recti& fov_rect, std::set<Sprite>& output) {
  const int terrain_layer = 0x10;
  const int entity_layer = 0x20;

  for (int y = fov_rect.min()[1]; y <= fov_rect.max()[1]; y++) {
    for (int x = fov_rect.min()[0]; x <= fov_rect.max()[0]; x++) {
      Vec2i offset(x, y);
      sprite.collect_sprites(offset, output);
      auto loc = fov.view_location(offset);
      if (loc.is_null())
        continue;

      bool in_fov = fov.is_seen(loc);

      auto ter = terrain_data[terrain.get(loc)];
      auto color = ter.color;
      auto icon = ter.icon;

      if (ter.kind & wallform_flag)
        icon += hex_wall(fov.wallform_mask(offset));

      if (!in_fov) // Darken terrain out of fov
        color = lerp(0.5, Color("black"), color.monochrome());
      auto terrain_tile = tile_drawable(ter.icon_set, icon, color);
      output.insert(Sprite{terrain_layer, offset, terrain_tile});

      if (in_fov) {
        for (auto& pair : spatial.entities_with_offsets_at(loc)) {
          Entity& entity = pair.second;
          auto& blob = entities.as<Blob_Part>(entity);
          output.insert(Sprite{entity_layer, offset + pair.first, entity_drawables[blob.icon]});
        }
      }
    }
  }
}

std::shared_ptr<Drawable> Display_System::tile_drawable(
  const char* set, int index, const Color& color, const Vec2f& offset) {
  index += atlas.frameset_start(set);
  Recti tile_rect = atlas.frame_rect(index);
  Vec2i tile_offset = atlas.offset(index);
  return std::shared_ptr<Drawable>(
    new Tile_Drawable(
      atlas.texture_id(),
      color,
      Tile_Rect{
        tile_rect.min()[0], tile_rect.min()[1],
        tile_rect.max()[0], tile_rect.max()[1],
        tile_offset[0], tile_offset[1]},
      atlas.get_dim(),
      offset));
}
