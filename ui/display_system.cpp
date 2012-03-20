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

static const Tile_Rect tile_rects[] = {
#include <tile_rect.hpp>
};

static const Surface tile_png {
#include <tile_atlas.hpp>
};

Display_System::Display_System(
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
    , tile_texture(tile_png) {
  // TODO: Less verbose data entry.
  entity_drawables.push_back(tile_drawable(8, "#f0f"));
  entity_drawables.push_back(tile_drawable(22, "#0f7"));
  entity_drawables.push_back(tile_drawable(24, "#fd0"));
  entity_drawables.push_back(tile_drawable(27, "#88f", -tile_size));
}

void Display_System::world_sprites(std::set<Sprite>& output) {
  const int terrain_layer = 0x10;
  const int entity_layer = 0x20;

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
        ter.icon,
        color);
      output.insert(Sprite{terrain_layer, offset, terrain_tile});

      if (in_fov) {
        for (auto& pair : spatial.entities_with_offsets_at(loc)) {
          Entity& entity = pair.second;
          auto& blob = entities.as<Blob_Part>(entity);
          if (blob.icon == icon_telos) {
            // TODO: Do this with components instead of a special case.
            output.insert(Sprite{entity_layer, offset + pair.first,
                  tile_drawable(27 + blob.base_facing % 3, "#88f", -tile_size)
                  });
            output.insert(Sprite{entity_layer + 1, offset + pair.first,
                  tile_drawable(30 + blob.turret_facing, "#ccf", -tile_size)
                  });
          } else {
            output.insert(Sprite{entity_layer, offset + pair.first, entity_drawables[blob.icon]});
          }
        }
      }
    }
  }
}

std::shared_ptr<Drawable> Display_System::tile_drawable(
  int index, const Color& color, const Vec2f& offset) {
  return std::shared_ptr<Drawable>(
    new Tile_Drawable(
      tile_texture.get(),
      color,
      tile_rects[index],
      tile_png.get_dim(),
      offset));
}
