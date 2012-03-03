#ifndef UTIL_IMGUI_HPP
#define UTIL_IMGUI_HPP

#include "vec.hpp"
#include "axis_box.hpp"
#include "core.hpp"

#define GEN_ID (const_hash(__FILE__) + __LINE__)

struct Imgui_State {
  Imgui_State() : pos{0, 0}, button(0) {}
  Vec2f pos;
  int button;
};

extern Imgui_State imgui_state;

bool im_button(int id, const char* title, const ARectf& bounds);

#endif
