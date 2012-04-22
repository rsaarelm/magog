/* atlas.cpp

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

#include "atlas.hpp"
#include <util/surface.hpp>
#include <list>
#include <memory>

/// Load a bitmap and generate surfaces from its N subtiles.
void process(
  const File_System& file,
  const char* filename,
  int num_tiles,
  std::vector<std::unique_ptr<Surface>>& output,
  std::vector<Vec2i>& offsets) {
  std::vector<uint8_t> png_file = file.read(filename);
  Surface master(png_file);
  int width = master.get_dim()[0] / num_tiles;
  int height = master.get_dim()[1];
  for (int i = 0; i < num_tiles; i++) {
    Vec2i origin = Vec2i(0 + width * i, 0);
    Recti crop = master.crop_rect(Recti(origin, {width, height}));
    std::unique_ptr<Surface> result = std::unique_ptr<Surface>(new Surface(crop.dim()));
    master.blit(crop, *result, Vec2i(0, 0));
    output.push_back(std::move(result));
    offsets.push_back(crop.min() - origin);
  }
}

/// Pack rectangles, whose sizes are in dims, in current_area, output to
/// inout_positions. Put indices of elements that didn't fit in
/// inout_unplaced_indices.
void pack(const std::vector<Vec2i>& dims, const Recti& current_area,
          std::vector<Vec2i>& inout_positions, std::list<size_t>& inout_unplaced_indices) {
  auto index = inout_unplaced_indices.begin();
  Recti place_rect;

  // Find the first unplaced item we can place.
  while (true) {
    if (index == inout_unplaced_indices.end()) {
      // Nothing left we can place.
      return;
    }
    place_rect = Recti(current_area.min(), dims[*index]);
    if (current_area.contains(place_rect)) {
      // Erase the index of the thing we managed to place.
      inout_unplaced_indices.erase(index);
      break;
    }
    ++index;
  }

  // Place the first unplaced element into top-left of current area.
  inout_positions[*index] = current_area.min();

  Recti recurse1, recurse2;
  // Split along the longer edge of the newly allocted rectangle.
  if (place_rect.dim()[1] > place_rect.dim()[0]) {
    // taller than wide.
    recurse1 = Recti(current_area.min() + place_rect.dim().elem_mul(Vec2i(0, 1)),
                      Vec2i(place_rect.dim()[0], current_area.dim()[1] - place_rect.dim()[1]));
    recurse2 = Recti(current_area.min() + place_rect.dim().elem_mul(Vec2i(1, 0)),
                      current_area.dim() - place_rect.dim().elem_mul(Vec2i(1, 0)));
  } else {
    // wider than tall or square.
    recurse1 = Recti(current_area.min() + place_rect.dim().elem_mul(Vec2i(1, 0)),
                      Vec2i(current_area.dim()[0] - place_rect.dim()[0], place_rect.dim()[1]));
    recurse2 = Recti(current_area.min() + place_rect.dim().elem_mul(Vec2i(0, 1)),
                      current_area.dim() - place_rect.dim().elem_mul(Vec2i(0, 1)));
  }
  ASSERT(!place_rect.intersects(recurse1));
  ASSERT(!place_rect.intersects(recurse2));
  ASSERT(current_area.contains(recurse1));
  ASSERT(current_area.contains(recurse2));
  ASSERT(!recurse1.intersects(recurse2));
  ASSERT(place_rect.volume() + recurse1.volume() + recurse2.volume() == current_area.volume());

  pack(dims, recurse1, inout_positions, inout_unplaced_indices);
  pack(dims, recurse2, inout_positions, inout_unplaced_indices);
}

void Atlas::init(File_System& file, const char* root_path) {
  std::vector<std::unique_ptr<Surface>> surfaces;
  int current_frameset_start = 0;

  // Load individual tiles.
  for (auto& a : file.list_files(root_path)) {
    // Expect file name to start with the number of tiles, atoi will parse
    // that. Default to 1 if parse fails.
    int num_tiles = atoi(a.c_str());
    if (num_tiles < 1)
      num_tiles = 1;

    // Extract the name from filenames of format "123-name.png"
    std::string name;
    int i = 0;
    for (; i < a.size() && (isdigit(a[i]) || a[i] == '-'); i++) ;
    while (i < a.size() && a[i] != '.')
      name += a[i++];

    // Store the frameset name.
    framesets[name] = current_frameset_start;
    current_frameset_start += num_tiles;

    std::string path(root_path);
    path += a;
    process(file, path.c_str(), num_tiles, surfaces, offsets);
  }

  // Get the smallest power-of-two dimensional texture size that fits the
  // number of pixels seen.
  std::vector<Vec2i> dims;

  int pixel_count = 0;
  for (auto& i : surfaces) {
    pixel_count += i->width() * i->height();
    dims.push_back(i->get_dim());
  }

  int width = 1, height = 1;
  while (width * height < pixel_count) {
    if (width > height)
      height <<= 1;
    else
      width <<= 1;
  }

  std::vector<Vec2i> packed;

  for (;;) {
    packed.clear();
    packed.resize(dims.size());
    std::list<size_t> indices;
    for (size_t i = 0; i < dims.size(); i++) indices.push_back(i);

    // Sort indices into order of descending size. Pack the biggest sprite
    // first.
    indices.sort([&](const size_t& a, const size_t& b) {
        return dims[a][0] * dims[a][1] > dims[b][0] * dims[b][1]; });

    pack(dims, Recti(Vec2i(width, height)), packed, indices);

    if (!indices.empty()) {
      if (width > height)
        height <<= 1;
      else
        width <<= 1;
    }
    else
      break;
  }

  Surface atlas(width, height);

  for (int i = 0; i < dims.size(); i++) {
    Recti rect(packed[i], dims[i]);
    frames.push_back(rect);
    surfaces[i]->blit(Recti(dims[i]), atlas, packed[i]);
  }

  atlas_texture = Gl_Texture(atlas);
}
