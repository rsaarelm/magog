/* surface.hpp

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

#ifndef UTIL_SURFACE_HPP
#define UTIL_SURFACE_HPP

#include "static_file.hpp"
#include "color.hpp"
#include "axis_box.hpp"
#include <GL/gl.h>
#include <cstdlib>
#include <algorithm>
#include <vector>

class Surface {
 public:
  Surface();
  Surface(const char* filename);
  Surface(const Static_File* file);
  Surface(int width, int height);
  Surface(const Vec2i& dim);
  ~Surface();

  void load_image(const uint8_t* buffer, size_t buffer_len);
  void load_image(const Static_File* file);
  void load_image(const char* filename);

  void init_image(int width, int height);
  void init_image(const Vec2i& dim) { init_image(dim[0], dim[1]); }
  Vec2i get_dim() const { return Vec2i(width, height); }

  uint8_t* get_data() { return data; }

  Color& operator[](int i) { return reinterpret_cast<Color*>(data)[i]; }

  Color operator[](int i) const { return reinterpret_cast<Color*>(data)[i]; }

  Color& operator[](const Vec2i& pos) {
    return (*this)[pos[0] + pos[1]*width];
  }

  Color operator[](const Vec2i& pos) const {
    return (*this)[pos[0] + pos[1]*width];
  }

  GLuint make_texture();

  /// Get the smallest rectangle containing all of the surface's
  /// non-transparent pixels.
  ARecti crop_rect() const;

  void blit(const ARecti& src_rect, Surface& dest, const Vec2i& dest_pos);

  bool contains(const Vec2i& pos) {
    return pos[0] >= 0 && pos[1] >= 0 && pos[0] < width && pos[1] < height;
  }
 private:
  Surface(const Surface&);
  Surface& operator=(const Surface&);

  uint8_t* data;

  int width;
  int height;
};

#endif
