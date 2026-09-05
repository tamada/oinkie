// A program whose function names contain a double quote, for #77.
//
// A C++ user-defined literal operator mangles to something Ghidra demangles
// as `operator""__km`, so lifting this exercises the escaping that the JSON
// writer in lifter/scripts/HighPCodeLifter.java has to do. Nothing here is
// about what the program computes.
//
// Built with: clang++ -std=c++17 -O0 -g0 -o bin/udl udl.cpp

#include <cstdio>

long double operator""_km(long double v) { return v * 1000.0L; }

unsigned long long operator""_bin(const char* s, unsigned long) { return s[0]; }

int main() {
  std::printf("%Lf %llu\n", 3.0_km, "x"_bin);
  return 0;
}
