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
#include <memory>

void usage(int argc, char* argv[]) {
  fprintf(stderr, "Usage: %s [rectdata_output_file] [atlas_png_output_file] [file]...\n");
  exit(1);
}

// Pack as many rectangles as you can into the result, then return it.
std::vector<Vec2i> try_pack(const std::vector<Vec2i>& dims, int width, int height) {
  // XXX A very stupid packer.
  std::vector<Vec2i> result;
  int x = 0;
  int y = 0;
  int max_y = 0;
  for (auto& dim : dims) {
    if (dim[1] > max_y)
      max_y = dim[1];
    if (dim[0] + x > width) {
      // New line
      x = 0;
      y += max_y;
    }
    if (dim[1] + y > height) {
      // Can't fit any more.
      return result;
    }
    result.push_back(Vec2i(x, y));
    x += dim[0];
  }
  return result;
}

int main(int argc, char* argv[]) {
  if (argc < 4)
    usage(argc, argv);
  std::vector<std::unique_ptr<Surface>> images;
  std::vector<Vec2i> dims;
  std::vector<Vec2i> offsets;
  long num_pixels = 0;
  for (int i = 3; i < argc; i++) {
    Surface* img = new Surface(argv[i]);
    ARecti rect = img->crop_rect();
    num_pixels += rect.volume();
    images.push_back(std::unique_ptr<Surface>(img));
    dims.push_back(rect.dim());
    offsets.push_back(rect.min());
  }

  // Get the smallest power-of-two square texture size that fits the
  // number of pixels seen.
  int size = 1;
  while (size * size < num_pixels)
    size <<= 1;

  std::vector<Vec2i> pack;
  for (;;) {
    pack = try_pack(dims, size, size);
    // Keep growing the bin until packing succeeds.
    if (pack.size() < dims.size())
      size <<= 1;
    else
      break;
  }

  FILE* rectdata = fopen(argv[1], "w");
  for (int i = 0; i < pack.size(); i++) {
    Vec2i p1 = pack[i];
    Vec2i p2 = pack[i] + dims[i];
    fprintf(rectdata, "{%d, %d, %d, %d, %d, %d},\n",
            p1[0], p1[1], p2[0], p2[1], offsets[i][0], offsets[i][1]);
  }
  fclose(rectdata);

  printf("Need %d x %d texture\n", size, size);

  Surface canvas(size, size);
  for (int i = 0; i < pack.size(); i++)
    images[i]->blit(ARecti(offsets[i], dims[i]), canvas, pack[i]);

  printf("%d %d\n", canvas.get_dim()[0], canvas.get_dim()[1]);
  int result = stbi_write_png(argv[2], canvas.get_dim()[0], canvas.get_dim()[1], 4, canvas.get_data(), 0);
  if (!result)
    return 1;

  return 0;
}
