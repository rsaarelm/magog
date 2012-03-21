/* box.hpp

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

#ifndef UTIL_BOX_HPP
#define UTIL_BOX_HPP

#include <util/alg.hpp>
#include <util/vec.hpp>
#include <util/core.hpp>
#include <algorithm>
#include <vector>

/// Axis-aligned variable-dimension box.
template<class T, int N> class Box {
 public:
  Box() {}

  Box(const Vec<T, N>& min, const Vec<T, N>& dim)
      : min_pt(min), dim_vec(dim) {
    ASSERT(all_of(dim_vec, [](T x) { return x >= 0; }));
  }

  Box(const Vec<T, N>& dim)
      : min_pt(), dim_vec(dim) {
    ASSERT(all_of(dim_vec, [](T x) { return x >= 0; }));
  }

  template<class ForwardIterator>
  static Box<T, N> smallest_containing(ForwardIterator first, ForwardIterator last) {
    Vec<T, N> min = *first;
    Vec<T, N> max = *first;
    while (++first != last) {
      min = elem_min(min, *first);
      max = elem_max(max, *first);
    }
    return Box<T, N>(min, max - min);
  }

  bool contains(const Vec<T, N>& pos) const {
    return pairwise_all_of(min_pt, pos, [](T a, T b) { return a <= b; }) &&
        pairwise_all_of(pos, max(), [](T a, T b) { return a < b; });
  }

  bool contains(const Box<T, N>& other) const {
    for (int i = 0; i < N; i++) {
      if (other.min()[i] < min()[i]) return false;
      if (other.max()[i] > max()[i]) return false;
    }
    return true;
  }

  bool intersects(const Box<T, N>& other) const {
    int no_intersect = N;
    for (int i = 0; i < N; i++) {
      if (!(other.min()[i] >= max()[i] || min()[i] >= other.max()[i]))
        no_intersect--;
    }
    return !no_intersect;
  }

  const Vec<T, N>& min() const { return min_pt; }

  Vec<T, N> max() const { return min_pt + dim_vec; }

  const Vec<T, N>& dim() const { return dim_vec; }

  Box<T, N> operator+(const Vec<T, N>& offset) const {
    return Box<T, N>(min_pt + offset, dim_vec);
  }

  T volume() const {
    return std::accumulate(
        dim_vec.begin(), dim_vec.end(), T(1),
        [] (const T& a, const T& b) { return a * b; });
  }

  int num_vertices() const {
    return 1 << N;
  }

  Vec<T, N> vertex(int idx) const {
    ASSERT(idx >= 0 && idx < num_vertices());
    Vec<T, N> result;
    for (int i = 0; i < N; i++)
      result[i] = (idx & (1 << i) ? min() : max())[i];
    return result;
  }

  std::vector<Vec<T, N>> vertices() const {
    std::vector<Vec<T, N>> result;
    for (int i = 0; i < num_vertices(); i++)
      result.push_back(vertex(i));
    return result;
  }

 private:
  Vec<T, N> min_pt;
  Vec<T, N> dim_vec;
};

typedef Box<int, 2> Recti;
typedef Box<float, 2> Rectf;
typedef Box<double, 2> Rectd;
typedef Box<int, 3> Cubei;
typedef Box<float, 3> Cubef;
typedef Box<double, 3> Cubed;

#endif
