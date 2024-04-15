#include <stdio.h>

#include "valloc.h"

int main(void) {
    // init global allocator
    valloc_init(1024);

    // allocate 1 byte of memory (for a ptr)
    Pointer ptr = valloc(10);

    // data buffer
    uint8_t buffer[] = { 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A };

    // write to memory
    write_buffer(&ptr, buffer, sizeof(buffer));

    // read 10 bytes from memory
    BufferData value = read_buffer(&ptr, sizeof(buffer));
    for(size_t i = 0; i < value.len; i++) {
        printf("Value: %d\n", value.data[i]);
    }

    // free memory
    vfree(&ptr);

    return 0;
}
