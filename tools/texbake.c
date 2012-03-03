#include "bake.h"
#include "../contrib/stb/stb_image.c"
#include <stdio.h>

void usage() {
  printf(
      "Usage: texbake [image file | option]* > baked_data.c\n"
      "  --rgba: Image data will be 4-channel RGBA from now on (default)\n"
      "  --a:    Image data will be 1-channel alpha from now on\n");
  exit(1);
}

int main(int argc, char* argv[]) {
  int w = 0, h = 0;
  int handled = 0;
  int i;
  int channels = 4;
  stbi_uc* data;

  printf("typedef struct { int w; int h; int bpp; const char* data; } _Texture_Data;\n");

  for (i = 1; i < argc; i++, handled++) {
    if (argv[i][0] == '-') {
      if (strcmp(argv[i], "--rgba") == 0)
        channels = 4;
      else if (strcmp(argv[i], "--a") == 0)
        channels = 1;
      else
        usage();
    } else {
      data = stbi_load(argv[i], &w, &h, NULL, channels);
      if (data == NULL) {
        printf("File %s not found.\n", argv[i]);
        exit(1);
      }
      printf("_Texture_Data _texdata_");
      print_base(argv[i]);
      printf(" = { %d, %d, %d,\n", w, h, channels);
      print_data(data, w * h * channels);
      printf("};\n\n");
    }
  }
  if (handled == 0)
    usage();
  return 0;
}
