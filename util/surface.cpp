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
    : data(nullptr)
    , width(0)
    , height(0) {}

// XXX: Use delegating constructors when gcc supports them.
Surface::Surface(const Static_File* file)
    : data(nullptr)
    , width(0)
    , height(0) {
  load_image(file);
}

Surface::Surface(const char* filename)
    : data(nullptr)
    , width(0)
    , height(0) {
  load_image(filename);
}

Surface::Surface(int width, int height)
    : data(nullptr)
    , width(0)
    , height(0) {
  init_image(width, height);
}

Surface::Surface(const Vec2i& dim)
    : data(nullptr)
    , width(0)
    , height(0) {
  init_image(dim);
}

Surface::~Surface() {
  // Need to use free since data may come from C code which malloc's it.
  free(data);
}

void Surface::load_image(const uint8_t* buffer, size_t buffer_len) {
  free(data);
  data = stbi_load_from_memory(buffer, buffer_len, &width, &height, nullptr, 4);
}

void Surface::load_image(const Static_File* file) { load_image(file->get_data(), file->get_len()); }

void Surface::load_image(const char* filename) {
  free(data);
  data = stbi_load(filename, &width, &height, nullptr, 4);
  if (!data) {
    throw "Unable to load file";
  }
}

void Surface::init_image(int width, int height) {
  free(data);
  this->width = width;
  this->height = height;
  data = static_cast<uint8_t*>(malloc(width * height * 4));
}

GLuint Surface::make_texture() {
  GLuint result;
  glGenTextures(1, &result);
  glBindTexture(GL_TEXTURE_2D, result);
  // TODO: Support other types of filtering.
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
  glTexImage2D(
      GL_TEXTURE_2D, 0, GL_RGBA8, width, height,
      0, GL_RGBA, GL_UNSIGNED_BYTE, data);
  return result;
}

ARecti Surface::crop_rect() const {
  int x0 = width, y0 = height, x1 = 0, y1 = 0;
  for (int i = 0; i < width * height; i++) {
    if ((*this)[i].a) {
      int x = i % width, y = i / width;
      if (x < x0) x0 = x;
      if (y < y0) y0 = y;
      if (x > x1) x1 = x;
      if (y > y1) y1 = y;
    }
  }

  if (x0 < x1 && y0 < y1)
    return ARecti(Vec2i(x0, y0), Vec2i(x1 - x0, y1 - y0));
  else
    return ARecti(Vec2i(0, 0));
}

void Surface::blit(const ARecti& src_rect, Surface& dest, const Vec2i& dest_pos) {
  // XXX: Unoptimized.
  for (int y = src_rect.min()[1], e_y = src_rect.max()[1]; y < e_y; y++)
    for (int x = src_rect.min()[0], e_x = src_rect.max()[0]; x < e_x; x++) {
      Vec2i src_vec(x, y);
      Vec2i dest_vec = src_vec - src_rect.min() + dest_pos;
      if (contains(src_vec) && dest.contains(dest_vec))
        dest[dest_vec] = (*this)[src_vec];
    }
}
