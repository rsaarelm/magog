// Copyright (C) 2012 Risto Saarelma

#include "imgui.hpp"
#include <GL/glew.h>
#include "color.hpp"
#include "font.hpp"
#include "gldraw.hpp"

Imgui_State imgui_state;

bool im_button(int id, const char* title, const ARectf& bounds) {
  bool hit = bounds.contains(imgui_state.pos);
  if (hit) {
    glColor4f(0, .50, 0, 1);
  } else {
    glColor4f(0, .25, 0, 1);
  }
  gl_rect(bounds);

  Vec2f dim(text_width(title), font_height());
  Vec2f centering_tweak = Vec2f(0, -font_height() / 5);
  Vec2f pos = bounds.min() + (bounds.dim() - dim) / 2.f + centering_tweak;
  Color(255, 255, 255).gl_color();
  draw_text(pos, title);
  return hit && imgui_state.button;
}
