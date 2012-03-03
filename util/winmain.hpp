#ifndef UTIL_WINMAIN_HPP
#define UTIL_WINMAIN_HPP

// Wrap main in a WinMain when building Windows release binaries.

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
