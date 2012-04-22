/* atlas.hpp

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
#ifndef UTIL_ATLAS_HPP
#define UTIL_ATLAS_HPP

#include <util/core.hpp>
#include <util/gl_texture.hpp>
#include <util/box.hpp>
#include <util/file_system.hpp>
#include <map>
#include <string>

class Atlas {
public:
  Atlas();
  Atlas(File_System& file, const char* root_path) { init(file, root_path); }

  void init(File_System& file, const char* root_path);
  Vec2i get_dim() const { return atlas_texture.get_dim(); }
  int frameset_start(const char* name) const { return framesets.at(name); }
  Recti frame_rect(int idx) const { return frames[idx]; }
  void bind() const { atlas_texture.bind(); }
  GLuint texture_id() { return atlas_texture.get(); }
  Vec2i offset(int idx) const { return offsets[idx]; }
private:
  Gl_Texture atlas_texture;
  std::map<std::string, int> framesets;
  std::vector<Recti> frames;
  std::vector<Vec2i> offsets;
};

#endif
