#include <stdio.h>

void print_base(const char* name) {
  int start = 0;
  int i;
  for (i = 0; name[i]; i++) {
    if (name[i] == '\\' || name[i] == '/')
      start = i + 1;
  }
  name += start;
  while (*name && *name != '.')
    printf("%c", *name++);
}

#define LINE_CHARS 16

void print_data(const unsigned char* data, int len) {
  int i = 0;
  int j;
  while (len > 0) {
    printf("\"");
    for(j = 0; j < LINE_CHARS && len > 0; j++, len--) {
      printf("\\x%02x", *data++);
    }
    printf("\"\n");
  }
}

