#ifndef XEV_IMGUI_HPP
#define XEV_IMGUI_HPP

#include "vec.hpp"
#include "axis_box.hpp"
#include "util.hpp"

#define GEN_ID (xev::const_hash(__FILE__) + __LINE__)

namespace xev {

struct Imgui_State {
  Imgui_State() : pos{0, 0}, button(0) {}
  Vec2f pos;
  int button;
};

extern Imgui_State imgui_state;

bool im_button(int id, const char* title, const ARectf& bounds);

}

#endif
