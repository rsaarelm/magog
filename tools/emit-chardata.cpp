/* emit-chardata.cpp

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

  FILE* output = fopen(argv[5], "w");

  // Emit a code fragment matching an array of
  // struct {
  //   int x0, y0; // Character rectangle on texture.
  //   int x1, y1;
  //   float xoff, yoff;  // Rendering offset for the character.
  //   float char_width;
  // };
  // data.
  for (auto character : data.chardata)
    fprintf(output, "{%d, %d, %d, %d, %g, %g, %g},\n",
           character.x0, character.y0, character.x1, character.y1,
           character.xoff, character.yoff, character.xadvance);
  fclose(output);
  return 0;
}
