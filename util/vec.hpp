// Copyright (C) 2012 Risto Saarelma

#ifndef UTIL_VEC_HPP
#define UTIL_VEC_HPP

/// \file vec.hpp \brief Geometric vectors

#include <cmath>
#include <algorithm>
#include <initializer_list>
#include <iostream>
#include <boost/static_assert.hpp>

template<class T, int C, int R> class Mtx;

/// Geometric vector class.
template<class T, int N> class Vec {
 public:
  Vec() {
    for (int i = 0; i < N; i++) {
      data[i] = T();
    }
  }

  Vec(std::initializer_list<T> args) {
    int i = 0;
    for (auto v : args) {
      data[i] = v;
      ++i;
      if (i == N)
        break;
    }
  }

  template<class U, int M>
  Vec(const Vec<U, M>& rhs) {
    for (int i = 0; i < N; i++) {
      if (i < M)
        data[i] = static_cast<T>(rhs[i]);
      else
        data[i] = T();
    }
  }

  Vec(const T& e0, const T& e1) {
    BOOST_STATIC_ASSERT(N == 2);
    data[0] = e0;
    data[1] = e1;
  }

  Vec(const T& e0, const T& e1, const T& e2) {
    BOOST_STATIC_ASSERT(N == 3);
    data[0] = e0;
    data[1] = e1;
    data[2] = e2;
  }

  Vec(const T& e0, const T& e1, const T& e2, const T& e3) {
    BOOST_STATIC_ASSERT(N == 4);
    data[0] = e0;
    data[1] = e1;
    data[2] = e2;
    data[3] = e3;
  }

  Vec(const Mtx<T, 1, N>& mtx) {
    (*this) = *reinterpret_cast<Vec<T, N>>(&mtx);
  }


  T& operator[](int i) {
    return data[i];
  }

  T operator[](int i) const {
    return data[i];
  }


  bool operator==(const Vec<T, N>& rhs) const {
    for (int i = 0; i < N; i++) {
      if (data[i] != rhs[i])
        return false;
    }
    return true;
  }

  bool operator!=(const Vec<T, N>& rhs) const { return !(*this == rhs); }

  /// Ordering relation predicate.
  bool operator<(const Vec<T, N>& rhs) const {
    for (int i = 0; i < N; i++) {
      if (data[i] < rhs[i])
        return true;
      else if (data[i] > rhs[i])
        return false;
    }
    return false;
  }


  T* begin() {
    return &data[0];
  }

  T* end() {
    return &data[N];
  }

  const T* begin() const {
    return &data[0];
  }

  const T* end() const {
    return &data[N];
  }


  Vec<T, N>& operator+=(const Vec<T, N>& rhs) {
    for (int i = 0; i < N; i++)
      data[i] += rhs[i];
    return *this;
  }

  Vec<T, N>& operator-=(const Vec<T, N>& rhs) {
    for (int i = 0; i < N; i++)
      data[i] -= rhs[i];
    return *this;
  }

  Vec<T, N>& operator*=(const Vec<T, N>& rhs) {
    *this = *this * rhs;
    return *this;
  }

  Vec<T, N>& operator*=(T rhs) {
    for (int i = 0; i < N; i++)
      data[i] *= rhs;
    return *this;
  }

  Vec<T, N>& operator/=(T rhs) {
    for (int i = 0; i < N; i++)
      data[i] /= rhs;
    return *this;
  }

  Vec<T, N>& in_elem_mul(const Vec<T, N>& rhs) {
    for (int i = 0; i < N; i++)
      data[i] *= rhs[i];
    return *this;
  }

  Vec<T, N>& in_elem_div(const Vec<T, N>& rhs) {
    for (int i = 0; i < N; i++)
      data[i] /= rhs[i];
    return *this;
  }

  Vec<T, N> elem_mul(const Vec<T, N>& rhs) const {
    Vec<T, N> result(*this);
    result.in_elem_mul(rhs);
    return result;
  }

  Vec<T, N> elem_div(const Vec<T, N>& rhs) const {
    Vec<T, N> result(*this);
    result.in_elem_div(rhs);
    return result;
  }


  T abs() const {
    T sum(0);
    for (auto i : *this)
      sum += i * i;
    return sqrt(sum);
  }

  void normalize() {
    (*this) /= abs();
  }


  Mtx<T, 1, N>& as_matrix() {
    return *reinterpret_cast<Mtx<T, 1, N>*>(this);
  }

  const Mtx<T, 1, N>& as_matrix() const {
    return *reinterpret_cast<const Mtx<T, 1, N>*>(this);
  }

  Mtx<T, N, 1>& transpose() {
    return *reinterpret_cast<Mtx<T, N, 1>*>(this);
  }

  const Mtx<T, N, 1>& transpose() const {
    return *reinterpret_cast<const Mtx<T, N, 1>*>(this);
  }

  Mtx<T, N, N> as_diagonal() const {
    Mtx<T, N, N> result;
    for (int i = 0; i < N; i++)
      result[i][i] = (*this)[i];
    return result;
  }
private:
  T data[N];
};

typedef Vec<int, 2>    Vec2i;
typedef Vec<float, 2>  Vec2f;
typedef Vec<double, 2> Vec2d;
typedef Vec<int, 3>    Vec3i;
typedef Vec<float, 3>  Vec3f;
typedef Vec<double, 3> Vec3d;
typedef Vec<int, 4>    Vec4i;
typedef Vec<float, 4>  Vec4f;
typedef Vec<double, 4> Vec4d;

template<class T, int N>
Vec<T, N> operator+(const Vec<T, N>& lhs, const Vec<T, N>& rhs) {
  Vec<T, N> result(lhs);
  result += rhs;
  return result;
}

template<class T, int N>
Vec<T, N> operator-(const Vec<T, N>& lhs, const Vec<T, N>& rhs) {
  Vec<T, N> result(lhs);
  result -= rhs;
  return result;
}

template<class T, int N>
Vec<T, N> operator/(const Vec<T, N>& lhs, T rhs) {
  Vec<T, N> result(lhs);
  result /= rhs;
  return result;
}

template<class T, int N>
Vec<T, N> operator*(T lhs, const Vec<T, N>& rhs) {
  Vec<T, N> result = rhs;
  result *= lhs;
  return result;
}

template<class T, int N>
Vec<T, N> operator*(const Vec<T, N>& lhs, T rhs) {
  Vec<T, N> result = lhs;
  result *= rhs;
  return result;
}

/// Complex product
template<class T>
Vec<T, 2> operator*(const Vec<T, 2>& lhs, const Vec<T, 2>& rhs) {
  return Vec<T, 2>{
    lhs[0]*rhs[0] - lhs[1]*rhs[1],
    lhs[1]*rhs[0] + lhs[0]*rhs[1]
  };
}

/// Cross product
template<class T>
Vec<T, 3> operator*(const Vec<T, 3>& lhs, const Vec<T, 3>& rhs) {
  return Vec<T, 3>{
    lhs[1]*rhs[2] - lhs[2]*rhs[1],
    lhs[2]*rhs[0] - lhs[0]*rhs[2],
    lhs[0]*rhs[1] - lhs[1]*rhs[0]
  };
}

/// Quaternion product
template<class T>
Vec<T, 4> operator*(const Vec<T, 4>& lhs, const Vec<T, 4>& rhs) {
  // Quaternion product
  return Vec<T, 4>{
    lhs[0]*rhs[0] - lhs[1]*rhs[1] - lhs[2]*rhs[2] - lhs[3]*rhs[3],
    lhs[0]*rhs[1] + lhs[1]*rhs[0] + lhs[2]*rhs[3] - lhs[3]*rhs[2],
    lhs[0]*rhs[2] - lhs[1]*rhs[3] + lhs[2]*rhs[0] + lhs[3]*rhs[1],
    lhs[0]*rhs[3] + lhs[1]*rhs[2] - lhs[2]*rhs[1] + lhs[3]*rhs[0]
  };
}

// Completionists may add octonion product here.

/// Convert an axis-angle orientation into the corresponding quaternion.
template<class T>
Vec<T, 4> quat(const Vec<T, 3>& axis, T angle) {
  T s = sin(angle/2);
  return Vec<T, 4>{T(cos(angle/2)), axis[0]*s, axis[1]*s, axis[2]*s};
}

/// Complex conjugate.
template<class T>
Vec<T, 2> conjugated(const Vec<T, 2>& complex) {
  return Vec<T, 2>{complex[0], -complex[1]};
}

/// Quaternion conjugate.
template<class T>
Vec<T, 4> conjugated(const Vec<T, 4>& quaternion) {
  return Vec<T, 4>{quaternion[0], -quaternion[1], -quaternion[2], -quaternion[3]};
}

template<class T, int N>
std::ostream& operator<<(std::ostream& out, const Vec<T, N>& vec) {
  out << "<" << vec[0];
  for (int i = 1; i < N; i++)
    out << ", " << vec[i];
  out << ">";
  return out;
}

#endif
