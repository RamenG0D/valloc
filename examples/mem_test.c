#include "valloc.h"
#include <stdlib.h>
#include <stdio.h>

int main(void) {
    const size_t len = 1024; char* mem = malloc(len);
    Valloc* v = new_valloc(mem, len);
    
    const size_t tlen = 10;
    char* ptr = virtual_alloc(v, tlen);
    
    int i;
    for(i = 0; i < tlen; ++i) {
        ptr[i] = 'a' + i;
    }
    for(i = 0; i < tlen; ++i) {
        printf("%c", ptr[i]);
    }
    printf("\n");

    // now we realloc the mem to half its current size
    const size_t nlen = len / 2;
    char* nmem = virtual_realloc(v, ptr, nlen); // we free `ptr` here so its also no longer valid

    // virtual_free(v, nmem); // not needed because we free the associated memory with the allocator

    free_valloc(v);
    
    return 0;
}
