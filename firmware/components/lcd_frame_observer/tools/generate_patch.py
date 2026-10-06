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
PATCH_VERSION = 5

# Each change has one exact anchor in the pinned SDK source. Keep the original
# refresh callbacks and DMA restart unchanged. The underrun ISR branch only
# records bounded diagnostics instead of printing from the interrupt.
REPLACEMENTS = (
    (
        "public_header",
        '#include "hal/color_hal.h"\n',
        '#include "hal/color_hal.h"\n'
        '#include "esp_private/esp_cache_private.h"\n'
        '#include "esp_timer.h"\n'
        '#include "freertos/task.h"\n'
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
    # The same lock makes task snapshots coherent with the bridge ISR.
    (
        "underrun_fields",
        "    uint8_t num_fbs;              // Number of frame buffers\n",
        "    p4desk_lcd_underrun_stats_t underrun_stats;\n"
        "    p4desk_lcd_cache_stats_t cache_stats;\n"
        "    p4desk_lcd_host_error_stats_t host_error_stats;\n"
        "    TaskHandle_t host_error_owner; // Sole task allowed to read-clear Host reports\n"
        "    uint8_t num_fbs;              // Number of frame buffers\n",
    ),
    (
        "unmask_all_visible_line_underruns",
        "    mipi_dsi_brg_ll_set_underrun_discard_count(hal->bridge, panel_config->video_timing.h_size);\n",
        "    // TRM Register 43.15: underruns before this line threshold are suppressed.\n"
        "    // A horizontal-size threshold can mask the entire visible frame.\n"
        "    // Observe every line; this changes diagnostics, not video timing.\n"
        "    mipi_dsi_brg_ll_set_underrun_discard_count(hal->bridge, 0);\n",
    ),
    (
        "bounded_underrun_diagnostics",
        "    if (intr_status & MIPI_DSI_BRG_LL_EVENT_UNDERRUN) {\n"
        "        // when an underrun happens, the LCD display may already becomes blue\n"
        "        // it's too late to recover the display, so we just print an error message\n"
        "        // as a hint to the user that he should optimize the memory bandwidth (with AXI-ICM)\n"
        '        ESP_DRAM_LOGE(TAG, "can\'t fetch data from external memory fast enough, underrun happens");\n'
        "    }\n",
        "    if (intr_status & MIPI_DSI_BRG_LL_EVENT_UNDERRUN) {\n"
        "        // Diagnostic only: no logging, allocation or panel reset in ISR.\n"
        "        const int64_t at_us = esp_timer_get_time();\n"
        "        portENTER_CRITICAL_ISR(&dpi_panel->frame_observer_lock);\n"
        "        if (dpi_panel->underrun_stats.count == 0) {\n"
        "            dpi_panel->underrun_stats.first_us = at_us;\n"
        "        }\n"
        "        if (dpi_panel->underrun_stats.count < UINT32_MAX) {\n"
        "            ++dpi_panel->underrun_stats.count;\n"
        "        }\n"
        "        dpi_panel->underrun_stats.last_us = at_us;\n"
        "        portEXIT_CRITICAL_ISR(&dpi_panel->frame_observer_lock);\n"
        "    }\n",
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
        "presentation_writeback_helper",
        "static bool dpi_panel_draw_bitmap_hook_end(esp_lcd_panel_t *panel)\n",
        "static esp_err_t p4desk_dpi_sync_for_present(esp_lcd_dpi_panel_t *dpi_panel, void *address, size_t bytes)\n"
        "{\n"
        "    const int64_t start_us = esp_timer_get_time();\n"
        "    const esp_err_t result = esp_cache_msync(address, bytes, ESP_CACHE_MSYNC_FLAG_DIR_C2M | ESP_CACHE_MSYNC_FLAG_UNALIGNED);\n"
        "    const uint64_t elapsed_us = (uint64_t)(esp_timer_get_time() - start_us);\n"
        "    portENTER_CRITICAL(&dpi_panel->frame_observer_lock);\n"
        "    p4desk_lcd_cache_stats_t *stats = &dpi_panel->cache_stats;\n"
        "    if (stats->calls < UINT32_MAX) ++stats->calls;\n"
        "    if (result != ESP_OK) {\n"
        "        if (stats->errors < UINT32_MAX) ++stats->errors;\n"
        "        stats->last_error = result;\n"
        "    }\n"
        "    const uint32_t bounded_us = elapsed_us > UINT32_MAX ? UINT32_MAX : (uint32_t)elapsed_us;\n"
        "    if (bounded_us > stats->max_us) stats->max_us = bounded_us;\n"
        "    stats->total_us = UINT64_MAX - stats->total_us < elapsed_us ? UINT64_MAX : stats->total_us + elapsed_us;\n"
        "    portEXIT_CRITICAL(&dpi_panel->frame_observer_lock);\n"
        "    return result;\n"
        "}\n\n"
        "static bool dpi_panel_draw_bitmap_hook_end(esp_lcd_panel_t *panel)\n",
    ),
    (
        "direct_writeback_checked_before_publish",
        "        esp_cache_msync(cache_sync_start, cache_sync_size, ESP_CACHE_MSYNC_FLAG_DIR_C2M | ESP_CACHE_MSYNC_FLAG_UNALIGNED);\n\n"
        "        dpi_panel->cur_fb_index = draw_buf_fb_index;\n",
        "        ESP_RETURN_ON_ERROR(p4desk_dpi_sync_for_present(dpi_panel, cache_sync_start, cache_sync_size),\n"
        "                            TAG, \"presentation cache writeback failed\");\n\n"
        "        dpi_panel->cur_fb_index = draw_buf_fb_index;\n",
    ),
    (
        "cpu_writeback_checked",
        "        esp_cache_msync(cache_sync_start, cache_sync_size, ESP_CACHE_MSYNC_FLAG_DIR_C2M | ESP_CACHE_MSYNC_FLAG_UNALIGNED);\n"
        "        // invoke the trans done callback\n",
        "        ESP_RETURN_ON_ERROR(p4desk_dpi_sync_for_present(dpi_panel, cache_sync_start, cache_sync_size),\n"
        "                            TAG, \"presentation cache writeback failed\");\n"
        "        // invoke the trans done callback\n",
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

// Project-local extension. Public DPI/VSYNC callbacks above remain unchanged.
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

esp_err_t p4desk_lcd_host_errors_poll(esp_lcd_panel_handle_t panel)
{
    ESP_RETURN_ON_FALSE(!xPortInIsrContext(), ESP_ERR_INVALID_STATE, TAG, "host polling requires task context");
    ESP_RETURN_ON_FALSE(panel && panel->draw_bitmap == dpi_panel_draw_bitmap &&
                        panel->draw_bitmap_2d == dpi_panel_draw_bitmap_2d,
                        ESP_ERR_INVALID_ARG, TAG, "host polling requires a DPI panel");
    esp_lcd_dpi_panel_t *dpi_panel = __containerof(panel, esp_lcd_dpi_panel_t, base);
    const TaskHandle_t caller = xTaskGetCurrentTaskHandle();
    portENTER_CRITICAL(&dpi_panel->frame_observer_lock);
    const bool authorized = !dpi_panel->host_error_owner || dpi_panel->host_error_owner == caller;
    if (authorized) dpi_panel->host_error_owner = caller;
    portEXIT_CRITICAL(&dpi_panel->frame_observer_lock);
    ESP_RETURN_ON_FALSE(authorized, ESP_ERR_INVALID_STATE, TAG, "host reports belong to the display owner");

    // ESP32-P4 TRM 43.4.2.4: each register is cleared by this read. Never poll
    // in the bridge ISR or the logging task, so no report can be silently lost.
    const uint32_t status0 = dpi_panel->bus->hal.host->int_st0.val;
    const uint32_t status1 = dpi_panel->bus->hal.host->int_st1.val;
    const int64_t at_us = esp_timer_get_time();
    portENTER_CRITICAL(&dpi_panel->frame_observer_lock);
    p4desk_lcd_host_error_stats_t *stats = &dpi_panel->host_error_stats;
    if (stats->polls < UINT32_MAX) ++stats->polls;
    stats->last_status0 = status0;
    stats->last_status1 = status1;
    stats->status0_or |= status0;
    stats->status1_or |= status1;
    if (status0 || status1) {
        if (!stats->error_polls) stats->first_error_us = at_us;
        if (stats->error_polls < UINT32_MAX) ++stats->error_polls;
        stats->last_error_us = at_us;
    }
    if ((status1 & P4DESK_DSI_HOST_DPI_FIFO_OVERFLOW) && stats->dpi_overflow_polls < UINT32_MAX)
        ++stats->dpi_overflow_polls;
    if ((status1 & P4DESK_DSI_HOST_DPI_FIFO_UNDERFLOW) && stats->dpi_underflow_polls < UINT32_MAX)
        ++stats->dpi_underflow_polls;
    portEXIT_CRITICAL(&dpi_panel->frame_observer_lock);
    return ESP_OK;
}

esp_err_t p4desk_lcd_host_error_stats(esp_lcd_panel_handle_t panel,
                                    p4desk_lcd_host_error_stats_t *out)
{
    if (out) *out = (p4desk_lcd_host_error_stats_t){0};
    ESP_RETURN_ON_FALSE(!xPortInIsrContext(), ESP_ERR_INVALID_STATE, TAG, "host snapshot requires task context");
    ESP_RETURN_ON_FALSE(panel && out && panel->draw_bitmap == dpi_panel_draw_bitmap &&
                        panel->draw_bitmap_2d == dpi_panel_draw_bitmap_2d,
                        ESP_ERR_INVALID_ARG, TAG, "host snapshot requires a DPI panel");
    esp_lcd_dpi_panel_t *dpi_panel = __containerof(panel, esp_lcd_dpi_panel_t, base);
    portENTER_CRITICAL(&dpi_panel->frame_observer_lock);
    *out = dpi_panel->host_error_stats;
    portEXIT_CRITICAL(&dpi_panel->frame_observer_lock);
    return ESP_OK;
}

esp_err_t p4desk_lcd_underrun_stats(esp_lcd_panel_handle_t panel,
                                  p4desk_lcd_underrun_stats_t *out)
{
    if (out) {
        *out = (p4desk_lcd_underrun_stats_t){0};
    }
    ESP_RETURN_ON_FALSE(!xPortInIsrContext(), ESP_ERR_INVALID_STATE, TAG, "underrun snapshot requires task context");
    ESP_RETURN_ON_FALSE(panel && out, ESP_ERR_INVALID_ARG, TAG, "invalid underrun snapshot");
    ESP_RETURN_ON_FALSE(panel->draw_bitmap == dpi_panel_draw_bitmap &&
                        panel->draw_bitmap_2d == dpi_panel_draw_bitmap_2d,
                        ESP_ERR_INVALID_ARG, TAG, "underrun snapshot requires a DPI panel");
    esp_lcd_dpi_panel_t *dpi_panel = __containerof(panel, esp_lcd_dpi_panel_t, base);
    portENTER_CRITICAL(&dpi_panel->frame_observer_lock);
    *out = dpi_panel->underrun_stats;
    portEXIT_CRITICAL(&dpi_panel->frame_observer_lock);
    return ESP_OK;
}

esp_err_t p4desk_lcd_cache_stats(esp_lcd_panel_handle_t panel,
                               p4desk_lcd_cache_stats_t *out)
{
    if (out) *out = (p4desk_lcd_cache_stats_t){0};
    ESP_RETURN_ON_FALSE(!xPortInIsrContext(), ESP_ERR_INVALID_STATE, TAG, "cache snapshot requires task context");
    ESP_RETURN_ON_FALSE(panel && out, ESP_ERR_INVALID_ARG, TAG, "invalid cache snapshot");
    ESP_RETURN_ON_FALSE(panel->draw_bitmap == dpi_panel_draw_bitmap &&
                        panel->draw_bitmap_2d == dpi_panel_draw_bitmap_2d,
                        ESP_ERR_INVALID_ARG, TAG, "cache snapshot requires a DPI panel");
    esp_lcd_dpi_panel_t *dpi_panel = __containerof(panel, esp_lcd_dpi_panel_t, base);
    portENTER_CRITICAL(&dpi_panel->frame_observer_lock);
    *out = dpi_panel->cache_stats;
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
            "underrun_diagnostics": "saturating count and monotonic first/last timestamps; task snapshot; no ISR logging",
            "presentation_cache_diagnostics": "existing writeback checked before publishing; saturating count/error/duration; no extra cache operations",
            "host_error_diagnostics": "one task reads INT_ST0/1 once per poll; RAM snapshot retains read-clear reports; poll counts may coalesce hardware events",
            "underrun_discard_line_threshold": 0,
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
