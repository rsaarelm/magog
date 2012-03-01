#ifndef XEV_GLDRAW_HPP
#define XEV_GLDRAW_HPP

#include <xev/axis_box.hpp>

namespace xev {

void gl_rect(const ARectf& box);

void gl_tex_rect(const ARectf& box, const ARectf& texcoords);

}

#endif
