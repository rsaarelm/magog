#include <GL/glew.h>
#include "gldraw.hpp"

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
