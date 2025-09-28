#include "main.h"
#include "../include/cbindgen.h"
#include <stdio.h>


int main() {
    printf("Calling functions from the C shared library:\n");
    hello_world();
    return 0;
}

