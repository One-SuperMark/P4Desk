#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// The source remains canonical; destination must be a separate owned buffer.
// Copy only width*height visible pixels, ignoring row and MCU padding.
void p4desk_pixels_copy_rgb565(uint16_t *destination, const uint16_t *source,
                              size_t width, size_t height, size_t source_stride,
                              bool rotate_180);
void p4desk_pixels_copy_gray565(uint16_t *destination, const uint8_t *source,
                               size_t width, size_t height, size_t source_stride,
                               bool rotate_180);
