/* gldraw.cpp

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

#include "gldraw.hpp"
#include <util/surface.hpp>

GLuint make_texture(const Surface& surface) {
  GLuint result;
  glGenTextures(1, &result);
  glBindTexture(GL_TEXTURE_2D, result);
  // TODO: Support other types of filtering.
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
  glTexImage2D(
    GL_TEXTURE_2D, 0, GL_RGBA8, surface.width(), surface.height(),
    0, GL_RGBA, GL_UNSIGNED_BYTE, surface.data());
  return result;
}

void gl_rect(const ARectf& box) {
  glBindTexture(GL_TEXTURE_2D, 0);
  glBegin(GL_QUADS);
  glVertex2f(box.min()[0], box.min()[1]);
  glVertex2f(box.max()[0], box.min()[1]);
  glVertex2f(box.max()[0], box.max()[1]);
  glVertex2f(box.min()[0], box.max()[1]);
  glEnd();
}

void gl_tex_rect(const ARectf& box, const ARectf& texcoords) {
  glBegin(GL_QUADS);
  glTexCoord2f(texcoords.min()[0], texcoords.min()[1]);
  glVertex2f(box.min()[0], box.min()[1]);
  glTexCoord2f(texcoords.max()[0], texcoords.min()[1]);
  glVertex2f(box.max()[0], box.min()[1]);
  glTexCoord2f(texcoords.max()[0], texcoords.max()[1]);
  glVertex2f(box.max()[0], box.max()[1]);
  glTexCoord2f(texcoords.min()[0], texcoords.max()[1]);
  glVertex2f(box.min()[0], box.max()[1]);
  glEnd();
}
