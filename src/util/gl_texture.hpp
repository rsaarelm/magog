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

#include <GL/gl.h>

class Surface;

/// RAII wrapper for OpenGL textures
struct Gl_Texture {
  Gl_Texture(const Surface& surface);

  ~Gl_Texture() {
    glDeleteTextures(1, &handle);
  }

  GLuint get() const { return handle; }

  void bind() { glBindTexture(GL_TEXTURE_2D, handle); }

  private:
  Gl_Texture(GLuint handle) : handle(handle) {}

  Gl_Texture(const Gl_Texture&);
  Gl_Texture& operator=(const Gl_Texture&);

  GLuint handle;
};

#endif
