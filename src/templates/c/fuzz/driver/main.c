#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* External function that users implement */
extern int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

/* Optional initialization function that users can implement */
__attribute__((weak)) int LLVMFuzzerInitialize(int *argc, char ***argv) {
    /* Default empty implementation */
    return 0;
}

#ifdef __AFL_COMPILER
/* AFL persistent mode support */
extern int __afl_persistent_loop(unsigned int);
extern void __afl_manual_init(void);
#endif

/* Main entry point - works with libFuzzer, AFL, HonggFuzz, or standalone */
int main(int argc, char **argv) {
    /* Call user initialization if provided */
    if (LLVMFuzzerInitialize) {
        LLVMFuzzerInitialize(&argc, &argv);
    }

#ifdef __AFL_COMPILER
    /* AFL mode - use persistent loop for performance */
    __afl_manual_init();
    
    while (__afl_persistent_loop(1000)) {
        static uint8_t input_buf[1024 * 1024]; /* 1MB max input */
        ssize_t len = read(0, input_buf, sizeof(input_buf) - 1);
        if (len >= 0) {
            LLVMFuzzerTestOneInput(input_buf, (size_t)len);
        }
    }
    
#elif defined(__libfuzzer__)
    /* libFuzzer mode - this shouldn't be reached as libFuzzer provides its own main */
    fprintf(stderr, "Error: This binary was built for libFuzzer but is being run directly\n");
    fprintf(stderr, "Use: ./fuzzer CORPUS_DIR\n");
    return 1;
    
#else
    /* Standalone mode - read from stdin or files */
    if (argc > 1) {
        /* File mode - process each file as input */
        for (int i = 1; i < argc; i++) {
            FILE *f = fopen(argv[i], "rb");
            if (!f) {
                perror(argv[i]);
                continue;
            }
            
            /* Get file size */
            fseek(f, 0, SEEK_END);
            long size = ftell(f);
            fseek(f, 0, SEEK_SET);
            
            if (size > 0 && size < 1024 * 1024) { /* 1MB limit */
                uint8_t *data = malloc(size);
                if (data && fread(data, 1, size, f) == (size_t)size) {
                    printf("Testing %s (%ld bytes)\n", argv[i], size);
                    LLVMFuzzerTestOneInput(data, size);
                }
                free(data);
            }
            fclose(f);
        }
    } else {
        /* Stdin mode */
        static uint8_t input_buf[1024 * 1024]; /* 1MB max input */
        ssize_t len = read(0, input_buf, sizeof(input_buf) - 1);
        if (len > 0) {
            LLVMFuzzerTestOneInput(input_buf, (size_t)len);
        }
    }
#endif

    return 0;
}