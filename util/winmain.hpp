/* winmain.hpp

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
