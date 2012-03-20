/* surface.cpp

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

#include "surface.hpp"
#include <cstdlib>

extern "C" {
  extern uint8_t* stbi_load_from_memory(
    uint8_t const *buffer, int len, int *x, int *y, int *comp, int req_comp);

  extern uint8_t* stbi_load(
    char const *filename, int *x, int *y, int *comp, int req_comp);
}

Surface::Surface()
    : data_(nullptr)
    , width_(0)
    , height_(0) {}

Surface::Surface(const char* filename)
    : data_(nullptr)
    , width_(0)
    , height_(0) {
  load_image(filename);
}

Surface::Surface(std::initializer_list<uint8_t> args)
    : data_(nullptr)
    , width_(0)
    , height_(0) {
  load_image(args.begin(), args.size());
}

Surface::Surface(int width, int height)
    : data_(nullptr)
    , width_(0)
    , height_(0) {
  init_image(width, height);
}

Surface::Surface(const Vec2i& dim)
    : data_(nullptr)
    , width_(0)
    , height_(0) {
  init_image(dim);
}

Surface::~Surface() {
  // Need to use free since data may come from C code which malloc's it.
  free(data_);
}

void Surface::load_image(const uint8_t* buffer, size_t buffer_len) {
  free(data_);
  data_ = stbi_load_from_memory(buffer, buffer_len, &width_, &height_, nullptr, 4);
}

void Surface::load_image(const char* filename) {
  free(data_);
  data_ = stbi_load(filename, &width_, &height_, nullptr, 4);
  if (!data_) {
    throw "Unable to load file";
  }
}

void Surface::init_image(int width, int height) {
  free(data_);
  width_ = width;
  height_ = height;
  data_ = static_cast<uint8_t*>(malloc(width_ * height_ * 4));
  memset(data_, 0, width_ * height_ * 4);
}

Recti Surface::crop_rect() const {
  int x0 = width_, y0 = height_, x1 = 0, y1 = 0;
  for (int i = 0; i < width_ * height_; i++) {
    if ((*this)[i].a) {
      int x = i % width_, y = i / width_;
      if (x < x0) x0 = x;
      if (y < y0) y0 = y;
      if (x > x1) x1 = x;
      if (y > y1) y1 = y;
    }
  }

  if (x0 < x1 && y0 < y1)
    return Recti(Vec2i(x0, y0), Vec2i(x1 - x0 + 1, y1 - y0 + 1));
  else
    return Recti(Vec2i(0, 0));
}

void Surface::blit(const Recti& src_rect, Surface& dest, const Vec2i& dest_pos) {
  // XXX: Unoptimized.
  for (int y = src_rect.min()[1], e_y = src_rect.max()[1]; y < e_y; y++)
    for (int x = src_rect.min()[0], e_x = src_rect.max()[0]; x < e_x; x++) {
      Vec2i src_vec(x, y);
      Vec2i dest_vec = src_vec - src_rect.min() + dest_pos;
      if (contains(src_vec) && dest.contains(dest_vec))
        dest[dest_vec] = (*this)[src_vec];
    }
}
