#pragma once


#include <stdint.h>

#ifdef MYLIBRARY_EXPORTS
#define MYLIBRARY_API __declspec(dllexport)
#else
#define MYLIBRARY_API __declspec(dllimport)
#endif


MYLIBRARY_API void hello_world();
MYLIBRARY_API intptr_t add(intptr_t a, intptr_t b);
MYLIBRARY_API intptr_t multiply(intptr_t a, intptr_t b);