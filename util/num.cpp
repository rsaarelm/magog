// Copyright (C) 2012 Risto Saarelma

#include "num.hpp"
#include "core.hpp"
#include <ctime>
#include <random>

using namespace std;

static mt19937 g_rng(static_cast<unsigned>(std::time(0)));

int rand_int(int max) {
  return uniform_int_distribution<int>(0, max)(g_rng);
}

bool one_chance_in(int n) {
  return rand_int(n) == 0;
}

void seed_Rand(int seed) {
  g_rng.seed(seed);
}

void seed_rand(const char* seed) {
  g_rng.seed(::hash(seed));
}
