/* file_system.hpp

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
#ifndef UTIL_FILE_SYSTEM_HPP
#define UTIL_FILE_SYSTEM_HPP

#include <vector>
#include <string>
#include <stdexcept>

class File_System_Exception : public std::exception {};

class File_System {
public:
  File_System(const char* rootfile);
  ~File_System();

  bool exists(const char* filename) const;

  std::vector<uint8_t> read(const char* filename) const;

  std::vector<std::string> list_files(const char* dir);
private:
  File_System(const File_System&);
  File_System& operator=(const File_System&);

  // Variable to ensure that multiple fs instances aren't instantiated at the
  // same time.
  static int file_system_counter;
};

#endif
