/* gl_texture.hpp

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
#ifndef UTIL_GL_TEXTURE_HPP
#define UTIL_GL_TEXTURE_HPP

#include <util/vec.hpp>
#include <GL/gl.h>

class Surface;

/// RAII wrapper for OpenGL textures
struct Gl_Texture {
  Gl_Texture() : handle(0) {}
  Gl_Texture(const Surface& surface);

  ~Gl_Texture() { free(); }

  Gl_Texture(Gl_Texture&& rhs) { *this = rhs; }

  Gl_Texture& operator=(Gl_Texture&& rhs) {
    free();
    handle = rhs.handle;
    dim = rhs.dim;
    rhs.handle = 0;
  }

  GLuint get() const { return handle; }

  void bind() const { glBindTexture(GL_TEXTURE_2D, handle); }

  Vec2i get_dim() const { return dim; }
private:
  Gl_Texture(GLuint handle) : handle(handle) {}
  Gl_Texture(const Gl_Texture&);
  Gl_Texture& operator=(const Gl_Texture&);

  void free() {
    if (handle)
      glDeleteTextures(1, &handle);
  }

  GLuint handle;
  Vec2i dim;
};

#endif
