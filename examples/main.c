#include <stdio.h>

static int g_myGlobal = 0;
static int g_myGlobal2 = 0;

// Address of the ELF header for a non-PIE object
char *start = (char*) 0x00000000400000;

void bar() {
    printf("bar");
}

void foo() {
    printf("foo");
}

int main() {
    printf("Hello World!\n");
    foo();
    bar();
    printf("main is located at %#010x\n", &main);
    printf("Start %c %c %c\n", start[1], start[2], start[3]);
    return 0;
}