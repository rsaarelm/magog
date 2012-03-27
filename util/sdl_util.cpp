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
