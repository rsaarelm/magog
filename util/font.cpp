// Copyright (C) 2012 Risto Saarelma

#include "font.hpp"
#include <cstdarg>
#include <GL/glew.h>
#include "core.hpp"
#include "gldraw.hpp"

static GLuint g_font_tex = 0;

static Font_Data g_fontdata_font;

void init_font(const Font_Data& data) {
  g_fontdata_font = data;
  glGenTextures(1, &g_font_tex);
  glBindTexture(GL_TEXTURE_2D, g_font_tex);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
  glTexImage2D(
      GL_TEXTURE_2D, 0, GL_ALPHA, g_fontdata_font.w, g_fontdata_font.h,
      0, GL_ALPHA, GL_UNSIGNED_BYTE, g_fontdata_font.data);
}

int draw_char(Vec2f pos, char ch) {
  if (!g_font_tex) die("Font not initialized");

  pos[1] += g_fontdata_font.font_height;

  glBindTexture(GL_TEXTURE_2D, g_font_tex);
  auto chdata = g_fontdata_font.chars[ch - 32];

  Vec2f offset = Vec2f(chdata.xoff, chdata.yoff) + pos;
  Vec2f origin(chdata.x0, chdata.y0);
  Vec2f dim(chdata.x1 - chdata.x0, chdata.y1 - chdata.y0);
  Vec2f tex_scale(g_fontdata_font.w, g_fontdata_font.h);

  gl_tex_rect(ARectf(offset, dim), ARectf(origin.elem_div(tex_scale), dim.elem_div(tex_scale)));

  return chdata.xadvance;
}

int text_width(const char* text) {
  int result = 0;
  for (const char* c = text; *c; c++)
    result += g_fontdata_font.chars[*c - 32].xadvance;
  return result;
}

int font_height() {
  return g_fontdata_font.font_height;
}

int draw_text_raw(const Vec2f& pos, const char* text) {
  int result = 0;
  for (const char* c = text; *c; c++)
    result += draw_char(pos + Vec2f(result, 0), *c);
  return result;
}

// XXX: Not thread safe, using a huge static buffer to print the text in
// because too lazy to do a proper buffer.
int draw_text(const Vec2f& pos, const char* fmt, ...) {
  static char buffer[16384];
  va_list ap;
  va_start(ap, fmt);
  vsnprintf(buffer, sizeof(buffer), fmt, ap);
  return draw_text_raw(pos, buffer);
}
