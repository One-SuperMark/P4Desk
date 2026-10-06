#include "monitor_http_body.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

typedef struct {
    const char *data;
    size_t length, position, chunk;
    bool completed;
    int error;
} source_t;
static int read_part(void *context, uint8_t *out, size_t capacity)
{
    source_t *s = context;
    if (s->error) return s->error;
    size_t n = s->length - s->position;
    if (n > s->chunk) n = s->chunk;
    if (n > capacity) n = capacity;
    memcpy(out, s->data + s->position, n);
    s->position += n;
    return (int)n;
}
static bool complete(void *context) { return ((source_t *)context)->completed; }
int main(void)
{
    uint8_t out[10]; size_t length;
    source_t s = {.data = "abcdef", .length = 6, .chunk = 2, .completed = true};
    // Header parser has completed, but all six body bytes remain cached.
    assert(p4desk_read_http_body(&s, read_part, complete, out, 6, &length) == 0);
    assert(length == 6 && memcmp(out, "abcdef", 6) == 0);
    s.position = 0;
    memset(out, 0x5a, sizeof(out));
    assert(p4desk_read_http_body(&s, read_part, complete, out, 5, &length) == -2);
    assert(length == 5 && out[5] == 0x5a && out[9] == 0x5a);
    s.position = 0; s.completed = false;
    assert(p4desk_read_http_body(&s, read_part, complete, out, 10, &length) == -3);
    s.error = -1;
    assert(p4desk_read_http_body(&s, read_part, complete, out, 10, &length) == -1);
    s = (source_t){.data = "", .chunk = 2, .completed = true};
    assert(p4desk_read_http_body(&s, read_part, complete, out, 10, &length) == 0 && length == 0);
    puts("HTTP body: prefetched complete, fragmented, exact capacity, overflow guards, incomplete EOF and transport failure passed");
}
