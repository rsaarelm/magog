// Copyright (C) 2012 Risto Saarelma

#include "surface.hpp"
#include <cstdlib>

extern "C" {
  extern uint8_t* stbi_load_from_memory(
      uint8_t const *buffer, int len, int *x, int *y, int *comp, int req_comp);
}

Surface::Surface()
    : data(nullptr)
      // , texture_handle(0)
    , width(0)
    , height(0) {}

// XXX: Use delegating constructors when gcc supports them.
Surface::Surface(const Static_File* file)
    : data(nullptr)
      // , texture_handle(0)
    , width(0)
    , height(0) {
  load_image(file);
}

Surface::Surface(int width, int height)
    : data(nullptr)
      // , texture_handle(0)
    , width(0)
    , height(0) {
  init_image(width, height);
}

Surface::Surface(const Vec2i& dim)
    : data(nullptr)
      // , texture_handle(0)
    , width(0)
    , height(0) {
  init_image(dim);
}

Surface::~Surface() {
  // Need to use free since data may come from C code which malloc's it.
  free(data);
}

void Surface::load_image(const uint8_t* buffer, size_t buffer_len) {
  free(data);
  data = stbi_load_from_memory(buffer, buffer_len, &width, &height, nullptr, 4);
}

void Surface::load_image(const Static_File* file) { load_image(file->get_data(), file->get_len()); }

void Surface::init_image(int width, int height) {
  free(data);
  width = width;
  height = height;
  data = static_cast<uint8_t*>(malloc(width * height * 4));
}

#if 0
GLuint Surface::make_texture() {
  GLuint result;
  glGenTextures(1, &result);
  glBindTexture(GL_TEXTURE_2D, result);
  // TODO: Support other types of filtering.
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
  glTexImage2D(
      GL_TEXTURE_2D, 0, GL_RGBA8, width, height,
      0, GL_RGBA, GL_UNSIGNED_BYTE, data);
  return result;
}
#endif
