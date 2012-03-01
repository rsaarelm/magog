#ifndef XEV_FONT_HPP
#define XEV_FONT_HPP

#include <cstddef>
#include <xev/contrib/stb/stb_truetype.h>
#include <xev/surface.hpp>
#include <xev/vec.hpp>

namespace xev {

struct Font_Data {
  int font_height;
  int w, h;
  stbtt_bakedchar* chars;
  const char* data;
};

void init_font(const Font_Data& data);

int text_width(const char* text);

int font_height();

int draw_text(const Vec2f& pos, const char* fmt, ...);

}

#endif
