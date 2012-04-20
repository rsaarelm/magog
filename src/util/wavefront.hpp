/* wavefront.hpp

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

#ifndef UTIL_WAVEFRONT_HPP
#define UTIL_WAVEFRONT_HPP

/** \file wavefront.hpp
 * Wavefront OBJ data parsing.
 */

#include <vector>
#include <tuple>
#include <string>
#include <iostream>
#include "vec.hpp"

struct Wavefront_Face_Point {
  int vertex_idx;
  int texcoord_idx;
  int normal_idx;

  bool operator==(const Wavefront_Face_Point& rhs) const {
    return vertex_idx == rhs.vertex_idx &&
        texcoord_idx == rhs.texcoord_idx &&
        normal_idx == rhs.normal_idx;
  }

  bool operator<(const Wavefront_Face_Point& rhs) const {
    if (vertex_idx < rhs.vertex_idx) {
      return true;
    } else if (vertex_idx == rhs.vertex_idx) {
      if (texcoord_idx < rhs.texcoord_idx) {
        return true;
      } else if (texcoord_idx == rhs.texcoord_idx) {
        if (normal_idx < rhs.normal_idx) {
          return true;
        }
      }
    }
    return false;
  }
};

/// Parse Wavefront OBJ data into a 3D model data structure.
class Parsed_Wavefront_Obj {
 public:
  Parsed_Wavefront_Obj(std::istream& is);

  const std::string name() const { return name_; }

  std::vector<Vec<float, 3>> vertices() const { return vertices_; }
  std::vector<Vec<float, 2>> tex_coords() const { return tex_coords_; }
  std::vector<Vec<float, 3>> normals() const { return normals_; }
  std::vector<std::vector<Wavefront_Face_Point>> faces() const { return faces_; }

 private:
  typedef std::vector<std::string> Tokens;

  void parse(std::istream& is);
  void parse_name(Tokens tokens);
  void parse_vertex(Tokens tokens);
  void parse_normal(Tokens tokens);
  void parse_tex_coord(Tokens tokens);
  void parse_face(Tokens tokens);

  std::string name_;
  std::vector<Vec<float, 3>> vertices_;
  std::vector<Vec<float, 2>> tex_coords_;
  std::vector<Vec<float, 3>> normals_;
  std::vector<std::vector<Wavefront_Face_Point>> faces_;
};

std::ostream& operator<<(std::ostream& out, Parsed_Wavefront_Obj& obj);

struct Unified_Model {
  std::vector<Vec<float, 3>> vertices;
  std::vector<Vec<float, 2>> tex_coords;
  std::vector<Vec<float, 3>> normals;
  std::vector<short> faces;
};

/// Unify a parsed Wavefront OBJ model's vertices, texture coordinates and normals into the same indexing.
Unified_Model unify_model(const Parsed_Wavefront_Obj& obj);

#endif
