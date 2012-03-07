// Copyright (C) 2012 Risto Saarelma

#ifndef UTIL_SHADER_HPP
#define UTIL_SHADER_HPP

/** \file shader.hpp
 * OpenGL shader utilities.
 */

#include <GL/glew.h>

GLuint load_shader(const char* source, GLenum type);

GLuint link_program(GLuint vertex_shader, GLuint fragment_shader);

extern const char* standard_vertex_shader;
extern const char* billboard_vertex_shader;

extern const char* standard_fragment_shader;

#endif
