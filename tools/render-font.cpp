/* render-font.cpp

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

#include "load_fonts.cpp"

int main(int argc, char* argv[]) {
  Font_Data data = load_fonts(argc, argv);

  // Use ImageMagick to create a png.
  char buffer[4096];
  snprintf(
    buffer, sizeof(buffer), "convert -depth 8 -size %dx%d gray: png:\"%s\"",
    data.width, data.height, argv[5]);

  FILE* output = popen(buffer, "w");
  if (!output) {
    fprintf(stderr, "Unable to open pipe\n");
    return 1;
  }

  for (auto byte : data.pixels)
    fputc(byte, output);
  fclose(output);
  return 0;
}
