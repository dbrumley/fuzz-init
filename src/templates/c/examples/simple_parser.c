// Example fuzz target for a simple key-value parser
// This demonstrates how to write a more realistic fuzz target

#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Simple key-value structure
typedef struct {
    char *key;
    char *value;
} KeyValue;

// Simple parser that's intentionally buggy for demonstration
KeyValue* parse_key_value(const char *input) {
    if (!input) return NULL;
    
    // Find the '=' separator
    char *eq_pos = strchr(input, '=');
    if (!eq_pos) return NULL;
    
    KeyValue *kv = malloc(sizeof(KeyValue));
    if (!kv) return NULL;
    
    // Extract key (everything before '=')
    size_t key_len = eq_pos - input;
    kv->key = malloc(key_len + 1);
    strncpy(kv->key, input, key_len);
    kv->key[key_len] = '\0';
    
    // Extract value (everything after '=')
    char *value_start = eq_pos + 1;
    size_t value_len = strlen(value_start);
    kv->value = malloc(value_len + 1);
    strcpy(kv->value, value_start);  // BUG: No bounds checking!
    
    return kv;
}

void free_key_value(KeyValue *kv) {
    if (kv) {
        free(kv->key);
        free(kv->value);
        free(kv);
    }
}

// Fuzz target
int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    // Ensure null termination
    if (size == 0) return 0;
    
    char *input = malloc(size + 1);
    if (!input) return 0;
    
    memcpy(input, data, size);
    input[size] = '\0';
    
    // Test the parser
    KeyValue *kv = parse_key_value(input);
    if (kv) {
        // Use the parsed data
        printf("Parsed: %s = %s\n", kv->key, kv->value);
        free_key_value(kv);
    }
    
    free(input);
    return 0;
}