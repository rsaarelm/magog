// Copyright (C) 2012 Risto Saarelma

#ifndef UTIL_FONT_HPP
#define UTIL_FONT_HPP

#include "vec.hpp"

void init_font();

int text_width(const char* text);

int font_height();

int draw_text(const Vec2f& pos, const char* fmt, ...);

#endif
