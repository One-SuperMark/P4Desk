#include "display_pixels.h"

#include <string.h>

void p4desk_pixels_copy_rgb565(uint16_t *destination, const uint16_t *source,
                              size_t width, size_t height, size_t source_stride,
                              bool rotate_180)
{
    for (size_t y = 0; y < height; y++) {
        const uint16_t *input = source + y * source_stride;
        uint16_t *output = destination + (rotate_180 ? height - 1 - y : y) * width;
        if (rotate_180) {
            for (size_t x = 0; x < width; x++) output[width - 1 - x] = input[x];
        } else memcpy(output, input, width * sizeof(uint16_t));
    }
}

void p4desk_pixels_copy_gray565(uint16_t *destination, const uint8_t *source,
                               size_t width, size_t height, size_t source_stride,
                               bool rotate_180)
{
    for (size_t y = 0; y < height; y++) {
        const uint8_t *input = source + y * source_stride;
        uint16_t *output = destination + (rotate_180 ? height - 1 - y : y) * width;
        for (size_t x = 0; x < width; x++) {
            const uint8_t v = input[x];
            const uint16_t rgb = ((uint16_t)(v >> 3) << 11) | ((uint16_t)(v >> 2) << 5) | (v >> 3);
            output[rotate_180 ? width - 1 - x : x] = rgb;
        }
    }
}
