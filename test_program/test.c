#include <stdint.h>

volatile uint32_t __attribute__((section(".input"))) input = 0;
volatile uint32_t __attribute__((section(".output"))) output = 0;

int _start() {
    output = 1;
    output += 0x3712;
    
    for(;;) { }
}

uint32_t vector_table[] __attribute__ ((used, section (".vector_table"))) = {
    (uint32_t)&_start,
    0,
};

