#ifndef XEV_MTX_HPP
#define XEV_MTX_HPP

#include <boost/static_assert.hpp>
#include <iostream>
#include "vec.hpp"

namespace xev {

///
/// Column-major fixed-size matrix template class.
///
template<class T, int C, int R> class Mtx {
public:
  typedef Vec<T, R> Column_Vec;

  Mtx() {
    for (int i = 0; i < R * C; i++) {
      data[i] = T();
    }
  }

  Mtx(std::initializer_list<T> args) {
    // Data needs to be transposed from the layout it's in into the internal
    // column-major layout.
    int i = 0;
    for (auto v : args) {
      (*this)[i % C][i / C] = v;
      if (++i == R * C)
        break;
    }
  }

  Column_Vec& operator[](int column) {
    // Vecs are just homogeneous arrays, data-layout-wise, so we can just
    // interpret a piece of the column-vector-layout matrix data as the actual
    // vector.
    return reinterpret_cast<Column_Vec*>(data)[column];
  }

  const Column_Vec& operator[](int column) const {
    return reinterpret_cast<const Column_Vec*>(data)[column];
  }

  void unit() {
    BOOST_STATIC_ASSERT(C == R);
    for (int i = 0; i < R; i++) {
      for (int j = 0; j < C; j++) {
        (*this)[i][j] = (i == j ? T(1) : T(0));
      }
    }
  }

  Vec<T, R>& as_vector() {
    BOOST_STATIC_ASSERT(C == 1);
    return (*this)[0];
  }

  const Vec<T, R>& as_vector() const {
    BOOST_STATIC_ASSERT(C == 1);
    return (*this)[0];
  }

  const T* data_ptr() const { return data; }
private:
  T data[R * C];
};

template<class T, int R1, int C1R2, int C2>
Mtx<T, C2, R1> operator*(
    const Mtx<T, C1R2, R1>& lhs,
    const Mtx<T, C2, C1R2>& rhs) {
  Mtx<T, C2, R1> result;
  for (int r = 0; r < R1; r++) {
    for (int c = 0; c < C2; c++) {
      for (int i = 0; i < C1R2; i++)
        result[c][r] += lhs[i][r] * rhs[c][i];
    }
  }
  return result;
}

template<class T, int C, int R>
Vec<T, C> operator*(const Mtx<T, C, R>& lhs, const Vec<T, C>& rhs) {
  auto result = lhs * rhs.as_matrix();
  return result.as_vector();
}

template<class T, int C, int R>
std::ostream& operator<<(std::ostream& out, const Mtx<T, C, R>& mtx) {
  for (int i = 0; i < R; i++) {
    out << "|";
    for (int j = 0; j < C; j++)
      out << " " << mtx[j][i];
    out << " |\n";
  }
  return out;
}

}

#endif
