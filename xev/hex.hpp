#ifndef XEV_HEX_HPP
#define XEV_HEX_HPP

#include "vec.hpp"
#include "alg.hpp"
#include <array>

namespace xev {
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

  0  .     8  .     16  .     24  .     32  .     40  .     48  .     56  .
   .   .    .   .     .   .     .   .     #   .     #   .     #   .     #   .
     *        *         *         *         *         *         *         *
   .   .    .   .     #   .     #   .     .   .     .   .     #   .     #   .
     .        #         .         #         .         #         .         #

  1  #     9  #     17  #     25  #     33  #     41  #     49  #     57  #
   .   .    .   .     .   .     .   .     #   .     #   .     #   .     #   .
     *        *         *         *         *         *         *         *
   .   .    .   .     #   .     #   .     .   .     .   .     #   .     #   .
     .        #         .         #         .         #         .         #

  2  .    10  .     18  .     26  .     34  .     42  .     50  .     58  .
   .   #    .   #     .   #     .   #     #   #     #   #     #   #     #   #
     *        *         *         *         *         *         *         *
   .   .    .   .     #   .     #   .     .   .     .   .     #   .     #   .
     .        #         .         #         .         #         .         #

  3  #    11  #     19  #     27  #     35  #     43  #     51  #     59  #
   .   #    .   #     .   #     .   #     #   #     #   #     #   #     #   #
     *        *         *         *         *         *         *         *
   .   .    .   .     #   .     #   .     .   .     .   .     #   .     #   .
     .        #         .         #         .         #         .         #

  4  .    12  .     20  .     28  .     36  .     44  .     52  .     60  .
   .   .    .   .     .   .     .   .     #   .     #   .     #   .     #   .
     *        *         *         *         *         *         *         *
   .   #    .   #     #   #     #   #     .   #     .   #     #   #     #   #
     .        #         .         #         .         #         .         #

  5  #    13  #     21  #     29  #     37  #     45  #     53  #     61  #
   .   .    .   .     .   .     .   .     #   .     #   .     #   .     #   .
     *        *         *         *         *         *         *         *
   .   #    .   #     #   #     #   #     .   #     .   #     #   #     #   #
     .        #         .         #         .         #         .         #

  6  .    14  .     22  .     30  .     38  .     46  .     54  .     62  .
   .   #    .   #     .   #     .   #     #   #     #   #     #   #     #   #
     *        *         *         *         *         *         *         *
   .   #    .   #     #   #     #   #     .   #     .   #     #   #     #   #
     .        #         .         #         .         #         .         #

  7  #    15  #     23  #     31  #     39  #     47  #     55  #     63  #
   .   #    .   #     .   #     .   #     #   #     #   #     #   #     #   #
     *        *         *         *         *         *         *         *
   .   #    .   #     #   #     #   #     .   #     .   #     #   #     #   #
     .        #         .         #         .         #         .         #

*/
int hex_wall(int edge_mask);

Vec2i hex_circle_vec(int radius, int index);

Range<Vec2i>::T hex_circle_points(int radius);

Range<Vec2i>::T hex_area_points(int radius);

int hex_dist(const Vec2i& vec);

}

#endif
