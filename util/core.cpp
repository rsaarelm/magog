/* core.cpp

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

#include "core.hpp"
#include <cstdio>
#include <cstdarg>
#include <stdlib.h>
#ifdef __WIN32__
#include <windows.h>
#endif

#ifdef __WIN32__
// TODO: Windows version
void print_trace(void) {}
#else
#include <execinfo.h>

void print_trace(void) {
  void *array[20];
  size_t size;
  char **strings;
  size_t i;

  size = backtrace(array, sizeof(array));
  strings = backtrace_symbols (array, size);

  printf ("Obtained %zd stack frames.\n", size);

  for (i = 0; i < size; i++)
    printf ("%s\n", strings[i]);

  free (strings);
}
#endif

void die(const char* str) {
  #ifndef NDEBUG
  print_trace();
  #endif

  #ifdef __WIN32__
  MessageBox(NULL, str, "Error", MB_OK);
  #else
  fprintf(stderr, "%s\n", str);
  #endif
  exit(1);
}

size_t hash(const char* s) {
  size_t next = *s ? hash(s + 1) : 2166136261u;
  return (next ^ *s) * 16777619u;
}
