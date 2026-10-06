/*
 * SPDX-License-Identifier: MIT
 */
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "esp_err.h"
#include "esp_lcd_types.h"

#ifdef __cplusplus
extern "C" {
#endif

/** One complete DMA source read, followed by the actual next DMA selection. */
typedef struct {
    const void *completed_fb;  /**< Source whose full DMA transfer just completed. */
    const void *next_fb;       /**< Source selected for the DMA transfer already restarted. */
    uint32_t counter;          /**< Full-transfer count, starting at 1 after panel init; wraps. */
    uint8_t completed_index;   /**< Index of completed_fb in the DPI framebuffer array. */
    uint8_t next_index;        /**< Index of next_fb in the DPI framebuffer array. */
} p4desk_lcd_frame_event_t;

/**
 * @brief Observe an actual DPI DMA frame boundary in ISR context.
 *
 * The event and its pointers describe DMA ownership, not optical presentation.
 * The event itself is valid only during this callback. Framebuffer allocations
 * remain valid until panel deletion. If completed_fb == next_fb, that buffer is
 * being read again and must not be overwritten. A single display owner must
 * still serialize rendering and selecting buffers.
 *
 * Keep this callback short and nonblocking. Return true when a higher priority
 * task was woken. The original on_refresh_done / VSYNC callbacks are retained.
 */
typedef bool (*p4desk_lcd_frame_observer_cb_t)(esp_lcd_panel_handle_t panel,
                                             const p4desk_lcd_frame_event_t *event,
                                             void *context);

/**
 * @brief Register or replace the project-local ESP-IDF 6.0.2 DPI observer.
 *
 * Task context only; callback == NULL unregisters. The callback/context pair is
 * atomically copied by the DMA ISR, and callbacks run outside the spinlock.
 * A previously copied callback can still run once after replacement or
 * unregistering. Keep every old context alive until DMA has stopped or the
 * panel has been deleted. A static context registered once is recommended.
 * Do not register concurrently with panel initialization or deletion.
 *
 * Handles returned by EK79007's DPI wrapper are supported. When
 * CONFIG_LCD_DSI_ISR_CACHE_SAFE is enabled, the callback must be in IRAM and
 * context, including everything the callback touches, must be in internal RAM.
 *
 * @return ESP_OK on success, ESP_ERR_INVALID_STATE from ISR context,
 *         ESP_ERR_INVALID_ARG for a null/non-DPI panel or an unsafe callback.
 */
esp_err_t p4desk_lcd_frame_observer_register(esp_lcd_panel_handle_t panel,
                                           p4desk_lcd_frame_observer_cb_t callback,
                                           void *context);

/** Bounded bridge-underrun diagnostics; timestamps use the ESP monotonic clock. */
typedef struct {
    uint32_t count;     /**< Cumulative count since panel creation; saturates at UINT32_MAX. */
    int64_t first_us;   /**< First underrun timestamp; zero with count == 0. */
    int64_t last_us;    /**< Latest underrun timestamp, including after count saturation. */
} p4desk_lcd_underrun_stats_t;

/**
 * @brief Read one coherent underrun snapshot without resetting its counters.
 *
 * Task context only. Handles returned by EK79007's DPI wrapper are supported.
 * The bridge ISR only increments the counter and records monotonic timestamps;
 * it does not log, allocate, reset the panel, or modify framebuffer ownership.
 * Query at a bounded diagnostic cadence (for example every 30 seconds), not per
 * frame. count == 0 means no underrun was observed since panel creation.
 * Do not query concurrently with panel initialization or deletion. *out is
 * cleared on failure when non-NULL.
 */
esp_err_t p4desk_lcd_underrun_stats(esp_lcd_panel_handle_t panel,
                                  p4desk_lcd_underrun_stats_t *out);

/** Presentation writeback timing only; no framebuffer contents or addresses. */
typedef struct {
    uint32_t calls;       /**< Cumulative attempts, saturates. */
    uint32_t errors;      /**< Failed writebacks, saturates. */
    uint32_t max_us;      /**< Longest whole cache call, including chunk scheduling. */
    uint64_t total_us;    /**< Cumulative call duration, saturates. */
    int32_t last_error;   /**< Most recent failing esp_err_t; zero until first error. */
} p4desk_lcd_cache_stats_t;

/** Task-context coherent snapshot, same handle/lifetime rules as underruns. */
esp_err_t p4desk_lcd_cache_stats(esp_lcd_panel_handle_t panel,
                               p4desk_lcd_cache_stats_t *out);

#define P4DESK_DSI_HOST_DPI_FIFO_OVERFLOW (UINT32_C(1) << 7)
#define P4DESK_DSI_HOST_DPI_FIFO_UNDERFLOW (UINT32_C(1) << 19)

/** Read-clear Host errors accumulated by the sole display-owner task. */
typedef struct {
    uint32_t polls, error_polls;
    uint32_t status0_or, status1_or;
    uint32_t dpi_overflow_polls, dpi_underflow_polls;
    uint32_t last_status0, last_status1;
    int64_t first_error_us, last_error_us; /**< Observation times, not hardware occurrence times. */
} p4desk_lcd_host_error_stats_t;

/**
 * @brief Poll Host INT_ST0/1 exactly once and retain their read-clear reports.
 *
 * Task context only. The first caller becomes the polling owner for this panel;
 * calls from another task are rejected before any register read. Use only from
 * the single display-owner task; do not also read these registers in another
 * diagnostic or ISR. This operation clears hardware reports as documented by
 * ESP32-P4 TRM section 43.4.2.4. It never changes masks, timing or transmission.
 * Counters count polls with each flag, not hardware events: multiple events may
 * coalesce between polls. No logging or framebuffer-content inspection occurs.
 * Do not call concurrently with panel initialization/deletion or Host reset.
 */
esp_err_t p4desk_lcd_host_errors_poll(esp_lcd_panel_handle_t panel);

/** Task-context coherent RAM snapshot; does not read/clear hardware status. */
esp_err_t p4desk_lcd_host_error_stats(esp_lcd_panel_handle_t panel,
                                    p4desk_lcd_host_error_stats_t *out);

/**
 * @brief Get the reserved writable capacity of an exact DPI framebuffer.
 *
 * Task context only. fb must equal one of the panel's framebuffer base pointers;
 * an interior pointer or an unrelated allocation is rejected. Capacity includes
 * allocation-only padding to a multiple of 16 rows and the PSRAM cache alignment.
 * DPI timing, DMA transfer length, visible height and drawing geometry remain
 * unchanged. For a 1024x600 RGB565 panel this is 1,245,184 bytes (608 rows).
 *
 * This query does not acquire ownership: the display owner must separately hold
 * FREE/BUILDING ownership before writing or decoding into the buffer. A success
 * here never makes a currently scanning or pending buffer writable. Do not query
 * concurrently with panel initialization or deletion. *out is zero on failure
 * when out is non-NULL.
 */
esp_err_t p4desk_lcd_frame_buffer_capacity(esp_lcd_panel_handle_t panel,
                                         const void *fb,
                                         size_t *out);

#ifdef __cplusplus
}
#endif
