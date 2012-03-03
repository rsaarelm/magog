#include <stdio.h>
#include "bake.h"
#define STB_TRUETYPE_IMPLEMENTATION
#include "../contrib/stb/stb_truetype.h"

void usage() {
  printf("Usage: fontbake ttf_file point_size > baked_data.c\n");
  exit(1);
}

#define DIM 128
#define NUM_CHARS 96

int main(int argc, char* argv[]) {
  if (argc != 3 || !atoi(argv[2]))
    usage();
  char ttf_buffer[1<<20];
  char bitmap[DIM * DIM];
  stbtt_fontinfo font;
  int i;
  stbtt_bakedchar chardata[NUM_CHARS];
  int height = atoi(argv[2]);

  fread(ttf_buffer, 1, sizeof(ttf_buffer), fopen(argv[1], "rb"));
  stbtt_BakeFontBitmap(ttf_buffer, 0, height, bitmap, DIM, DIM, 32, NUM_CHARS, chardata);

  printf("#include <contrib/stb/stb_truetype.h>\n");
  printf("#include <util/font.hpp>\n");
  printf("Font_Data _fontdata_");
  print_base(argv[1]);
  printf(" = {%d, %d, %d, (stbtt_bakedchar[]){\n", height, DIM, DIM);
  for (i = 0; i < NUM_CHARS; i++)
    printf("{%d, %d, %d, %d, %f, %f, %f}%s\n",
           chardata[i].x0, chardata[i].y0, chardata[i].x1, chardata[i].y1,
           chardata[i].xoff, chardata[i].yoff, chardata[i].xadvance,
           i == NUM_CHARS - 1 ? "}," : ",");
  print_data(bitmap, sizeof(bitmap));
  printf("};\n");

  return 0;
}
