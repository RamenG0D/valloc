#include <stdio.h>

#include "../valloc.h"

int main(void) {
    // init global allocator
    valloc_init(1024);

    typedef struct Inner {
        uint8_t data[10];
        float value;
    } Inner;

    // allocate 1 byte of memory (for a ptr)
    Pointer ptr = valloc(sizeof(Inner));

    // write data to the memory block
    Inner* inner = (Inner*)ptr.address;
    for (size_t i = 0; i < 10; i++) {
        inner->data[i] = i;
    }
    inner->value = 3.14;

    // read data from the memory block
    for(size_t i = 0; i < 10; i++) {
        printf("%d ", inner->data[i]);
    }
    printf("\n");
    printf("%f\n", inner->value);
    
    // free memory
    vfree(&ptr);

    return 0;
}
