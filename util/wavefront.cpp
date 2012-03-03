#include "wavefront.hpp"
#include <boost/tokenizer.hpp>
#include <iostream>
#include <algorithm>
#include <cstdlib>
#include <map>
#include <cassert>

Parsed_Wavefront_Obj::Parsed_Wavefront_Obj(std::istream& is) {
  parse(is);
}

static std::vector<std::string> tokenize_line(const std::string& line) {
  using namespace boost;

  std::vector<std::string> result;
  char_delimiters_separator<char> seps(true, "/", nullptr);
  tokenizer<char_delimiters_separator<char>> tokens(line, seps);

  for (auto token : tokens)
    result.push_back(token);

  return result;
}

void Parsed_Wavefront_Obj::parse(std::istream& is) {
  // XXX: Comments on the same line with actual data will mess up parsing.
  std::string str;
  while (!is.eof()) {
    std::getline(is, str);
    auto tokens = tokenize_line(str);
    if (!tokens.empty()) {
      if (tokens[0] == "o")
        parse_name(tokens);
      else if (tokens[0] == "v")
        parse_vertex(tokens);
      else if (tokens[0] == "vn")
        parse_normal(tokens);
      else if (tokens[0] == "vt")
        parse_tex_coord(tokens);
      else if (tokens[0] == "f")
        parse_face(tokens);
#if 0
      else if (tokens[0] == "mtllib") {} // Material library. Not handled.
      else if (tokens[0] == "usemtl") {} // Material specification. Not handled.
      else if (tokens[0] == "g") {} // Group. Not handled.
      else if (tokens[0] == "s") {} // Smoothing group. Not handled.
#endif
    }
  }
}

void Parsed_Wavefront_Obj::parse_name(Parsed_Wavefront_Obj::Tokens tokens) {
  name_ = tokens[1];
}

void Parsed_Wavefront_Obj::parse_vertex(Parsed_Wavefront_Obj::Tokens tokens) {
  Vec<float, 3> vec(
    atof(tokens[1].c_str()),
    atof(tokens[2].c_str()),
    atof(tokens[3].c_str()));
  vertices().push_back(vec);
}

void Parsed_Wavefront_Obj::parse_normal(Parsed_Wavefront_Obj::Tokens tokens) {
  Vec<float, 3> vec(
    atof(tokens[1].c_str()),
    atof(tokens[2].c_str()),
    atof(tokens[3].c_str()));
  normals().push_back(vec);
}

void Parsed_Wavefront_Obj::parse_tex_coord(Parsed_Wavefront_Obj::Tokens tokens) {
  Vec<float, 2> vec(
    atof(tokens[1].c_str()),
    1.0f - atof(tokens[2].c_str()));
  tex_coords().push_back(vec);
}

void Parsed_Wavefront_Obj::parse_face(Parsed_Wavefront_Obj::Tokens tokens) {
  // Possible formats for the face spec are
  // f v1 v2 v3 v4 ...
  // f v1/vt1 v2/vt2 v3/vt3 ...
  // f v1/vt1/vn1 v2/vt2/vn2 v3/vt3/vn3 ...
  // f v1//vn1 v2//vn2 v3//vn3 ...
  int n_slashes = std::count(tokens.begin(), tokens.end(), "/");
  int n_elts = tokens.size() - n_slashes - 1;
  std::vector<Wavefront_Face_Point> face;
  if (n_slashes == 0) { // vertex indices only
    for (int i = 0; i < n_elts; i++)
      face.push_back(Wavefront_Face_Point{
          atoi(tokens[i+1].c_str()) - 1,
          -1,
          -1});
  } else if (n_slashes * 2 == n_elts) { // vertex and texture indices
    for (int i = 0; i < n_elts / 2; i++)
      face.push_back(Wavefront_Face_Point{
          atoi(tokens[i * 3 + 1].c_str()) - 1,
          atoi(tokens[i * 3 + 3].c_str()) - 1,
          -1});
  } else if (n_slashes * 3 == n_elts * 2) { // vertex, texture and normal indices
    for (int i = 0; i < n_elts / 3; i++)
      face.push_back(Wavefront_Face_Point{
          atoi(tokens[i * 5 + 1].c_str()) - 1,
          atoi(tokens[i * 5 + 3].c_str()) - 1,
          atoi(tokens[i * 5 + 5].c_str()) - 1});
  } else if (n_slashes == n_elts) { // vertex and normal indices
    for (int i = 0; i < n_elts / 2; i++)
      face.push_back(Wavefront_Face_Point{
          atoi(tokens[i * 4 + 1].c_str()) - 1,
          -1,
          atoi(tokens[i * 4 + 4].c_str()) - 1});
  } else { } // Degenerate line.
  faces().push_back(face);
}

std::ostream& operator<<(std::ostream& out, Parsed_Wavefront_Obj& obj) {
  out << "WaveFront object '" << obj.name() << "'";
  return out;
}

Unified_Model unify_model(const Parsed_Wavefront_Obj& obj) {
  std::map<Wavefront_Face_Point, int> unified_indices;
  size_t current_idx = 0;
  Unified_Model result;

  for (auto face : obj.faces()) {
    // Add any new unique combined indices to the lookup and to the result
    // arrays.
    for (auto point : face) {
      if (unified_indices.count(point) == 0) {
        unified_indices[point] = current_idx++;
        result.vertices.push_back(obj.vertices()[point.vertex_idx]);
        result.tex_coords.push_back(obj.tex_coords()[point.texcoord_idx]);
        result.normals.push_back(obj.normals()[point.normal_idx]);

        // Sanity check.
        assert(current_idx == result.vertices.size() &&
               current_idx == result.tex_coords.size() &&
               current_idx == result.normals.size());
      }
    }

    std::vector<short> out_face;
    if (face.size() == 3) {
      out_face.push_back(unified_indices[face[0]]);
      out_face.push_back(unified_indices[face[1]]);
      out_face.push_back(unified_indices[face[2]]);
      result.faces.insert(result.faces.end(), out_face.begin(), out_face.end());
    } else if (face.size() == 4) {
      out_face.push_back(unified_indices[face[0]]);
      out_face.push_back(unified_indices[face[1]]);
      out_face.push_back(unified_indices[face[2]]);
      result.faces.insert(result.faces.end(), out_face.begin(), out_face.end());
      out_face.push_back(unified_indices[face[0]]);
      out_face.push_back(unified_indices[face[2]]);
      out_face.push_back(unified_indices[face[3]]);
      result.faces.insert(result.faces.end(), out_face.begin(), out_face.end());
    } else {
      // TODO: General triangulation case.
      // TODO: Just throw an error instead if can't be bothered to do that.
    }
  }
  return result;
}
