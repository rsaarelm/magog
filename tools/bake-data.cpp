/* bake-data.cpp

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
