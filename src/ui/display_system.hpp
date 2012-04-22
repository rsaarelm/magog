/* display_system.hpp

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
#ifndef UI_DISPLAY_SYSTEM_HPP
#define UI_DISPLAY_SYSTEM_HPP

#include <ui/sprite_system.hpp>
#include <world/entities_system.hpp>
#include <world/terrain_system.hpp>
#include <world/spatial_system.hpp>
#include <world/fov_system.hpp>
#include <util/atlas.hpp>
#include <util/file_system.hpp>
#include <util/gl_texture.hpp>
#include <util/color.hpp>
#include <util/vec.hpp>
#include <util/mtx.hpp>
#include <util/box.hpp>
#include <set>
#include <memory>

const Vec2f tile_size(16, 16);

const Mtx<float, 2, 2> tile_projection{
  tile_size[0],    -tile_size[0],
  tile_size[1] / 2, tile_size[1] / 2};

const Mtx<float, 2, 2> tile_projection_inv = inverse(tile_projection);

class Display_System {
public:
  Display_System(
    File_System& file,
    Entities_System& entities,
    Terrain_System& terrain,
    Spatial_System& spatial,
    Fov_System& fov,
    Sprite_System& sprite);

  void draw(Entity player, const Rectf& screen_rect);

  void world_sprites(const Recti& fov_rect, std::set<Sprite>& output);

  std::shared_ptr<Drawable> tile_drawable(
    const char* frameset, int index, const Color& color,
    const Vec2f& offset = Vec2f(0, 0));
private:

  Entities_System& entities;
  Terrain_System& terrain;
  Spatial_System& spatial;
  Fov_System& fov;
  Sprite_System& sprite;

  Atlas atlas;

  std::vector<std::shared_ptr<Drawable>> entity_drawables;
};

#endif
