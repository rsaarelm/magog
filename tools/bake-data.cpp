// Copyright (C) 2012 Risto Saarelma

#include <stdio.h>

const int bytes_per_line = 16;

int main(int argc, char* argv[]) {
  FILE* input = stdin;
  FILE* output = stdout;

  if (argc > 1)
    input = fopen(argv[1], "rb");
  if (input == 0) {
    fprintf(stderr, "Unable to open input file '%s'\n", argv[1]);
    return 1;
  }

  if (argc > 2)
    output = fopen(argv[2], "w");

  int column = 0;
  int c;
  while ((c = fgetc(input)) >= 0) {
    fprintf(output, "%d,", c);
    if (column++ == bytes_per_line) {
      column = 0;
      fprintf(output, "\n");
    }
  }
  if (column)
    fprintf(output, "\n");
  fclose(input);
  fclose(output);
  return 0;
}
