/* static_file.cpp

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

#include "static_file.hpp"

extern "C" {
extern char *stbi_zlib_decode_malloc(char const *buffer, int len, int *outlen);
}

Static_File* Static_File::s_root = nullptr;

Static_File::Static_File(std::string name, int len, bool compressed, const unsigned char* data)
    : name(name)
    , len(len)
    , data(data)
    , own_data(false)
    , next(Static_File::s_root)
{
  Static_File::s_root = this;

  if (compressed)
    decompress();
}

Static_File::~Static_File() {
  if (own_data)
    delete data;
}

Static_File* Static_File::find(const std::string name) {
  for (Static_File* ptr = Static_File::s_root; ptr; ptr = ptr->next) {
    if (ptr->get_name() == name)
      return ptr;
  }
  return nullptr;
}

void Static_File::decompress() {
  char* new_data;
  int new_len;
  new_data = stbi_zlib_decode_malloc(
      reinterpret_cast<const char*>(data), len, &new_len);
  own_data = true;
  data = reinterpret_cast<const unsigned char*>(new_data);
  len = new_len;
}
