/* sdl_util.cpp

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

#include "sdl_util.hpp"
#include <util/core.hpp>
#include <util/format.hpp>
#define STB_IMAGE_WRITE_IMPLEMENTATION
#include <contrib/stb/stb_image_write.h>
#include <time.h>
#include <GL/gl.h>
#include <vector>

int write_png(const char* filename, SDL_Surface* surface) {
  int comp = 0;

  ASSERT(surface);

  int lock_success = SDL_LockSurface(surface);
  if (lock_success)
    die("Surface locking failed: %s", SDL_GetError());

  ASSERT(surface->pixels);

  if (surface->format->BitsPerPixel == 24)
    comp = 3;
  else if (surface->format->BitsPerPixel == 32)
    comp = 4;
  else
    die("Unsupported bits per pixel %d", surface->format->BitsPerPixel);

  int result = stbi_write_png(filename, surface->w, surface->h, comp, surface->pixels, surface->pitch);

  SDL_UnlockSurface(surface);
  return result;
}

SDL_Surface* opengl_screen_to_surface() {
  SDL_Surface* screen = SDL_GetVideoSurface();
  SDL_Surface* buffer = SDL_CreateRGBSurface(
    SDL_SWSURFACE, screen->w, screen->h, 24,
    0x000000ff, 0x0000ff00, 0x00ff0000, 0);

  int pitch = screen->w * 3;

  // Get OpenGL pixels. These will have the lines going from bottom to top.
  std::vector<uint8_t> pixels;
  pixels.resize(screen->w * screen->h * 3);
  SDL_LockSurface(screen);
  glReadPixels(0, 0, screen->w, screen->h, GL_RGB, GL_UNSIGNED_BYTE, pixels.data());
  SDL_UnlockSurface(screen);

  // Reverse the lines for the SDL buffer.
  for (int y = 0; y < screen->h; y++)
    memcpy(
      buffer->pixels + (y * buffer->pitch),
      pixels.data() + (screen->h - y - 1) * pitch,
      pitch);

  return buffer;
}

void screenshot(const char* prefix) {
  // Need to create a temp surface because of OpenGL.
  SDL_Surface* buffer = opengl_screen_to_surface();

  std::string filename = format("%s-%s.png", prefix, clock());
  write_png(filename.c_str(), buffer);
  SDL_FreeSurface(buffer);
}

#if defined (MACOSX)
static SDLKey scancode_map[] = {
  SDLK_a,
  SDLK_s,
  SDLK_d,
  SDLK_f,
  SDLK_h,
  SDLK_g,
  SDLK_z,
  SDLK_x,
  SDLK_c,
  SDLK_v,
  SDLK_WORLD_0,
  SDLK_b,
  SDLK_q,
  SDLK_w,
  SDLK_e,
  SDLK_r,
  SDLK_y,
  SDLK_t,
  SDLK_1,
  SDLK_2,
  SDLK_3,
  SDLK_4,
  SDLK_6,
  SDLK_5,
  SDLK_EQUALS,
  SDLK_9,
  SDLK_7,
  SDLK_MINUS,
  SDLK_8,
  SDLK_0,
  SDLK_RIGHTBRACKET,
  SDLK_o,
  SDLK_u,
  SDLK_LEFTBRACKET,
  SDLK_i,
  SDLK_p,
  SDLK_RETURN,
  SDLK_l,
  SDLK_j,
  SDLK_QUOTE,
  SDLK_k,
  SDLK_SEMICOLON,
  SDLK_BACKSLASH,
  SDLK_COMMA,
  SDLK_SLASH,
  SDLK_n,
  SDLK_m,
  SDLK_PERIOD,
  SDLK_TAB,
  SDLK_SPACE,
  SDLK_BACKQUOTE,
  SDLK_BACKSPACE,
  SDLK_UNKNOWN,
  SDLK_ESCAPE,
  SDLK_UNKNOWN,
  SDLK_LMETA,
  SDLK_LSHIFT,
  SDLK_CAPSLOCK,
  SDLK_LALT,
  SDLK_LCTRL,
  SDLK_UNKNOWN,
  SDLK_UNKNOWN,
  SDLK_UNKNOWN,
  SDLK_UNKNOWN,
  SDLK_UNKNOWN,
  SDLK_KP_PERIOD,
  SDLK_UNKNOWN,
  SDLK_KP_MULTIPLY,
  SDLK_UNKNOWN,
  SDLK_KP_PLUS,
  SDLK_UNKNOWN,
  SDLK_NUMLOCK,
  SDLK_UNKNOWN,
  SDLK_UNKNOWN,
  SDLK_UNKNOWN,
  SDLK_KP_DIVIDE,
  SDLK_KP_ENTER,
  SDLK_UNKNOWN,
  SDLK_KP_MINUS,
  SDLK_UNKNOWN,
  SDLK_UNKNOWN,
  SDLK_KP_EQUALS,
  SDLK_KP0,
  SDLK_KP1,
  SDLK_KP2,
  SDLK_KP3,
  SDLK_KP4,
  SDLK_KP5,
  SDLK_KP6,
  SDLK_KP7,
  SDLK_UNKNOWN,
  SDLK_KP8,
  SDLK_KP9,
  SDLK_UNKNOWN,
  SDLK_UNKNOWN,
  SDLK_UNKNOWN,
  SDLK_F5,
  SDLK_F6,
  SDLK_F7,
  SDLK_F3,
  SDLK_F8,
  SDLK_F9,
  SDLK_UNKNOWN,
  SDLK_F11,
  SDLK_UNKNOWN,
  SDLK_F13,
  SDLK_PAUSE,
  SDLK_PRINT,
  SDLK_UNKNOWN,
  SDLK_F10,
  SDLK_UNKNOWN,
  SDLK_F12,
  SDLK_UNKNOWN,
  SDLK_SCROLLOCK,
  SDLK_INSERT,
  SDLK_HOME,
  SDLK_PAGEUP,
  SDLK_DELETE,
  SDLK_F4,
  SDLK_END,
  SDLK_F2,
  SDLK_PAGEDOWN,
  SDLK_F1,
  SDLK_LEFT,
  SDLK_RIGHT,
  SDLK_DOWN,
  SDLK_UP,
  SDLK_UNKNOWN,
  SDLK_RMETA,
  SDLK_RSHIFT,
  SDLK_RALT,
  SDLK_RCTRL,
};

#else // !MACOSX

// XXX: Assuming non-Mac computers are PC, no separate failure branch for
// exotic machines.

static SDLKey scancode_map[] = {
  SDLK_UNKNOWN,
  SDLK_ESCAPE,
  SDLK_1,
  SDLK_2,
  SDLK_3,
  SDLK_4,
  SDLK_5,
  SDLK_6,
  SDLK_7,
  SDLK_8,
  SDLK_9,
  SDLK_0,
  SDLK_MINUS,
  SDLK_EQUALS,
  SDLK_BACKSPACE,
  SDLK_TAB,
  SDLK_q,
  SDLK_w,
  SDLK_e,
  SDLK_r,
  SDLK_t,
  SDLK_y,
  SDLK_u,
  SDLK_i,
  SDLK_o,
  SDLK_p,
  SDLK_LEFTBRACKET,
  SDLK_RIGHTBRACKET,
  SDLK_RETURN,
  SDLK_LCTRL,
  SDLK_a,
  SDLK_s,
  SDLK_d,
  SDLK_f,
  SDLK_g,
  SDLK_h,
  SDLK_j,
  SDLK_k,
  SDLK_l,
  SDLK_SEMICOLON,
  SDLK_QUOTE,
  SDLK_BACKQUOTE,
  SDLK_LSHIFT,
  SDLK_BACKSLASH,
  SDLK_z,
  SDLK_x,
  SDLK_c,
  SDLK_v,
  SDLK_b,
  SDLK_n,
  SDLK_m,
  SDLK_COMMA,
  SDLK_PERIOD,
  SDLK_SLASH,
  SDLK_RSHIFT,
  SDLK_KP_MULTIPLY,
  SDLK_LALT,
  SDLK_SPACE,
  SDLK_CAPSLOCK,
  SDLK_F1,
  SDLK_F2,
  SDLK_F3,
  SDLK_F4,
  SDLK_F5,
  SDLK_F6,
  SDLK_F7,
  SDLK_F8,
  SDLK_F9,
  SDLK_F10,
  SDLK_NUMLOCK,
  SDLK_SCROLLOCK,
  SDLK_KP7,
  SDLK_KP8,
  SDLK_KP9,
  SDLK_KP_MINUS,
  SDLK_KP4,
  SDLK_KP5,
  SDLK_KP6,
  SDLK_KP_PLUS,
  SDLK_KP1,
  SDLK_KP2,
  SDLK_KP3,
  SDLK_KP0,
  SDLK_KP_PERIOD,
  SDLK_UNKNOWN,
  SDLK_UNKNOWN,
  SDLK_LESS,
  SDLK_F11,
  SDLK_F12,
};
#endif

SDLKey keysym_for_scancode(int scancode) {
#if defined __linux__
  scancode -= 8;
#endif
  if (scancode > 0 && scancode < sizeof(scancode_map))
    return scancode_map[scancode];
  return SDLK_UNKNOWN;
}
