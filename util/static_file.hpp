/* static_file.hpp

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

#ifndef UTIL_STATIC_FILE_HPP
#define UTIL_STATIC_FILE_HPP

#include <string>

class Static_File {
 public:
  Static_File(std::string name, int len, bool compressed, const unsigned char* data);
  ~Static_File();

  const unsigned char* get_data() const { return data; }
  int get_len() const { return len; }
  std::string get_name() const { return name; }

  static Static_File* find(const std::string name);
 private:
  Static_File(const Static_File&);
  Static_File& operator=(const Static_File&);

  void decompress();

  std::string name;
  int len;
  const unsigned char* data;
  bool own_data;

  static Static_File* s_root;
  Static_File* next;
};

// Produce junk global variables just for the purpose of pushing their values
// into the static root index in Static_File.
#define _UTIL_MERGE(x, y) x##y
#define _UTIL_LABEL(a) _UTIL_MERGE(_util_gen_file_,a)

#define UTIL_COMPRESSED_FILE(name, len, data) \
  static Static_File _UTIL_LABEL(__LINE__)(name, len, true, reinterpret_cast<const unsigned char*>(data));

#define UTIL_FILE(name, len, data) \
  static Static_File _UTIL_LABEL(__LINE__)(name, len, false, reinterpret_cast<const unsigned char*>(data));

#endif
