// Copyright (C) 2012 Risto Saarelma

#ifndef UTIL_WINMAIN_HPP
#define UTIL_WINMAIN_HPP

/** \file winmain.hpp
 * A WinMain function for non-console Windows executables.
 *
 * To use, just include this file into the .cpp file that contains the
 * application's main function. The `WinMain` function gets generated and
 * calls the `main` function with the appropriate `argc` and `argv` on Windows
 * release builds. This file does nothing on non-Windows platforms and on
 * Windows debug builds, where you are assumed to want a console you can print
 * info to.
 */

#if defined(NDEBUG) && defined(__WIN32__)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
extern int __argc;
extern char** __argv;
int main(int argc, char* argv[]);
int APIENTRY WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow)
{ return main(__argc, __argv); }
#endif

#endif
