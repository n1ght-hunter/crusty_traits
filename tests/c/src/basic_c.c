
#define MYLIBRARY_EXPORTS

#include "basic_c.h"
#include "../include/cbindgen.h"
#include <stdio.h>

void hello_world() {
    printf("Hello from the C shared library!\n");
}

intptr_t add(intptr_t a, intptr_t b) {
    return a + b;
}

intptr_t multiply(intptr_t a, intptr_t b) {
    return a * b;
}