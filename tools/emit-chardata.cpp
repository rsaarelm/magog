// Copyright (C) 2012 Risto Saarelma

#include "load_fonts.cpp"

int main(int argc, char* argv[]) {
  Font_Data data = load_fonts(argc, argv);

  FILE* output = fopen(argv[5], "w");

  // Emit a code fragment matching an array of
  // struct {
  //   int character_index;
  //   int x0, y0; // Character rectangle on texture.
  //   int x1, y1;
  //   float xoff, yoff;  // Rendering offset for the character.
  //   float xadvance;    // Width of the character in text.
  // };
  // data.
  for (size_t i = 0; i < data.chardata.size(); i++) {
    auto character = data.chardata[i];
    fprintf(output, "{%d, %d, %d, %d, %d, %g, %g, %g},\n",
           data.first_char + i,
           character.x0, character.y0, character.x1, character.y1,
           character.xoff, character.yoff, character.xadvance);
  }
  fclose(output);
  return 0;
}
