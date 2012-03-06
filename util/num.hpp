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
typename Container::iterator rand_choice(const Container& container) {
  auto result = container.begin();
  std::advance(result, rand_int(container.size() - 1));
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

#endif
