// Copyright (C) 2012 Risto Saarelma

#include <stdio.h>

const int bytes_per_line = 16;

int main(int argc, char* argv[]) {
  int column = 0;
  int c;
  while ((c = getchar()) >= 0) {
    printf("\\x%02x,", c);
    if (column++ == bytes_per_line) {
      column = 0;
      printf("\n");
    }
  }
  if (column)
    printf("\n");
  return 0;
}
