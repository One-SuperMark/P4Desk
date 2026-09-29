#!/usr/bin/env python3
"""Generate the project-local observer extension for the pinned IDF DPI driver.

The SDK input is read only. A SHA mismatch or non-unique anchor fails before
writing any output. Generated code retains the original Apache-2.0 header.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys


PINNED_SHA256 = "7a5acacd2be4f4560b2ae85e2414aa3c5c36d313f58e62208e24539ac33e607a"
PATCH_VERSION = 2

# Each change has one exact anchor in the pinned SDK source. Keep the original
# refresh callback and bridge ISR bodies unchanged.
REPLACEMENTS = (
    (
        "public_header",
        '#include "hal/color_hal.h"\n',
        '#include "hal/color_hal.h"\n'
        '#include "esp_private/esp_cache_private.h"\n'
        '#include "p4desk_lcd_frame_observer.h"\n',
    ),
    (
        "allocation_capacity_field",
        "    size_t fb_size;               // Frame buffer size, in bytes\n",
        "    size_t fb_size;               // Frame buffer size, in bytes\n"
        "    size_t fb_capacity;           // Reserved allocation, including invisible JPEG MCU padding\n",
    ),
    (
        "allocation_only_row_padding",
        "    // allocate frame buffer from PSRAM\n"
        "    size_t fb_size = panel_config->video_timing.h_size * panel_config->video_timing.v_size * bits_per_pixel / 8;\n"
        "    for (int i = 0; i < num_fbs; i++) {\n"
        "        uint8_t *frame_buffer = heap_caps_calloc(1, fb_size, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT | MALLOC_CAP_DMA);\n",
        "    // allocate frame buffer from PSRAM\n"
        "    size_t fb_size = panel_config->video_timing.h_size * panel_config->video_timing.v_size * bits_per_pixel / 8;\n"
        "    // Reserve invisible MCU padding only. fb_size and every DPI scan/draw\n"
        "    // geometry below remain the original physical dimensions.\n"
        "    size_t fb_alignment = 0;\n"
        "    ESP_GOTO_ON_ERROR(esp_cache_get_alignment(MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT | MALLOC_CAP_DMA, &fb_alignment),\n"
        '                      err, TAG, "frame buffer cache alignment unavailable");\n'
        "    ESP_GOTO_ON_FALSE(fb_alignment && !(fb_alignment & (fb_alignment - 1)),\n"
        '                      ESP_ERR_INVALID_STATE, err, TAG, "invalid frame buffer cache alignment");\n'
        "    size_t padded_rows = 0, fb_capacity = 0;\n"
        "    ESP_GOTO_ON_FALSE(!__builtin_add_overflow((size_t)panel_config->video_timing.v_size, (size_t)15, &padded_rows),\n"
        '                      ESP_ERR_INVALID_ARG, err, TAG, "frame buffer rows overflow");\n'
        "    padded_rows &= ~(size_t)15;\n"
        "    ESP_GOTO_ON_FALSE(!__builtin_mul_overflow((size_t)panel_config->video_timing.h_size, padded_rows, &fb_capacity) &&\n"
        "                      !__builtin_mul_overflow(fb_capacity, bits_per_pixel, &fb_capacity),\n"
        '                      ESP_ERR_INVALID_ARG, err, TAG, "frame buffer capacity overflow");\n'
        "    fb_capacity /= 8;\n"
        "    ESP_GOTO_ON_FALSE(!__builtin_add_overflow(fb_capacity, fb_alignment - 1, &fb_capacity),\n"
        '                      ESP_ERR_INVALID_ARG, err, TAG, "frame buffer aligned capacity overflow");\n'
        "    fb_capacity &= ~(fb_alignment - 1);\n"
        "    dpi_panel->fb_capacity = fb_capacity;\n"
        "    for (int i = 0; i < num_fbs; i++) {\n"
        "        uint8_t *frame_buffer = heap_caps_aligned_calloc(fb_alignment, 1, fb_capacity, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT | MALLOC_CAP_DMA);\n",
    ),
    (
        "observer_fields",
        "    uint8_t cur_fb_index;         // Current frame buffer index\n",
        "    uint8_t cur_fb_index;         // Current frame buffer index\n"
        "    // Project-local observer: DMA source ownership, not optical VSYNC.\n"
        "    portMUX_TYPE frame_observer_lock;\n"
        "    uint8_t scanning_fb_index;\n"
        "    uint32_t frame_observer_counter;\n"
        "    p4desk_lcd_frame_observer_cb_t frame_observer_callback;\n"
        "    void *frame_observer_context;\n",
    ),
    (
        "dma_selection_snapshot",
        "    uint8_t fb_index = dpi_panel->cur_fb_index;\n"
        "    dw_gdma_link_list_handle_t link_list = dpi_panel->link_lists[fb_index];\n",
        "    // Snapshot the previous source and the exact list selected below.\n"
        "    // A task may publish a later cur_fb_index after this snapshot; that\n"
        "    // later selection is not attributed to this DMA transfer.\n"
        "    portENTER_CRITICAL_ISR(&dpi_panel->frame_observer_lock);\n"
        "    uint8_t completed_index = dpi_panel->scanning_fb_index;\n"
        "    uint8_t fb_index = dpi_panel->cur_fb_index;\n"
        "    dpi_panel->scanning_fb_index = fb_index;\n"
        "    uint32_t counter = ++dpi_panel->frame_observer_counter;\n"
        "    p4desk_lcd_frame_observer_cb_t observer = dpi_panel->frame_observer_callback;\n"
        "    void *observer_context = dpi_panel->frame_observer_context;\n"
        "    portEXIT_CRITICAL_ISR(&dpi_panel->frame_observer_lock);\n"
        "    p4desk_lcd_frame_event_t observer_event = {\n"
        "        .completed_fb = dpi_panel->fbs[completed_index],\n"
        "        .next_fb = dpi_panel->fbs[fb_index],\n"
        "        .counter = counter,\n"
        "        .completed_index = completed_index,\n"
        "        .next_index = fb_index,\n"
        "    };\n"
        "    dw_gdma_link_list_handle_t link_list = dpi_panel->link_lists[fb_index];\n",
    ),
    (
        "observer_after_dma_restart",
        "    dw_gdma_channel_enable_ctrl(chan, true);\n\n"
        "#if !MIPI_DSI_BRG_LL_EVENT_VSYNC\n",
        "    dw_gdma_channel_enable_ctrl(chan, true);\n\n"
        "    // Observe only after the original DMA restart, outside the lock.\n"
        "    if (observer && observer(&dpi_panel->base, &observer_event, observer_context)) {\n"
        "        yield_needed = true;\n"
        "    }\n\n"
        "#if !MIPI_DSI_BRG_LL_EVENT_VSYNC\n",
    ),
    (
        "lock_initialization",
        '    ESP_GOTO_ON_FALSE(dpi_panel, ESP_ERR_NO_MEM, err, TAG, "no memory for DPI panel");\n',
        '    ESP_GOTO_ON_FALSE(dpi_panel, ESP_ERR_NO_MEM, err, TAG, "no memory for DPI panel");\n'
        "    portMUX_INITIALIZE(&dpi_panel->frame_observer_lock);\n",
    ),
    (
        "initial_scanning_source",
        "    // by default, we use the fb0 as the first working frame buffer\n"
        "    dpi_panel->cur_fb_index = 0;\n"
        "    link_list = dpi_panel->link_lists[0];\n",
        "    // by default, we use the fb0 as the first working frame buffer\n"
        "    portENTER_CRITICAL(&dpi_panel->frame_observer_lock);\n"
        "    dpi_panel->cur_fb_index = 0;\n"
        "    dpi_panel->scanning_fb_index = 0;\n"
        "    dpi_panel->frame_observer_counter = 0;\n"
        "    portEXIT_CRITICAL(&dpi_panel->frame_observer_lock);\n"
        "    link_list = dpi_panel->link_lists[0];\n",
    ),
    (
        "draw_source_snapshot",
        "    uint8_t cur_fb_index = dpi_panel->cur_fb_index;\n"
        "    uint8_t *frame_buffer = dpi_panel->fbs[cur_fb_index];\n",
        "    portENTER_CRITICAL(&dpi_panel->frame_observer_lock);\n"
        "    uint8_t cur_fb_index = dpi_panel->cur_fb_index;\n"
        "    portEXIT_CRITICAL(&dpi_panel->frame_observer_lock);\n"
        "    uint8_t *frame_buffer = dpi_panel->fbs[cur_fb_index];\n",
    ),
    (
        "publish_after_cache_writeback",
        "        dpi_panel->cur_fb_index = draw_buf_fb_index;\n"
        "        // invoke the trans done callback\n",
        "        portENTER_CRITICAL(&dpi_panel->frame_observer_lock);\n"
        "        dpi_panel->cur_fb_index = draw_buf_fb_index;\n"
        "        portEXIT_CRITICAL(&dpi_panel->frame_observer_lock);\n"
        "        // invoke the trans done callback\n",
    ),
)

REGISTER_IMPLEMENTATION = r'''

// Project-local extension. The original public DPI/VSYNC callbacks above remain unchanged.
esp_err_t p4desk_lcd_frame_observer_register(esp_lcd_panel_handle_t panel,
                                           p4desk_lcd_frame_observer_cb_t callback,
                                           void *context)
{
    ESP_RETURN_ON_FALSE(!xPortInIsrContext(), ESP_ERR_INVALID_STATE, TAG, "observer registration requires task context");
    ESP_RETURN_ON_FALSE(panel, ESP_ERR_INVALID_ARG, TAG, "invalid panel");
    // EK79007 replaces del/init/reset but retains these DPI drawing methods.
    ESP_RETURN_ON_FALSE(panel->draw_bitmap == dpi_panel_draw_bitmap &&
                        panel->draw_bitmap_2d == dpi_panel_draw_bitmap_2d,
                        ESP_ERR_INVALID_ARG, TAG, "observer requires a DPI panel");
#if CONFIG_LCD_DSI_ISR_CACHE_SAFE
    if (callback) {
        ESP_RETURN_ON_FALSE(esp_ptr_in_iram(callback), ESP_ERR_INVALID_ARG, TAG, "observer callback not in IRAM");
    }
    if (callback && context) {
        ESP_RETURN_ON_FALSE(esp_ptr_internal(context), ESP_ERR_INVALID_ARG, TAG, "observer context not in internal RAM");
    }
#endif // CONFIG_LCD_DSI_ISR_CACHE_SAFE
    esp_lcd_dpi_panel_t *dpi_panel = __containerof(panel, esp_lcd_dpi_panel_t, base);
    portENTER_CRITICAL(&dpi_panel->frame_observer_lock);
    dpi_panel->frame_observer_callback = callback;
    dpi_panel->frame_observer_context = callback ? context : NULL;
    portEXIT_CRITICAL(&dpi_panel->frame_observer_lock);
    return ESP_OK;
}

esp_err_t p4desk_lcd_frame_buffer_capacity(esp_lcd_panel_handle_t panel,
                                         const void *fb,
                                         size_t *out)
{
    if (out) {
        *out = 0;
    }
    ESP_RETURN_ON_FALSE(!xPortInIsrContext(), ESP_ERR_INVALID_STATE, TAG, "capacity query requires task context");
    ESP_RETURN_ON_FALSE(panel && fb && out, ESP_ERR_INVALID_ARG, TAG, "invalid capacity query");
    // Accept EK79007's wrapper; it preserves the DPI drawing methods.
    ESP_RETURN_ON_FALSE(panel->draw_bitmap == dpi_panel_draw_bitmap &&
                        panel->draw_bitmap_2d == dpi_panel_draw_bitmap_2d,
                        ESP_ERR_INVALID_ARG, TAG, "capacity query requires a DPI panel");
    esp_lcd_dpi_panel_t *dpi_panel = __containerof(panel, esp_lcd_dpi_panel_t, base);
    for (uint8_t index = 0; index < dpi_panel->num_fbs; ++index) {
        if (fb == dpi_panel->fbs[index]) {
            *out = dpi_panel->fb_capacity;
            return ESP_OK;
        }
    }
    return ESP_ERR_INVALID_ARG;
}
'''


def patch_source(source: bytes, *, verify_sha: bool = True) -> bytes:
    """Validate all anchors before producing any edited text."""
    actual_sha = hashlib.sha256(source).hexdigest()
    if verify_sha and actual_sha != PINNED_SHA256:
        raise ValueError(f"ESP-IDF 6.0.2 DPI SHA256 mismatch: {actual_sha}")
    text = source.decode("utf-8")
    for name, before, _ in REPLACEMENTS:
        count = text.count(before)
        if count != 1:
            raise ValueError(f"anchor {name!r} must occur exactly once; found {count}")
    if "p4desk_lcd_frame_observer_register(" in text:
        raise ValueError("DPI source already contains the observer extension")
    for _, before, after in REPLACEMENTS:
        text = text.replace(before, after, 1)
    return (text + REGISTER_IMPLEMENTATION).encode("utf-8")


def write_if_changed(path: Path, contents: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists() or path.read_bytes() != contents:
        path.write_bytes(contents)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True, help="original SDK DPI source, read only")
    parser.add_argument("--output", type=Path, required=True, help="generated source under the build directory")
    parser.add_argument("--manifest", type=Path, help="optional machine-readable patch provenance")
    args = parser.parse_args()
    try:
        original = args.input.resolve(strict=True)
        output = args.output.resolve()
        if original == output:
            raise ValueError("the output must not overwrite the SDK input")
        if args.manifest and args.manifest.resolve() in (original, output):
            raise ValueError("the manifest must be separate from input and output")
        source = original.read_bytes()
        patched = patch_source(source)
        manifest = {
            "idf_version": "6.0.2",
            "patch_version": PATCH_VERSION,
            "source_sha256": PINNED_SHA256,
            "generated_sha256": hashlib.sha256(patched).hexdigest(),
            "source_bytes": len(source),
            "generated_bytes": len(patched),
            "anchors": [name for name, _, _ in REPLACEMENTS],
            "source_license": "Apache-2.0",
            "allocation_padding_rows_multiple": 16,
            "scan_geometry_modified": False,
        }
        write_if_changed(output, patched)
        if args.manifest:
            write_if_changed(args.manifest, (json.dumps(manifest, indent=2) + "\n").encode("utf-8"))
        print(f"DPI observer generated: {manifest['generated_sha256']} ({len(patched)} bytes)")
        return 0
    except (OSError, UnicodeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
