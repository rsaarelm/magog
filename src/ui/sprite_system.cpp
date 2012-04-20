/* sprite_system.cpp

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

#include "sprite_system.hpp"
#include <util/alg.hpp>

void Sprite_System::collect_sprites(
  const Vec2i& view_space_pos, std::set<Sprite>& output) {
  auto loc = fov.view_location(view_space_pos);
  if (!loc.is_null()) {
    auto pair = index.equal_range(loc);
    for (auto i = pair.first; i != pair.second; i++) {
      auto& offset = i->second.first;
      auto& drawable = i->second.second;

      output.insert(Sprite{
          drawable->get_z_layer(),
          offset + view_space_pos,
          drawable});
    }
  }
}

void Sprite_System::add(
  const std::shared_ptr<Drawable>& drawable, const Footprint& footprint) {
  ASSERT(footprint.size() > 0);

  index.add(drawable, footprint);

  Sprite_System::Element element(drawable, footprint);
  drawables.push(element);
}

void Sprite_System::add(
  const std::shared_ptr<Drawable>& drawable, Location location) {
  add(drawable, drawable->footprint(location));
}

void Sprite_System::update(float interval_sec) {
  for (size_t i = 0, j = drawables.size(); i < j; i++) {
    Element element = drawables.front();
    drawables.pop();
    bool is_alive = element.first->update(interval_sec);
    if (is_alive)
      drawables.push(element);
    else
      remove(element);
  }
}

void Sprite_System::remove(Element element) {
  index.remove(element.first);
}
