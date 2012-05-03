/* format.hpp

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
#ifndef UTIL_FORMAT_HPP
#define UTIL_FORMAT_HPP

/// \file format.hpp \brief C++-style variadic printf-style string construction.

#include <string>
#include <sstream>

class Format_Exception : public std::exception {};

std::string format(const char* fmt);

void die(const char* str);

template<typename T, typename... Args>
std::string format(const char* fmt, T value, Args... args) {
  std::stringstream result;

  while (*fmt) {
    if (*fmt == '%' && *(++fmt) != '%') {
      if (*fmt++ != 's')
        die("format only supports %s");
      result << value;
      result << format(fmt, args...);
      return result.str();
    } else {
      result << *fmt++;
    }
  }

  die("extra arguments given to format");
  // Won't get here, but have it anyway to keep the compiler happy.
  return result.str();
}

#endif
