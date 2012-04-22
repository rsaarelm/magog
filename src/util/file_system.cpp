/* file_system.cpp

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

#include "file_system.hpp"
#include <util/core.hpp>

#include <physfs.h>

int File_System::file_system_counter = 0;

File_System::File_System(const char* rootfile) {
  ASSERT(file_system_counter == 0); // Mustn't have multiple filesystem instances
  file_system_counter++;
  PHYSFS_init(nullptr);
  PHYSFS_addToSearchPath(rootfile, 1);

}

File_System::~File_System() {
  PHYSFS_deinit();
  file_system_counter--;
}

bool File_System::exists(const char* filename) const {
  return PHYSFS_exists(filename);
}

std::vector<uint8_t> File_System::read(const char* filename) const {
  PHYSFS_File* file = PHYSFS_openRead(filename);
  if (file == nullptr)
    throw File_System_Exception();
  std::vector<uint8_t> result;
  size_t len = PHYSFS_fileLength(file);
  result.resize(len);
  PHYSFS_read(file, result.data(), 1, len);
  return result;
}
