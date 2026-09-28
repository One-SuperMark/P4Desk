#include "display_pixels.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void rgb_orientation_and_padding(void)
{
    const uint16_t source[] = {11,12,13,14,999,999, 21,22,23,24,999,999, 31,32,33,34,999,999};
    const uint16_t reversed[] = {34,33,32,31, 24,23,22,21, 14,13,12,11};
    const uint16_t forward[] = {11,12,13,14, 21,22,23,24, 31,32,33,34};
    uint16_t buffer[14] = {0};
    buffer[0] = buffer[13] = 0xbeef;
    p4desk_pixels_copy_rgb565(buffer + 1, source, 4, 3, 6, true);
    assert(memcmp(buffer + 1, reversed, sizeof(reversed)) == 0);
    // Rebuilding from the canonical source must not alternate the orientation.
    p4desk_pixels_copy_rgb565(buffer + 1, source, 4, 3, 6, true);
    assert(memcmp(buffer + 1, reversed, sizeof(reversed)) == 0);
    p4desk_pixels_copy_rgb565(buffer + 1, source, 4, 3, 6, false);
    assert(memcmp(buffer + 1, forward, sizeof(forward)) == 0);
    assert(buffer[0] == 0xbeef && buffer[13] == 0xbeef);
}

static void visible_600_rows_exclude_mcu_padding(void)
{
    const size_t width = 1024, height = 600, stride = 1056, decoded_rows = 608;
    uint16_t *source = malloc(stride * decoded_rows * sizeof(uint16_t));
    uint16_t *guarded = malloc((width * height + 2) * sizeof(uint16_t));
    assert(source && guarded);
    for (size_t i = 0; i < stride * decoded_rows; i++) source[i] = 0xfbad;
    for (size_t y = 0; y < height; y++)
        for (size_t x = 0; x < width; x++) source[y * stride + x] = (uint16_t)(y + 1);
    guarded[0] = guarded[width * height + 1] = 0xbeef;
    p4desk_pixels_copy_rgb565(guarded + 1, source, width, height, stride, true);
    for (size_t y = 0; y < height; y++)
        for (size_t x = 0; x < width; x++) assert(guarded[1 + y * width + x] == height - y);
    assert(guarded[0] == 0xbeef && guarded[width * height + 1] == 0xbeef);
    free(guarded);
    free(source);
}

static void gray_conversion_and_orientation(void)
{
    const uint8_t source[] = {0,128,255,77, 63,191,200,77};
    const uint16_t expected[] = {0xce59,0xbdf7,0x39e7, 0xffff,0x8410,0x0000};
    uint16_t destination[6];
    p4desk_pixels_copy_gray565(destination, source, 3, 2, 4, true);
    assert(memcmp(destination, expected, sizeof(expected)) == 0);
}

int main(void)
{
    rgb_orientation_and_padding();
    visible_600_rows_exclude_mcu_padding();
    gray_conversion_and_orientation();
    puts("display pixel rotation: RGB565, 600/608 row crop, grayscale and guards passed");
    return 0;
}
