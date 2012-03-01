#include <cstdio>
#include <cstdarg>
#include <xev/util.hpp>
#ifdef __WIN32__
#include <windows.h>
#endif

namespace xev {

void die(const char* format, ...) {
  va_list ap;
  va_start(ap, format);
  #ifdef __WIN32__
  char buffer[4096];
  vsnprintf(buffer, sizeof(buffer), format, ap);
  MessageBox(NULL, buffer, "Error", MB_OK);
  #else
  vfprintf(stderr, format, ap);
  fprintf(stderr, "\n");
  #endif
  va_end(ap);
  exit(1);
}

size_t hash(const char* s) {
  if (*s)
    return (hash(s + 1) ^ *s) * 16777619u;
  else
    return 2166136261u;
}

}
