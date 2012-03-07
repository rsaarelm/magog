// Copyright (C) 2012 Risto Saarelma

#include <cstdint>
#include <stdio.h>
#include <vector>
#define STB_TRUETYPE_IMPLEMENTATION
#include "../contrib/stb/stb_truetype.h"

using namespace std;

vector<uint8_t> read_binary_stdin() {
  vector<uint8_t> result;
  freopen(NULL, "rb", stdin);
  while (true) {
    int c = fgetc(stdin);
    if (c < 0)
      break;
    result.push_back(static_cast<uint8_t>(c));
  }
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

  if (error <= 0)
    fprintf(stderr, "WARNING: load_fonts couldn't fit all characters\n");

  return result;
}

void usage(int argc, char* argv[]) {
  fprintf(stderr, "Usage: %s pixel_height first_char num_chars\n", argv[0]);
  exit(1);
}

Font_Data load_fonts(int argc, char* argv[]) {
  if (argc != 4)
    usage(argc, argv);

  int height = atoi(argv[1]);
  int first_char = atoi(argv[2]);
  int num_chars = atoi(argv[3]);

  if (height <= 0 || first_char < 0 || num_chars < 1)
    usage(argc, argv);

  return load_fonts(height, first_char, num_chars, read_binary_stdin());
}
