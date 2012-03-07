// Copyright (C) 2012 Risto Saarelma

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
