/* world_space_anims.cpp

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

#include "world_space_anims.hpp"
#include <world/world.hpp>
#include <util/alg.hpp>

void World_Space_Anims::collect_sprites(
  const Vec2i& view_space_pos, std::set<Sprite>& output) {
  auto loc = view_space_location(view_space_pos);
  if (loc) {
    auto pair = locations.equal_range(*loc);
    for (auto i = pair.first; i != pair.second; i++) {
      auto& offset = i->second.first;
      auto& drawable = i->second.second;

      output.insert(Sprite{
          drawable->get_z_layer(),
          offset + view_space_pos,
          *drawable});
    }
  }
}

void World_Space_Anims::add(
  std::unique_ptr<Drawable> drawable, const Footprint& footprint) {
  ASSERT(footprint.size() > 0);

  Drawable* ptr = drawable.get();
  World_Space_Anims::Element element(std::move(drawable), footprint);

  for (auto offset_location : footprint) {
    locations.insert(std::make_pair(
                     offset_location.second,
                     std::make_pair(-offset_location.first, ptr)));
  }

  drawables.push(std::move(element));
}

void World_Space_Anims::add(
  std::unique_ptr<Drawable> drawable, const Location& location) {
  add(std::move(drawable), drawable->footprint(location));
}

void World_Space_Anims::update(float interval_sec) {
  for (size_t i = 0, j = drawables.size(); i < j; i++) {
    Element element = std::move(drawables.front());
    drawables.pop();
    bool is_alive = element.first->update(interval_sec);
    if (is_alive)
      drawables.push(std::move(element));
    else
      remove(std::move(element));
  }
}

void World_Space_Anims::remove(Element element) {
  Drawable* ptr = element.first.get();
  size_t sanity_check = element.second.size();
  ASSERT(sanity_check > 0); // Must have some footprint elements to begin with.

  for (auto& pair : element.second) {
    auto& footprint_offset = pair.first;
    auto loc_pair = locations.equal_range(pair.second);
    for (auto i = loc_pair.first; i != loc_pair.second;) {
      auto& location_offset = i->second.first;
      Drawable* location_ptr = i->second.second;
      if (location_offset == -footprint_offset && location_ptr == ptr) {
        locations.erase(i++);
        sanity_check--;
      } else {
        ++i;
      }
    }
  }
  ASSERT(sanity_check == 0);
}
