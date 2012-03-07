// Copyright (C) 2012 Risto Saarelma

#include "load_fonts.cpp"

int main(int argc, char* argv[]) {
  Font_Data data = load_fonts(argc, argv);

  // Use ImageMagick to create a png.
  char buffer[4096];
  snprintf(buffer, sizeof(buffer), "convert -depth 8 -size %dx%d gray: png:", data.width, data.height);

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
