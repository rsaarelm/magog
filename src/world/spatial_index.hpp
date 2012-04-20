/* spatial_index.hpp

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
#ifndef WORLD_SPATIAL_INDEX_HPP
#define WORLD_SPATIAL_INDEX_HPP

#include <world/location.hpp>
#include <util/vec.hpp>
#include <util/core.hpp>
#include <map>

template <class C>
class Spatial_Index {
public:
  typedef typename std::multimap<Plain_Location, std::pair<Vec2i, C>>::iterator iterator;
  typedef typename std::multimap<Plain_Location, std::pair<Vec2i, C>>::const_iterator const_iterator;

  Spatial_Index() {}

  void add(const C& element, const Footprint& footprint) {
    ASSERT(footprints.find(element) == footprints.end());

    footprints[element] = footprint;

    for (auto i : footprint) {
      auto& offset = i.first;
      auto& location = i.second;
      contents.insert(std::make_pair(
                        location,
                        std::make_pair(-offset, element)));
    }
  }

  void remove(const C& element) {
    auto foot = footprints.find(element);
    ASSERT(foot != footprints.end());

    size_t sanity_check = foot->second.size();

    for (auto& foot_elem : foot->second) {
      auto& foot_offset = foot_elem.first;
      auto loc_pair = contents.equal_range(foot_elem.second);
      for (auto obj = loc_pair.first; obj != loc_pair.second;) {
        auto& offset = obj->second.first;
        auto& value = obj->second.second;

        if (offset == -foot_offset && value == element) {
          contents.erase(obj++);
          sanity_check--;
        } else {
          ++obj;
        }
      }
    }
    ASSERT(sanity_check == 0);
    footprints.erase(foot);
  }

  std::pair<iterator, iterator> equal_range(Plain_Location location) {
    return contents.equal_range(location);
  }

  std::pair<const_iterator, const_iterator> equal_range(Plain_Location location) const {
    return contents.equal_range(location);
  }

  bool has(const C& element) const {
    return footprints.find(element) != footprints.end();
  }
private:
  Spatial_Index(const Spatial_Index&);
  Spatial_Index& operator=(const Spatial_Index&);

  std::map<C, Footprint> footprints;
  std::multimap<Plain_Location, std::pair<Vec2i, C>> contents;
};

#endif
