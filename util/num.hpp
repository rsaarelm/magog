/* num.hpp

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

#ifndef UTIL_NUM_HPP
#define UTIL_NUM_HPP

/** \file num.hpp
 * Numerical and random number generating functions.
 */

#include <cstddef>
#include <iterator>

const float pi = 3.1415926535;

/// Modulo function that handles negative numbers like you'd expect.
template<class Num>
Num mod(Num x, Num m) {
  return (x < 0 ? ((x % m) + m) : x % m);
}

/// Signum function, return -1, 0 or 1 for negative, zero or positive arguments.
template<class Num>
Num sign(Num x) {
  if (x < Num(0))
    return Num(-1);
  else if (x > Num(0))
    return Num(1);
  else
    return Num(0);
}

/// Return a random integer from `[0, max)`.
int rand_int(int max);

/// Return a randomly chosen element from a container.
template<class Container>
typename Container::const_iterator rand_choice(const Container& container) {
  auto result = container.begin();
  std::advance(result, rand_int(container.size() - 1));
  return result;
}

template<class Iterator>
Iterator rand_choice(const Iterator begin, const Iterator end) {
  size_t size = std::distance(begin, end);
  Iterator result = begin;
  std::advance(result, rand_int(size - 1));
  return result;
}

/// Return a random float from `[0, 1)`.
float uniform_rand();

/// Return true with probability `1 / n`.
bool one_chance_in(int n);

/// Seed the default random number generator with the given value.
void seed_rand(int seed);

/// Seed the default random number generator with the given string value.
void seed_rand(const char* seed);

/// Add results of four dice which can give -1, 0 or 1 with equal
/// probabilities. Result is distributed in a crude approximation of a normal
/// distribution. The name refers to the pen&paper RPG system that uses these
/// type of rolls.
int fudge_roll();

template<class A, class C>
C lerp(const C& a, const C& b, A x) {
  return a + x * (b - a);
}

double int_noise(int seed);

#endif
