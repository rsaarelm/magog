/* num.cpp

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

#include "num.hpp"
#include "core.hpp"
#include <ctime>
#include <random>

using namespace std;

static mt19937 g_rng(static_cast<unsigned>(std::time(0)));

int rand_int(int max) {
  return uniform_int_distribution<int>(0, max)(g_rng);
}

float uniform_rand() {
  return uniform_real_distribution<>(0.0, 1.0)(g_rng);
}

bool one_chance_in(int n) {
  return rand_int(n) == 0;
}

void seed_rand(int seed) {
  g_rng.seed(seed);
}

void seed_rand(const char* seed) {
  g_rng.seed(::hash(seed));
}

int fudge_roll() {
  int result = 0;
  for (int i = 0; i < 4; i++)
    result += rand_int(3) - 1;
  return result;
}

double int_noise(int seed) {
  seed = (seed >> 13) ^ seed;
  int x = (seed * (seed * seed * 60493 + 19990303) + 1376312589) & 0x7fffffff;
  return 1.0 - (static_cast<double>(x) / 1073741824.0);
}
