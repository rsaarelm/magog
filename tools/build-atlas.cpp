/* build-atlas.cpp

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

#include <util/surface.hpp>
#include <util/axis_box.hpp>
#define STB_IMAGE_WRITE_IMPLEMENTATION
#include <contrib/stb/stb_image_write.h>
#include <vector>
#include <list>
#include <memory>

void usage(int argc, char* argv[]) {
  fprintf(stderr, "Usage: %s [rectdata_output_file] [atlas_png_output_file] ([file] | -n [num_tiles])...\n", argv[0]);
  exit(1);
}

void pack(const std::vector<Vec2i>& dims, const ARecti& current_area,
          std::vector<Vec2i>& inout_positions, std::list<size_t>& inout_unplaced_indices) {
  auto index = inout_unplaced_indices.begin();
  ARecti place_rect;

  // Find the first unplaced item we can place.
  while (true) {
    if (index == inout_unplaced_indices.end()) {
      // Nothing left we can place.
      return;
    }
    place_rect = ARecti(current_area.min(), dims[*index]);
    if (current_area.contains(place_rect)) {
      // Erase the index of the thing we managed to place.
      inout_unplaced_indices.erase(index);
      break;
    }
    ++index;
  }

  // Place the first unplaced element into top-left of current area.
  inout_positions[*index] = current_area.min();

  ARecti recurse1, recurse2;
  // Split along the longer edge of the newly allocted rectangle.
  if (place_rect.dim()[1] > place_rect.dim()[0]) {
    // taller than wide.
    recurse1 = ARecti(current_area.min() + place_rect.dim().elem_mul(Vec2i(0, 1)),
                      Vec2i(place_rect.dim()[0], current_area.dim()[1] - place_rect.dim()[1]));
    recurse2 = ARecti(current_area.min() + place_rect.dim().elem_mul(Vec2i(1, 0)),
                      current_area.dim() - place_rect.dim().elem_mul(Vec2i(1, 0)));
  } else {
    // wider than tall or square.
    recurse1 = ARecti(current_area.min() + place_rect.dim().elem_mul(Vec2i(1, 0)),
                      Vec2i(current_area.dim()[0] - place_rect.dim()[0], place_rect.dim()[1]));
    recurse2 = ARecti(current_area.min() + place_rect.dim().elem_mul(Vec2i(0, 1)),
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

int main(int argc, char* argv[]) {
  if (argc < 4)
    usage(argc, argv);
  std::vector<std::unique_ptr<Surface>> images;
  std::vector<Vec2i> dims;
  std::vector<Vec2i> offsets;
  long num_pixels = 0;
  for (int i = 3; i < argc; i++) {
    int n_tiles = 1;
    const char* arg = argv[i];
    if (arg[0] == '-') {
      if (strcmp(arg, "-n") == 0) {
        i++;
        if (i >= argc)
          usage(argc, argv);
        n_tiles = atoi(argv[i]);
        if (n_tiles < 1)
          usage(argc, argv);
        i++;
        if (i >= argc)
          break;
      } else {
        usage(argc, argv);
      }
    }
    Surface tiles(argv[i]);
    // Extract horizontal tile strips.
    for (int j = 0; j < n_tiles; j++) {
      int width = tiles.get_dim()[0] / n_tiles;
      int height = tiles.get_dim()[1];
      Surface* img = new Surface(width, height);
      tiles.blit(
        ARecti(Vec2i(width * j, 0), Vec2i(width, height)),
        *img,
        Vec2i(0, 0));
      ARecti rect = img->crop_rect();
      num_pixels += rect.volume();
      images.push_back(std::unique_ptr<Surface>(img));
      dims.push_back(rect.dim());
      offsets.push_back(rect.min());
    }
  }

  // Get the smallest power-of-two square texture size that fits the
  // number of pixels seen.
  int size = 1;
  while (size * size < num_pixels)
    size <<= 1;

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

    pack(dims, ARecti(Vec2i(size, size)), packed, indices);

    if (!indices.empty())
      size <<= 1;
    else
      break;
  }

  FILE* rectdata = fopen(argv[1], "w");
  for (int i = 0; i < packed.size(); i++) {
    Vec2i p1 = packed[i];
    Vec2i p2 = packed[i] + dims[i];
    fprintf(rectdata, "{%d, %d, %d, %d, %d, %d},\n",
            p1[0], p1[1], p2[0], p2[1], offsets[i][0], offsets[i][1]);
  }
  fclose(rectdata);

  Surface canvas(size, size);
  for (int i = 0; i < packed.size(); i++)
    images[i]->blit(ARecti(offsets[i], dims[i]), canvas, packed[i]);

  int result = stbi_write_png(argv[2], canvas.get_dim()[0], canvas.get_dim()[1], 4, canvas.data(), 0);
  if (!result)
    return 1;

  return 0;
}
