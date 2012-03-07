// Copyright (C) 2012 Risto Saarelma

#ifndef UTIL_AXISBOX_HPP
#define UTIL_AXISBOX_HPP

#include "alg.hpp"
#include "vec.hpp"
#include "core.hpp"
#include <algorithm>

/// Axis-aligned variable-dimension box.
template<class T, int N> class Axis_Box {
 public:
  Axis_Box() {}

  Axis_Box(const Vec<T, N>& min, const Vec<T, N>& dim)
      : min_pt(min), dim_vec(dim) {
    ASSERT(all_of(dim_vec, [](T x) { return x >= 0; }));
  }

  bool contains(const Vec<T, N>& pos) const {
    return pairwise_all_of(min_pt, pos, [](T a, T b) { return a <= b; }) &&
        pairwise_all_of(pos, max(), [](T a, T b) { return a < b; });
  }

  const Vec<T, N>& min() const { return min_pt; }

  Vec<T, N> max() const { return min_pt + dim_vec; }

  const Vec<T, N>& dim() const { return dim_vec; }

  T volume() const {
    return std::accumulate(
        dim_vec.begin(), dim_vec.end(), T(1),
        [] (const T& a, const T& b) { return a * b; });
  }

 private:
  Vec<T, N> min_pt;
  Vec<T, N> dim_vec;
};

typedef Axis_Box<int, 2> ARecti;
typedef Axis_Box<float, 2> ARectf;
typedef Axis_Box<double, 2> ARectd;
typedef Axis_Box<int, 3> ACubei;
typedef Axis_Box<float, 3> ACubef;
typedef Axis_Box<double, 3> ACubed;

#endif
