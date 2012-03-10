/* core.hpp

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

#ifndef UTIL_CORE_HPP
#define UTIL_CORE_HPP

/// \file core.hpp \brief Low-level helper utilities.

#include <cstddef>

template<size_t N, size_t I=0>
struct _hash_calc {
  static constexpr size_t apply(const char (&s)[N]) {
    return (_hash_calc<N, I+1>::apply(s) ^ s[I]) * 16777619u;
  }
};

template<size_t N>
struct _hash_calc<N, N> {
  static constexpr size_t apply(const char (&s)[N]) {
    return 2166136261u;
  }
};

/// Hash a string at compile-time.
template<size_t N>
constexpr size_t const_hash(const char (&s)[N]) {
  return _hash_calc<N>::apply(s);
}

/// Hash a string.
size_t hash(const char* s);

/// Terminate program with an error message.
void die(const char* format, ...);

#ifdef NDEBUG
#define ASSERT(expr) ((void)0)
#else
#define ASSERT(expr) ((expr) ? ((void)0) : die("Assertion %s failed at %s: %d", #expr, __FILE__, __LINE__))
#endif

#endif
