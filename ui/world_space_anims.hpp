/* world_space_anims.hpp

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

#ifndef WORLD_SPACE_ANIMS_HPP
#define WORLD_SPACE_ANIMS_HPP

#include "drawable.hpp"
#include "sprite.hpp"
#include <world/spatial_index.hpp>
#include <world/fov_system.hpp>
#include <util/vec.hpp>
#include <map>
#include <set>
#include <queue>
#include <memory>

class World_Space_Anims {
public:
  World_Space_Anims(Fov_System& fov)
  : fov(fov) {}

  void collect_sprites(const Vec2i& view_space_pos, std::set<Sprite>& output);

  void add(const std::shared_ptr<Drawable>& drawable, Location loc);
  void add(const std::shared_ptr<Drawable>& drawable, const Footprint& footprint);

  void update(float interval_sec);
private:
  World_Space_Anims(const World_Space_Anims&);
  World_Space_Anims& operator=(const World_Space_Anims&);

  typedef std::pair<std::shared_ptr<Drawable>, Footprint> Element;

  void remove(Element element);

  Fov_System& fov;

  std::queue<Element> drawables;

  Spatial_Index<std::shared_ptr<Drawable>> index;
};

#endif
