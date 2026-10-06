#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef int (*p4desk_body_read_fn)(void *, uint8_t *, size_t);
typedef bool (*p4desk_body_complete_fn)(void *);

// Drain cached body bytes even when the HTTP parser already reports complete.
// Return 0 on complete EOF, -1 on read failure, -2 on overflow, -3 on truncation.
static inline int p4desk_read_http_body(void *context, p4desk_body_read_fn read_body,
                                      p4desk_body_complete_fn complete,
                                      uint8_t *out, size_t capacity, size_t *length)
{
    *length = 0;
    for (;;) {
        uint8_t extra;
        const size_t available = capacity - *length;
        const size_t count = available ? available : 1;
        int n = read_body(context, available ? out + *length : &extra, count);
        if (n < 0 || (size_t)n > count) return -1;
        if (n == 0) return complete(context) ? 0 : -3;
        if (!available) return -2;
        *length += (size_t)n;
    }
}
