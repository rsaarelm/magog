#ifndef UTIL_HEX_HPP
#define UTIL_HEX_HPP

#include "vec.hpp"
#include "alg.hpp"
#include <array>

extern const std::array<const Vec2i, 6> hex_dirs;

/*
   Pick the nicest-looking wall piece for a given configuration of six
   neighboring wall tiles. Haven't managed to come up with an algorithmic
   approach that encapsulates all the hard-to-define nice, so this is just a
   big hand-coded lookup table.

   Indexed edge layout and the various wall mask configurations:

      0
    5   1
      o
    4   2
   y  3  x
  /       \

  00  .    01  #    02  .    03  #    04  .    05  #    06  .    07  #
    .   .    .   .    .   #    .   #    .   .    .   .    .   #    .   #
      *        *        *        *        *        *        *        *
    .   .    .   .    .   .    .   .    .   #    .   #    .   #    .   #
      .        .        .        .        .        .        .        .

  08  .    09  #    10  .    11  #    12  .    13  #    14  .    15  #
    .   .    .   .    .   #    .   #    .   .    .   .    .   #    .   #
      *        *        *        *        *        *        *        *
    .   .    .   .    .   .    .   .    .   #    .   #    .   #    .   #
      #        #        #        #        #        #        #        #

  16  .    17  #    18  .    19  #    20  .    21  #    22  .    23  #
    .   .    .   .    .   #    .   #    .   .    .   .    .   #    .   #
      *        *        *        *        *        *        *        *
    #   .    #   .    #   .    #   .    #   #    #   #    #   #    #   #
      .        .        .        .        .        .        .        .

  24  .    25  #    26  .    27  #    28  .    29  #    30  .    31  #
    .   .    .   .    .   #    .   #    .   .    .   .    .   #    .   #
      *        *        *        *        *        *        *        *
    #   .    #   .    #   .    #   .    #   #    #   #    #   #    #   #
      #        #        #        #        #        #        #        #

  32  .    33  #    34  .    35  #    36  .    37  #    38  .    39  #
    #   .    #   .    #   #    #   #    #   .    #   .    #   #    #   #
      *        *        *        *        *        *        *        *
    .   .    .   .    .   .    .   .    .   #    .   #    .   #    .   #
      .        .        .        .        .        .        .        .

  40  .    41  #    42  .    43  #    44  .    45  #    46  .    47  #
    #   .    #   .    #   #    #   #    #   .    #   .    #   #    #   #
      *        *        *        *        *        *        *        *
    .   .    .   .    .   .    .   .    .   #    .   #    .   #    .   #
      #        #        #        #        #        #        #        #

  48  .    49  #    50  .    51  #    52  .    53  #    54  .    55  #
    #   .    #   .    #   #    #   #    #   .    #   .    #   #    #   #
      *        *        *        *        *        *        *        *
    #   .    #   .    #   .    #   .    #   #    #   #    #   #    #   #
      .        .        .        .        .        .        .        .

  56  .    57  #    58  .    59  #    60  .    61  #    62  .    63  #
    #   .    #   .    #   #    #   #    #   .    #   .    #   #    #   #
      *        *        *        *        *        *        *        *
    #   .    #   .    #   .    #   .    #   #    #   #    #   #    #   #
      #        #        #        #        #        #        #        #
*/
int hex_wall(int edge_mask);

Vec2i hex_circle_vec(int radius, int index);

Range<Vec2i>::T hex_circle_points(int radius);

Range<Vec2i>::T hex_area_points(int radius);

int hex_dist(const Vec2i& vec);

#endif
