set -e

clang --target=riscv32 -march=rv32i -nostdlib test.c -Os -Wl,-T,test.ld -o test.o
llvm-objdump-14 -Mno-aliases -S test.o
llvm-objcopy-14 -O binary test.o test.bin

