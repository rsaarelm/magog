/* load_fonts.cpp

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

#include <cstdint>
#include <stdio.h>
#include <vector>
#define STB_TRUETYPE_IMPLEMENTATION
#include <contrib/stb/stb_truetype.h>

using namespace std;

vector<uint8_t> read_binary_file(const char* filename) {
  vector<uint8_t> result;
  FILE* file = fopen(filename, "rb");
  if (!file) {
    fprintf(stderr, "Couldn't open file '%s'\n", filename);
    exit(1);
  }
  while (true) {
    int c = fgetc(file);
    if (c < 0)
      break;
    result.push_back(static_cast<uint8_t>(c));
  }
  fclose(file);
  return result;
}

struct Font_Data {
  int first_char;
  int width;
  int height;
  vector<uint8_t> pixels;
  vector<stbtt_bakedchar> chardata;
};

Font_Data load_fonts(
  int height, int first_char, int num_chars, vector<uint8_t> ttf) {
  Font_Data result;

  result.first_char = first_char;
  result.chardata.resize(num_chars);

  int pixel_estimate = height * height * num_chars;

  // Get the smallest power-of-two square texture size that fits the
  // guesstimated number of pixels.
  int dim = 1;
  while (dim * dim < pixel_estimate)
    dim <<= 1;
  //fprintf(stderr, "Guesstimating you want a %d x %d texture\n", dim, dim);

  result.width = dim;
  result.height = dim;
  result.pixels.resize(result.width * result.height);

  int error = stbtt_BakeFontBitmap(
    ttf.data(), 0, height, result.pixels.data(), result.width, result.height,
    first_char, num_chars, result.chardata.data());

  if (error <= 0) {
    fprintf(stderr, "WARNING: load_fonts couldn't fit all characters\n");
    exit(1);
  }

  return result;
}

void usage(int argc, char* argv[]) {
  fprintf(stderr, "Usage: %s pixel_height first_char num_chars input_file output_file\n", argv[0]);
  exit(1);
}

Font_Data load_fonts(int argc, char* argv[]) {
  if (argc != 6)
    usage(argc, argv);

  int height = atoi(argv[1]);
  int first_char = atoi(argv[2]);
  int num_chars = atoi(argv[3]);

  if (height <= 0 || first_char < 0 || num_chars < 1)
    usage(argc, argv);

  return load_fonts(height, first_char, num_chars, read_binary_file(argv[4]));
}
