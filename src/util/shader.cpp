/* shader.cpp

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

#include "shader.hpp"
#include <util/core.hpp>
#include <iostream>
#include <GL/gl.h>

static std::string gl_info(
    GLuint object,
    PFNGLGETSHADERIVPROC glGet__iv,
    PFNGLGETSHADERINFOLOGPROC glGet__InfoLog) {
  GLint log_length;
  std::string result;

  glGet__iv(object, GL_INFO_LOG_LENGTH, &log_length);
  result.resize(log_length);
  glGet__InfoLog(object, log_length, NULL, &result[0]);
  return result;
}

GLuint load_shader(const char* source, GLenum type) {
  GLuint shader = glCreateShader(type);
  if (!shader) {
    throw "Shader creation failed";
  }
  glShaderSource(shader, 1, &source, NULL);
  glCompileShader(shader);
  GLint shader_ok;
  glGetShaderiv(shader, GL_COMPILE_STATUS, &shader_ok);
  if (!shader_ok) {
    auto err = gl_info(shader, glGetShaderiv, glGetShaderInfoLog);
    die("Shader compile failed: %s", err.c_str());
  }
  return shader;
}

GLuint link_program(GLuint vertex_shader, GLuint fragment_shader) {
  GLuint program = glCreateProgram();
  if (!program) {
    throw "Program creation failed";
  }
  glAttachShader(program, vertex_shader);
  glAttachShader(program, fragment_shader);
  glLinkProgram(program);

  GLint program_ok;
  glGetProgramiv(program, GL_LINK_STATUS, &program_ok);
  if (!program_ok) {
    auto err = gl_info(program, glGetProgramiv, glGetProgramInfoLog);
    die("Program linking failed: %s", err.c_str());
  }
  return program;
}

const char* standard_vertex_shader = R"glsl(
#version 110

uniform mat4 p_matrix, mv_matrix;

attribute vec4 a_position;
attribute vec2 a_texcoord;
attribute vec4 a_normal;

varying vec2 v_texcoord;

void main() {
  gl_Position = p_matrix * mv_matrix * a_position;
  v_texcoord = a_texcoord;
}
)glsl";

const char* billboard_vertex_shader = R"glsl(
#version 110

uniform mat4 p_matrix, mv_matrix;

attribute vec2 a_position;

varying vec2 v_texcoord;

void main() {
  gl_Position = p_matrix * mv_matrix * vec4(a_position, 0.0, 1.0);
  v_texcoord = (a_position * vec2(0.5) + vec2(0.5)) * vec2(1.0, -1.0);
}
)glsl";

const char* standard_fragment_shader = R"glsl(
#version 110

varying vec2 v_texcoord;
uniform sampler2D s_texture;

void main() {
  gl_FragColor = texture2D(s_texture, v_texcoord);
}
)glsl";

