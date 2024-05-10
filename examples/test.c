#include <stdio.h>
#include "valloc.h"

int main(void) {
    // init global allocator
    global_init(1024);
    
    typedef struct Inner {
        uint8_t data[10];
        float value;
    } Inner;

    // allocate 1 byte of memory (for a ptr)
    Inner* ptr = (Inner*)valloc(sizeof(Inner));

    // write data to the memory block
    for (size_t i = 0; i < 10; i++) {
        ptr->data[i] = i;
    }
    ptr->value = 3.14;

    // read data from the memory block
    printf("[ ");
    for(size_t i = 0; i < 10; i++) {
        if(i < 9) printf("%d, ", ptr->data[i]); else printf("%d ", ptr->data[i]);
    }
    printf(" ]\n");
    printf("%f\n", ptr->value);
    
    // free memory
    vfree(ptr);

    return 0;
}
