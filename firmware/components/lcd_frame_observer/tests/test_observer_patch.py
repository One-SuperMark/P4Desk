#!/usr/bin/env python3
"""Bounded host checks for the pinned driver extension; never opens hardware."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


COMPONENT = Path(__file__).resolve().parents[1]
TOOL = COMPONENT / "tools/generate_patch.py"
spec = importlib.util.spec_from_file_location("dpi_patch", TOOL)
patch = importlib.util.module_from_spec(spec)
spec.loader.exec_module(patch)
# Initialize on import as well as direct execution so unittest discovery works.
# An explicit environment/CLI path takes precedence over the local pinned SDK.
DEFAULT_IDF_PATH = Path(os.environ.get("IDF_PATH") or "/Volumes/work/esp/esp-idf-v6.0.2").expanduser().resolve()
SDK_SOURCE = DEFAULT_IDF_PATH / "components/esp_lcd/dsi/esp_lcd_panel_dpi.c"


def function_source(text: str, name: str, next_name: str | None = None) -> str:
    start = text.index(name)
    if next_name:
        return text[start:text.index(next_name, start)].strip()
    return text[start:].strip()


class ObserverPatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if not SDK_SOURCE.is_file():
            raise unittest.SkipTest(
                f"Pinned ESP-IDF 6.0.2 DPI source not found: {SDK_SOURCE}; "
                "set IDF_PATH or run this test file with --idf-path PATH"
            )
        cls.source = SDK_SOURCE.read_bytes()
        cls.patched = patch.patch_source(cls.source)
        cls.original_text = cls.source.decode("utf-8")
        cls.patched_text = cls.patched.decode("utf-8")

    def test_pinned_source_and_unique_anchors(self) -> None:
        self.assertEqual(hashlib.sha256(self.source).hexdigest(), patch.PINNED_SHA256)
        for name, before, _ in patch.REPLACEMENTS:
            with self.subTest(anchor=name):
                self.assertEqual(self.original_text.count(before), 1)

    def test_rejects_changed_source_and_nonunique_anchor(self) -> None:
        with self.assertRaisesRegex(ValueError, "SHA256 mismatch"):
            patch.patch_source(self.source + b"\n")
        before = patch.REPLACEMENTS[0][1]
        with self.assertRaisesRegex(ValueError, "found 2"):
            patch.patch_source(self.source + before.encode(), verify_sha=False)
        with self.assertRaisesRegex(ValueError, "found 0"):
            patch.patch_source(self.source.replace(before.encode(), b""), verify_sha=False)

    def test_preserves_license_bridge_and_original_refresh_callbacks(self) -> None:
        self.assertEqual(self.source.split(b"*/", 1)[0], self.patched.split(b"*/", 1)[0])
        bridge = function_source(self.original_text, "void mipi_dsi_bridge_isr_handler", "// Please note, errors")
        # Only replace the underrun printing branch, keeping interrupt clearing,
        # VSYNC notification and yield behavior exactly as in the pinned driver.
        _, before, after = next(change for change in patch.REPLACEMENTS
                                if change[0] == "bounded_underrun_diagnostics")
        self.assertEqual(function_source(self.patched_text, "void mipi_dsi_bridge_isr_handler",
                                         "// Please note, errors"), bridge.replace(before, after))
        original_register = function_source(self.original_text, "esp_err_t esp_lcd_dpi_panel_register_event_callbacks")
        self.assertIn(original_register, self.patched_text)
        refresh = self.original_text.split("#if !MIPI_DSI_BRG_LL_EVENT_VSYNC\n", 1)[1].split("#endif", 1)[0]
        self.assertIn(refresh, self.patched_text)

    def test_actual_bridge_underrun_counter_snapshot_and_vsync(self) -> None:
        compiler = shutil.which("cc")
        if not compiler:
            self.skipTest("host C compiler unavailable")
        bridge = function_source(self.patched_text, "void mipi_dsi_bridge_isr_handler",
                                 "// Please note, errors")
        query = function_source(self.patched_text, "esp_err_t p4desk_lcd_underrun_stats(",
                                "esp_err_t p4desk_lcd_cache_stats(")
        self.assertNotIn("ESP_DRAM_LOG", bridge)
        self.assertNotIn("heap_caps_", bridge)
        self.assertNotIn("panel_reset", bridge)
        fixture = r'''
#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
typedef int esp_err_t;
#define ESP_OK 0
#define ESP_ERR_INVALID_ARG 1
#define ESP_ERR_INVALID_STATE 2
#define MIPI_DSI_BRG_LL_EVENT_UNDERRUN 1
#define MIPI_DSI_BRG_LL_EVENT_VSYNC 2
enum { P4DESK_SCAN_RUNNING, P4DESK_SCAN_REQUESTED, P4DESK_SCAN_STOPPING, P4DESK_SCAN_STOPPED };
#define ESP_RETURN_ON_FALSE(condition, code, ...) do { if (!(condition)) return (code); } while(0)
#define __containerof(pointer, type, field) ((type *)((char *)(pointer) - offsetof(type, field)))
typedef struct { int (*draw_bitmap)(void), (*draw_bitmap_2d)(void); } esp_lcd_panel_t;
typedef esp_lcd_panel_t *esp_lcd_panel_handle_t;
typedef struct { uint32_t count; int64_t first_us, last_us; } p4desk_lcd_underrun_stats_t;
typedef struct { int bridge; } mipi_dsi_hal_context_t;
typedef struct { mipi_dsi_hal_context_t hal; } fake_bus_t;
typedef struct {
    esp_lcd_panel_t base;
    fake_bus_t *bus;
    int frame_observer_lock;
    uint8_t scan_stop_state;
    p4desk_lcd_underrun_stats_t underrun_stats;
    bool (*on_refresh_done)(esp_lcd_panel_t *, void *, void *);
    void *user_ctx;
} esp_lcd_dpi_panel_t;
static int lock_depth, cleared, refreshed, yielded;
static uint32_t interrupt_status;
static int64_t now_us;
static bool in_isr, refresh_wakes;
#define portENTER_CRITICAL_ISR(lock) do { (void)(lock); assert(in_isr && lock_depth == 0); ++lock_depth; } while(0)
#define portEXIT_CRITICAL_ISR(lock) do { (void)(lock); assert(in_isr && lock_depth == 1); --lock_depth; } while(0)
#define portENTER_CRITICAL(lock) do { (void)(lock); assert(!in_isr && lock_depth == 0); ++lock_depth; } while(0)
#define portEXIT_CRITICAL(lock) do { (void)(lock); assert(!in_isr && lock_depth == 1); --lock_depth; } while(0)
#define portYIELD_FROM_ISR() do { assert(in_isr && lock_depth == 0); ++yielded; } while(0)
static int64_t esp_timer_get_time(void) { assert(in_isr && lock_depth == 0); return now_us; }
static bool xPortInIsrContext(void) { return in_isr; }
static uint32_t mipi_dsi_brg_ll_get_interrupt_status(int bridge)
{ assert(bridge == 7); return interrupt_status; }
static void mipi_dsi_brg_ll_clear_interrupt_status(int bridge, uint32_t status)
{ assert(bridge == 7 && status == interrupt_status); ++cleared; }
static int dpi_panel_draw_bitmap(void) { return 0; }
static int dpi_panel_draw_bitmap_2d(void) { return 0; }
static int wrong_method(void) { return 0; }
static bool refresh(esp_lcd_panel_t *panel, void *event, void *context)
{ assert(panel && !event && context == &refreshed && lock_depth == 0); ++refreshed; return refresh_wakes; }
'''
        checks = r'''
int main(void)
{
    fake_bus_t bus = {.hal = {.bridge = 7}};
    esp_lcd_dpi_panel_t panel = {
        .base = {.draw_bitmap = dpi_panel_draw_bitmap, .draw_bitmap_2d = dpi_panel_draw_bitmap_2d},
        .bus = &bus, .on_refresh_done = refresh, .user_ctx = &refreshed,
    };
    p4desk_lcd_underrun_stats_t stats = {.count = 99, .first_us = 99, .last_us = 99};
    assert(p4desk_lcd_underrun_stats(&panel.base, &stats) == ESP_OK);
    assert(stats.count == 0 && stats.first_us == 0 && stats.last_us == 0);
    in_isr = true;
    mipi_dsi_bridge_isr_handler(&panel);
    assert(panel.underrun_stats.count == 0 && cleared == 1 && refreshed == 0);
    interrupt_status = MIPI_DSI_BRG_LL_EVENT_UNDERRUN;
    now_us = 0; mipi_dsi_bridge_isr_handler(&panel);
    assert(panel.underrun_stats.count == 1 && panel.underrun_stats.first_us == 0);
    now_us = 100; mipi_dsi_bridge_isr_handler(&panel);
    assert(panel.underrun_stats.count == 2 && panel.underrun_stats.last_us == 100);
    interrupt_status |= MIPI_DSI_BRG_LL_EVENT_VSYNC;
    refresh_wakes = true; now_us = 200; mipi_dsi_bridge_isr_handler(&panel);
    assert(panel.underrun_stats.count == 3 && refreshed == 1 && yielded == 1);
    interrupt_status = MIPI_DSI_BRG_LL_EVENT_VSYNC;
    refresh_wakes = false; mipi_dsi_bridge_isr_handler(&panel);
    assert(panel.underrun_stats.count == 3 && refreshed == 2 && yielded == 1);
    panel.underrun_stats.count = UINT32_MAX - 1;
    interrupt_status = MIPI_DSI_BRG_LL_EVENT_UNDERRUN;
    now_us = 300; mipi_dsi_bridge_isr_handler(&panel);
    now_us = 400; mipi_dsi_bridge_isr_handler(&panel);
    assert(panel.underrun_stats.count == UINT32_MAX && panel.underrun_stats.first_us == 0);
    assert(panel.underrun_stats.last_us == 400 && lock_depth == 0);
    panel.scan_stop_state = P4DESK_SCAN_STOPPING;
    now_us = 500; mipi_dsi_bridge_isr_handler(&panel);
    assert(panel.underrun_stats.last_us == 400); // intentional stopped stream
    panel.scan_stop_state = P4DESK_SCAN_RUNNING;
    assert(p4desk_lcd_underrun_stats(&panel.base, &stats) == ESP_ERR_INVALID_STATE);
    assert(stats.count == 0 && stats.first_us == 0 && stats.last_us == 0);
    in_isr = false;
    assert(p4desk_lcd_underrun_stats(&panel.base, &stats) == ESP_OK);
    assert(stats.count == UINT32_MAX && stats.first_us == 0 && stats.last_us == 400);
    assert(p4desk_lcd_underrun_stats(NULL, &stats) == ESP_ERR_INVALID_ARG && stats.count == 0);
    assert(p4desk_lcd_underrun_stats(&panel.base, NULL) == ESP_ERR_INVALID_ARG);
    panel.base.draw_bitmap_2d = wrong_method;
    stats.count = 99;
    assert(p4desk_lcd_underrun_stats(&panel.base, &stats) == ESP_ERR_INVALID_ARG && stats.count == 0);
    assert(lock_depth == 0 && cleared == 8);
    return 0;
}
'''
        with tempfile.TemporaryDirectory(prefix="p4desk-dpi-underrun-") as directory:
            source = Path(directory) / "probe.c"
            executable = Path(directory) / "probe"
            source.write_text(fixture + bridge + "\n" + query + checks)
            result = subprocess.run([compiler, "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                                     str(source), "-o", str(executable)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            subprocess.run([str(executable)], check=True, capture_output=True, text=True)

    def test_existing_writeback_is_checked_before_buffer_publication(self) -> None:
        compiler = shutil.which("cc")
        if not compiler:
            self.skipTest("host C compiler unavailable")
        helper = function_source(self.patched_text, "static esp_err_t p4desk_dpi_sync_for_present(",
                                 "static bool dpi_panel_draw_bitmap_hook_end(")
        query = function_source(self.patched_text, "esp_err_t p4desk_lcd_cache_stats(",
                                "esp_err_t p4desk_lcd_frame_buffer_capacity(")
        start = self.patched_text.index("    if (!do_copy) { // no copy, just do cache memory write back")
        end = self.patched_text.index("    } else if (dpi_panel->draw_bitmap_hook)", start)
        direct = self.patched_text[start:end] + "    }\n    return ESP_OK;\n}\n"
        self.assertEqual(helper.count("esp_cache_msync("), 1)
        self.assertNotIn("esp_cache_msync(", direct)
        fixture = r'''
#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
typedef int esp_err_t;
#define ESP_OK 0
#define ESP_ERR_INVALID_ARG 1
#define ESP_ERR_INVALID_STATE 2
#define ESP_CACHE_MSYNC_FLAG_DIR_C2M 1
#define ESP_CACHE_MSYNC_FLAG_UNALIGNED 2
#define ESP_RETURN_ON_FALSE(condition, code, ...) do { if (!(condition)) return (code); } while(0)
#define ESP_RETURN_ON_ERROR(expression, ...) do { esp_err_t ret_ = (expression); if (ret_) return ret_; } while(0)
#define ESP_LOGV(...) do {} while(0)
#define __containerof(pointer, type, field) ((type *)((char *)(pointer) - offsetof(type, field)))
typedef struct { int (*draw_bitmap)(void), (*draw_bitmap_2d)(void); } esp_lcd_panel_t;
typedef esp_lcd_panel_t *esp_lcd_panel_handle_t;
typedef struct { uint32_t calls, errors, max_us; uint64_t total_us; int32_t last_error; } p4desk_lcd_cache_stats_t;
typedef struct {
    esp_lcd_panel_t base;
    int frame_observer_lock;
    p4desk_lcd_cache_stats_t cache_stats;
    uint8_t cur_fb_index;
    uint8_t *fbs[3];
    unsigned h_pixels;
    bool (*on_color_trans_done)(esp_lcd_panel_t *, void *, void *);
    void *user_ctx;
} esp_lcd_dpi_panel_t;
static bool in_isr;
static int lock_depth, synced, notified;
static esp_err_t sync_result;
static int64_t clock_us, duration_us;
static void *synced_address;
static size_t synced_bytes;
static bool xPortInIsrContext(void) { return in_isr; }
#define portENTER_CRITICAL(lock) do { (void)(lock); assert(!in_isr && lock_depth == 0); ++lock_depth; } while(0)
#define portEXIT_CRITICAL(lock) do { (void)(lock); assert(!in_isr && lock_depth == 1); --lock_depth; } while(0)
static int64_t esp_timer_get_time(void) { assert(lock_depth == 0); return clock_us; }
static esp_err_t esp_cache_msync(void *address, size_t bytes, int flags)
{
    assert(lock_depth == 0 && flags == (ESP_CACHE_MSYNC_FLAG_DIR_C2M | ESP_CACHE_MSYNC_FLAG_UNALIGNED));
    ++synced; synced_address = address; synced_bytes = bytes; clock_us += duration_us;
    return sync_result;
}
static int dpi_panel_draw_bitmap(void) { return 0; }
static int dpi_panel_draw_bitmap_2d(void) { return 0; }
static int wrong_method(void) { return 0; }
static bool notify(esp_lcd_panel_t *panel, void *event, void *context)
{ assert(panel && !event && !context && lock_depth == 0); ++notified; return false; }
'''
        prefix = r'''
static esp_err_t draw_direct(esp_lcd_dpi_panel_t *dpi_panel, uint8_t index, int y_start, int y_end)
{
    bool do_copy = false;
    uint8_t draw_buf_fb_index = index;
    size_t bits_per_pixel = 16;
    (void)&dpi_panel->base;
'''
        checks = r'''
int main(void)
{
    uint8_t storage[3][8192];
    esp_lcd_dpi_panel_t panel = {
        .base = {.draw_bitmap = dpi_panel_draw_bitmap, .draw_bitmap_2d = dpi_panel_draw_bitmap_2d},
        .fbs = {storage[0], storage[1], storage[2]}, .h_pixels = 1024,
        .on_color_trans_done = notify,
    };
    duration_us = 43;
    assert(draw_direct(&panel, 1, 0, 3) == ESP_OK);
    assert(synced == 1 && notified == 1 && panel.cur_fb_index == 1);
    assert(synced_address == storage[1] && synced_bytes == 6144);
    p4desk_lcd_cache_stats_t stats;
    assert(p4desk_lcd_cache_stats(&panel.base, &stats) == ESP_OK);
    assert(stats.calls == 1 && stats.errors == 0 && stats.max_us == 43 && stats.total_us == 43);
    sync_result = 17; duration_us = 71;
    assert(draw_direct(&panel, 2, 1, 3) == 17);
    assert(synced == 2 && notified == 1 && panel.cur_fb_index == 1);
    assert(synced_address == storage[2] + 2048 && synced_bytes == 4096);
    assert(p4desk_lcd_cache_stats(&panel.base, &stats) == ESP_OK);
    assert(stats.calls == 2 && stats.errors == 1 && stats.max_us == 71 && stats.total_us == 114 && stats.last_error == 17);
    panel.cache_stats.calls = UINT32_MAX; panel.cache_stats.errors = UINT32_MAX;
    panel.cache_stats.total_us = UINT64_MAX - 1;
    duration_us = (int64_t)UINT32_MAX + 1;
    assert(draw_direct(&panel, 2, 0, 3) == 17);
    assert(panel.cur_fb_index == 1 && notified == 1 && synced == 3);
    assert(p4desk_lcd_cache_stats(&panel.base, &stats) == ESP_OK);
    assert(stats.calls == UINT32_MAX && stats.errors == UINT32_MAX);
    assert(stats.max_us == UINT32_MAX && stats.total_us == UINT64_MAX);
    in_isr = true;
    assert(p4desk_lcd_cache_stats(&panel.base, &stats) == ESP_ERR_INVALID_STATE && stats.calls == 0);
    in_isr = false;
    assert(p4desk_lcd_cache_stats(NULL, &stats) == ESP_ERR_INVALID_ARG && stats.calls == 0);
    assert(p4desk_lcd_cache_stats(&panel.base, NULL) == ESP_ERR_INVALID_ARG);
    panel.base.draw_bitmap_2d = wrong_method;
    assert(p4desk_lcd_cache_stats(&panel.base, &stats) == ESP_ERR_INVALID_ARG && stats.calls == 0);
    assert(lock_depth == 0);
    return 0;
}
'''
        with tempfile.TemporaryDirectory(prefix="p4desk-dpi-cache-") as directory:
            source = Path(directory) / "probe.c"
            executable = Path(directory) / "probe"
            source.write_text(fixture + helper + query + prefix + direct + checks)
            result = subprocess.run([compiler, "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                                     str(source), "-o", str(executable)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            subprocess.run([str(executable)], check=True, capture_output=True, text=True)

    def test_padding_preserves_original_scan_and_visible_geometry(self) -> None:
        original_geometry = (
            "    dpi_panel->fb_size = fb_size;\n"
            "    dpi_panel->bits_per_pixel = bits_per_pixel;\n"
            "    dpi_panel->h_pixels = panel_config->video_timing.h_size;\n"
            "    dpi_panel->v_pixels = panel_config->video_timing.v_size;\n"
        )
        self.assertIn(original_geometry, self.patched_text)
        self.assertIn("        .size = dpi_panel->fb_size * 8 / 64,\n", self.patched_text)
        cache_geometry = (
            "        uint8_t *cache_sync_start = dpi_panel->fbs[draw_buf_fb_index] + (y_start * dpi_panel->h_pixels) * bits_per_pixel / 8;\n"
            "        size_t cache_sync_size = (y_end - y_start) * dpi_panel->h_pixels * bits_per_pixel / 8;\n"
        )
        self.assertIn(cache_geometry, self.patched_text)
        self.assertIn("    size_t fb_size = dpi_panel->fb_size;\n", self.patched_text)
        self.assertEqual(self.patched_text.count("dpi_panel->fb_capacity = fb_capacity;"), 1)

    def test_host_error_poll_has_one_owner_and_retains_reports(self) -> None:
        compiler = shutil.which("cc")
        if not compiler:
            self.skipTest("host C compiler unavailable")
        poll = function_source(self.patched_text, "esp_err_t p4desk_lcd_host_errors_poll(",
                               "esp_err_t p4desk_lcd_host_error_stats(")
        query = function_source(self.patched_text, "esp_err_t p4desk_lcd_host_error_stats(",
                                "esp_err_t p4desk_lcd_underrun_stats(")
        self.assertEqual(poll.count("host->int_st0.val"), 1)
        self.assertEqual(poll.count("host->int_st1.val"), 1)
        self.assertNotIn("host->", query)
        self.assertNotIn("ESP_LOG", poll)
        self.assertNotIn("heap_caps_", poll)
        self.assertNotIn("reset(", poll)
        fixture = r'''
#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
typedef int esp_err_t;
typedef void *TaskHandle_t;
#define ESP_OK 0
#define ESP_ERR_INVALID_ARG 1
#define ESP_ERR_INVALID_STATE 2
#define P4DESK_DSI_HOST_DPI_FIFO_OVERFLOW (UINT32_C(1) << 7)
#define P4DESK_DSI_HOST_DPI_FIFO_UNDERFLOW (UINT32_C(1) << 19)
#define ESP_RETURN_ON_FALSE(condition, code, ...) do { if (!(condition)) return (code); } while(0)
#define __containerof(pointer, type, field) ((type *)((char *)(pointer) - offsetof(type, field)))
typedef struct { int (*draw_bitmap)(void), (*draw_bitmap_2d)(void); } esp_lcd_panel_t;
typedef esp_lcd_panel_t *esp_lcd_panel_handle_t;
typedef struct {
    uint32_t polls, error_polls, status0_or, status1_or, dpi_overflow_polls, dpi_underflow_polls;
    uint32_t last_status0, last_status1;
    int64_t first_error_us, last_error_us;
} p4desk_lcd_host_error_stats_t;
typedef struct { struct { volatile uint32_t val; } int_st0, int_st1; } fake_host_t;
typedef struct { struct { fake_host_t *host; } hal; } fake_bus_t;
typedef struct {
    esp_lcd_panel_t base;
    int frame_observer_lock;
    fake_bus_t *bus;
    p4desk_lcd_host_error_stats_t host_error_stats;
    TaskHandle_t host_error_owner;
} esp_lcd_dpi_panel_t;
static bool in_isr;
static int lock_depth, first_task, second_task;
static TaskHandle_t caller;
static int64_t now_us;
static bool xPortInIsrContext(void) { return in_isr; }
static TaskHandle_t xTaskGetCurrentTaskHandle(void) { assert(!in_isr); return caller; }
#define portENTER_CRITICAL(lock) do { (void)(lock); assert(!in_isr && lock_depth == 0); ++lock_depth; } while(0)
#define portEXIT_CRITICAL(lock) do { (void)(lock); assert(!in_isr && lock_depth == 1); --lock_depth; } while(0)
static int64_t esp_timer_get_time(void) { assert(lock_depth == 0); return now_us; }
static int dpi_panel_draw_bitmap(void) { return 0; }
static int dpi_panel_draw_bitmap_2d(void) { return 0; }
static int wrong_method(void) { return 0; }
'''
        checks = r'''
int main(void)
{
    fake_host_t host = {0};
    fake_bus_t bus = {.hal = {.host = &host}};
    esp_lcd_dpi_panel_t panel = {
        .base = {.draw_bitmap = dpi_panel_draw_bitmap, .draw_bitmap_2d = dpi_panel_draw_bitmap_2d},
        .bus = &bus,
    };
    caller = &first_task;
    assert(p4desk_lcd_host_errors_poll(&panel.base) == ESP_OK);
    assert(panel.host_error_owner == caller && panel.host_error_stats.polls == 1);
    assert(panel.host_error_stats.error_polls == 0);
    host.int_st0.val = 4; host.int_st1.val = P4DESK_DSI_HOST_DPI_FIFO_UNDERFLOW;
    now_us = 100;
    assert(p4desk_lcd_host_errors_poll(&panel.base) == ESP_OK);
    // Hardware read-clear behavior is specified by TRM; emulate clearing between
    // polls here, while structural checks above ensure one production read each.
    host.int_st0.val = host.int_st1.val = 0;
    now_us = 150;
    assert(p4desk_lcd_host_errors_poll(&panel.base) == ESP_OK);
    p4desk_lcd_host_error_stats_t stats;
    caller = &second_task;
    assert(p4desk_lcd_host_error_stats(&panel.base, &stats) == ESP_OK);
    assert(stats.polls == 3 && stats.error_polls == 1 && stats.status0_or == 4);
    assert(stats.dpi_underflow_polls == 1 && stats.last_status0 == 0 && stats.last_status1 == 0);
    assert(stats.first_error_us == 100 && stats.last_error_us == 100);
    host.int_st0.val = 32; host.int_st1.val = P4DESK_DSI_HOST_DPI_FIFO_OVERFLOW;
    assert(p4desk_lcd_host_errors_poll(&panel.base) == ESP_ERR_INVALID_STATE);
    assert(panel.host_error_stats.polls == 3 && host.int_st0.val == 32 && host.int_st1.val == 128);
    caller = &first_task; now_us = 200;
    assert(p4desk_lcd_host_errors_poll(&panel.base) == ESP_OK);
    assert(p4desk_lcd_host_error_stats(&panel.base, &stats) == ESP_OK);
    assert(stats.error_polls == 2 && stats.status0_or == 36);
    assert(stats.status1_or == (P4DESK_DSI_HOST_DPI_FIFO_OVERFLOW | P4DESK_DSI_HOST_DPI_FIFO_UNDERFLOW));
    assert(stats.dpi_overflow_polls == 1 && stats.first_error_us == 100 && stats.last_error_us == 200);
    panel.host_error_stats.polls = panel.host_error_stats.error_polls = UINT32_MAX;
    panel.host_error_stats.dpi_overflow_polls = panel.host_error_stats.dpi_underflow_polls = UINT32_MAX;
    host.int_st1.val |= P4DESK_DSI_HOST_DPI_FIFO_UNDERFLOW; now_us = 300;
    assert(p4desk_lcd_host_errors_poll(&panel.base) == ESP_OK);
    assert(p4desk_lcd_host_error_stats(&panel.base, &stats) == ESP_OK);
    assert(stats.polls == UINT32_MAX && stats.error_polls == UINT32_MAX && stats.dpi_overflow_polls == UINT32_MAX);
    assert(stats.dpi_underflow_polls == UINT32_MAX && stats.last_error_us == 300);
    in_isr = true;
    assert(p4desk_lcd_host_errors_poll(&panel.base) == ESP_ERR_INVALID_STATE);
    assert(p4desk_lcd_host_error_stats(&panel.base, &stats) == ESP_ERR_INVALID_STATE && stats.polls == 0);
    in_isr = false;
    assert(p4desk_lcd_host_errors_poll(NULL) == ESP_ERR_INVALID_ARG);
    assert(p4desk_lcd_host_error_stats(NULL, &stats) == ESP_ERR_INVALID_ARG && stats.polls == 0);
    assert(p4desk_lcd_host_error_stats(&panel.base, NULL) == ESP_ERR_INVALID_ARG);
    panel.base.draw_bitmap_2d = wrong_method;
    assert(p4desk_lcd_host_errors_poll(&panel.base) == ESP_ERR_INVALID_ARG);
    assert(p4desk_lcd_host_error_stats(&panel.base, &stats) == ESP_ERR_INVALID_ARG && stats.polls == 0);
    assert(lock_depth == 0);
    return 0;
}
'''
        with tempfile.TemporaryDirectory(prefix="p4desk-dpi-host-") as directory:
            source = Path(directory) / "probe.c"
            executable = Path(directory) / "probe"
            source.write_text(fixture + poll + query + checks)
            result = subprocess.run([compiler, "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                                     str(source), "-o", str(executable)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            subprocess.run([str(executable)], check=True, capture_output=True, text=True)

    def test_underrun_line_mask_and_isr_safety_requirements(self) -> None:
        self.assertIn("mipi_dsi_brg_ll_set_underrun_discard_count(hal->bridge, 0);", self.patched_text)
        self.assertNotIn("mipi_dsi_brg_ll_set_underrun_discard_count(hal->bridge, panel_config->video_timing.h_size);",
                         self.patched_text)
        callback = function_source(self.patched_text, "bool mipi_dsi_dma_trans_done_cb(",
                                   "void mipi_dsi_bridge_isr_handler(")
        bridge = function_source(self.patched_text, "void mipi_dsi_bridge_isr_handler(",
                                 "// Please note, errors")
        self.assertNotIn("host->int_st", callback + bridge)
        registration = function_source(self.patched_text, "esp_err_t p4desk_lcd_frame_observer_register(",
                                       "esp_err_t p4desk_lcd_host_errors_poll(")
        self.assertIn("#if CONFIG_LCD_DSI_ISR_CACHE_SAFE", registration)
        self.assertIn("esp_ptr_in_iram(callback)", registration)
        self.assertIn("esp_ptr_internal(context)", registration)
        cmake = (COMPONENT / "CMakeLists.txt").read_text()
        self.assertIn("CONFIG_LCD_DSI_OBJ_FORCE_INTERNAL", cmake)
        linker = (SDK_SOURCE.parents[1] / "linker.lf").read_text()
        self.assertIn("esp_lcd_panel_dpi: mipi_dsi_dma_trans_done_cb (noflash)", linker)
        self.assertIn("esp_lcd_panel_dpi: mipi_dsi_bridge_isr_handler (noflash)", linker)

    def test_actual_capacity_calculation_and_exact_pointer_query(self) -> None:
        compiler = shutil.which("cc")
        if not compiler:
            self.skipTest("host C compiler unavailable")
        start = self.patched_text.index("    // allocate frame buffer from PSRAM\n")
        end = self.patched_text.index("    for (int i = 0; i < num_fbs; i++)", start)
        allocation = self.patched_text[start:end]
        query = function_source(self.patched_text, "esp_err_t p4desk_lcd_frame_buffer_capacity(")
        fixture = r'''
#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
typedef int esp_err_t;
#define ESP_OK 0
#define ESP_ERR_INVALID_ARG 1
#define ESP_ERR_INVALID_STATE 2
#define MALLOC_CAP_SPIRAM 1
#define MALLOC_CAP_8BIT 2
#define MALLOC_CAP_DMA 4
#define ESP_GOTO_ON_ERROR(result, label, ...) do { ret = (result); if (ret != ESP_OK) goto label; } while(0)
#define ESP_GOTO_ON_FALSE(condition, code, label, ...) do { if (!(condition)) { ret = (code); goto label; } } while(0)
#define ESP_RETURN_ON_FALSE(condition, code, ...) do { if (!(condition)) return (code); } while(0)
#define __containerof(pointer, type, field) ((type *)((char *)(pointer) - offsetof(type, field)))
typedef struct {
    int (*draw_bitmap)(void);
    int (*draw_bitmap_2d)(void);
    int (*del)(void);
} esp_lcd_panel_t;
typedef esp_lcd_panel_t *esp_lcd_panel_handle_t;
typedef struct {
    esp_lcd_panel_t base;
    size_t fb_capacity;
    uint8_t num_fbs;
    uint8_t *fbs[3];
} esp_lcd_dpi_panel_t;
typedef struct { struct { uint32_t h_size, v_size; } video_timing; } panel_config_t;
static size_t cache_alignment = 64;
static bool in_isr;
static int dpi_panel_draw_bitmap(void) { return 0; }
static int dpi_panel_draw_bitmap_2d(void) { return 0; }
static int ek79007_del(void) { return 0; }
static bool xPortInIsrContext(void) { return in_isr; }
static int esp_cache_get_alignment(unsigned caps, size_t *out)
{ assert(caps == (MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT | MALLOC_CAP_DMA)); *out = cache_alignment; return ESP_OK; }
static esp_err_t calculate(esp_lcd_dpi_panel_t *dpi_panel, const panel_config_t *panel_config,
                           size_t bits_per_pixel, size_t *visible)
{
    esp_err_t ret = ESP_OK;
'''
        suffix = r'''
    *visible = fb_size;
    return ESP_OK;
err:
    return ret;
}
'''
        checks = r'''
int main(void)
{
    uint8_t storage[3][8] = {{0}}, unrelated = 0;
    esp_lcd_dpi_panel_t panel = {
        .base = {.draw_bitmap = dpi_panel_draw_bitmap, .draw_bitmap_2d = dpi_panel_draw_bitmap_2d,
                 .del = ek79007_del},
        .num_fbs = 3, .fbs = {storage[0], storage[1], storage[2]},
    };
    panel_config_t config = {.video_timing = {.h_size = 1024, .v_size = 600}};
    size_t visible = 0, capacity = 99;
    assert(calculate(&panel, &config, 16, &visible) == ESP_OK);
    assert(visible == 1228800 && panel.fb_capacity == 1245184);
    assert((panel.fb_capacity - visible) * 3 == 49152);
    for (unsigned index = 0; index < 3; ++index) {
        assert(p4desk_lcd_frame_buffer_capacity(&panel.base, panel.fbs[index], &capacity) == ESP_OK);
        assert(capacity == 1245184);
    }
    assert(p4desk_lcd_frame_buffer_capacity(&panel.base, storage[1] + 1, &capacity) == ESP_ERR_INVALID_ARG);
    assert(capacity == 0);
    capacity = 99;
    assert(p4desk_lcd_frame_buffer_capacity(&panel.base, &unrelated, &capacity) == ESP_ERR_INVALID_ARG && capacity == 0);
    assert(p4desk_lcd_frame_buffer_capacity(NULL, storage[0], &capacity) == ESP_ERR_INVALID_ARG && capacity == 0);
    assert(p4desk_lcd_frame_buffer_capacity(&panel.base, NULL, &capacity) == ESP_ERR_INVALID_ARG && capacity == 0);
    assert(p4desk_lcd_frame_buffer_capacity(&panel.base, storage[0], NULL) == ESP_ERR_INVALID_ARG);
    in_isr = true; capacity = 99;
    assert(p4desk_lcd_frame_buffer_capacity(&panel.base, storage[0], &capacity) == ESP_ERR_INVALID_STATE && capacity == 0);
    in_isr = false;
    panel.base.draw_bitmap = ek79007_del;
    assert(p4desk_lcd_frame_buffer_capacity(&panel.base, storage[0], &capacity) == ESP_ERR_INVALID_ARG && capacity == 0);
    config.video_timing.v_size = 608;
    assert(calculate(&panel, &config, 16, &visible) == ESP_OK && visible == 1245184 && panel.fb_capacity == 1245184);
    config.video_timing.v_size = 609;
    assert(calculate(&panel, &config, 16, &visible) == ESP_OK && visible == 1247232 && panel.fb_capacity == 1277952);
    config.video_timing.h_size = config.video_timing.v_size = 1; cache_alignment = 128;
    assert(calculate(&panel, &config, 16, &visible) == ESP_OK && visible == 2 && panel.fb_capacity == 128);
    cache_alignment = 3;
    assert(calculate(&panel, &config, 16, &visible) == ESP_ERR_INVALID_STATE);
    cache_alignment = 64; config.video_timing.h_size = config.video_timing.v_size = UINT32_MAX;
    assert(calculate(&panel, &config, UINT32_MAX, &visible) == ESP_ERR_INVALID_ARG);
    return 0;
}
'''
        with tempfile.TemporaryDirectory(prefix="p4desk-dpi-capacity-") as directory:
            source = Path(directory) / "probe.c"
            executable = Path(directory) / "probe"
            source.write_text(fixture + allocation + suffix + query + checks)
            result = subprocess.run([compiler, "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                                     str(source), "-o", str(executable)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            subprocess.run([str(executable)], check=True, capture_output=True, text=True)

    def test_generator_is_repeatable_and_refuses_sdk_overwrite(self) -> None:
        with tempfile.TemporaryDirectory(prefix="p4desk-dpi-generator-") as directory:
            output = Path(directory) / "esp_lcd_panel_dpi.c"
            manifest = Path(directory) / "patch-manifest.json"
            command = [sys.executable, str(TOOL), "--input", str(SDK_SOURCE), "--output", str(output),
                       "--manifest", str(manifest)]
            subprocess.run(command, check=True, capture_output=True, text=True)
            self.assertEqual(output.read_bytes(), self.patched)
            timestamp = output.stat().st_mtime_ns
            subprocess.run(command, check=True, capture_output=True, text=True)
            self.assertEqual(output.stat().st_mtime_ns, timestamp)
            provenance = json.loads(manifest.read_text())
            self.assertEqual(provenance["source_sha256"], patch.PINNED_SHA256)
            self.assertEqual(provenance["generated_sha256"], hashlib.sha256(self.patched).hexdigest())
            result = subprocess.run([sys.executable, str(TOOL), "--input", str(SDK_SOURCE), "--output",
                                     str(SDK_SOURCE)], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("must not overwrite", result.stderr)
            self.assertEqual(SDK_SOURCE.read_bytes(), self.source)

    def test_actual_dma_callback_tracks_selected_source_and_preserves_yield(self) -> None:
        compiler = shutil.which("cc")
        if not compiler:
            self.skipTest("host C compiler unavailable")
        callback = function_source(self.patched_text, "bool mipi_dsi_dma_trans_done_cb(",
                                   "void mipi_dsi_bridge_isr_handler")
        fixture = r'''
#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
typedef struct { int unused; } esp_lcd_panel_t;
typedef void *dw_gdma_channel_handle_t;
typedef void *dw_gdma_link_list_handle_t;
typedef struct { int unused; } dw_gdma_trans_done_event_data_t;
typedef struct { bool is_valid, is_last; } dw_gdma_block_markers_t;
enum { P4DESK_SCAN_RUNNING, P4DESK_SCAN_REQUESTED, P4DESK_SCAN_STOPPING, P4DESK_SCAN_STOPPED };
typedef struct {
    const void *completed_fb, *next_fb;
    uint32_t counter;
    uint8_t completed_index, next_index;
} p4desk_lcd_frame_event_t;
typedef bool (*p4desk_lcd_frame_observer_cb_t)(esp_lcd_panel_t *, const p4desk_lcd_frame_event_t *, void *);
typedef struct {
    esp_lcd_panel_t base;
    int frame_observer_lock;
    uint8_t scanning_fb_index, cur_fb_index;
    uint8_t scan_stop_state;
    uint32_t frame_observer_counter;
    p4desk_lcd_frame_observer_cb_t frame_observer_callback;
    void *frame_observer_context;
    uint8_t *fbs[3];
    dw_gdma_link_list_handle_t link_lists[3];
    bool (*on_refresh_done)(esp_lcd_panel_t *, void *, void *);
    void *user_ctx;
} esp_lcd_dpi_panel_t;
static int lock_depth, restarts, refreshes, observed, context_value;
static bool observer_wakes, refresh_wakes, publish_during_restart;
static void *selected_list;
static p4desk_lcd_frame_event_t last;
static esp_lcd_dpi_panel_t *running_panel;
#define portENTER_CRITICAL_ISR(lock) do { (void)(lock); assert(lock_depth == 0); ++lock_depth; } while(0)
#define portEXIT_CRITICAL_ISR(lock) do { (void)(lock); assert(lock_depth == 1); --lock_depth; } while(0)
static void *dw_gdma_link_list_get_item(void *list, int index) { assert(index == 0); return list; }
static void dw_gdma_lli_set_block_markers(void *item, dw_gdma_block_markers_t markers)
{ assert(item && markers.is_valid && markers.is_last); assert(lock_depth == 0); }
static void dw_gdma_channel_use_link_list(void *channel, void *list)
{
    (void)channel; selected_list = list;
    if (publish_during_restart) running_panel->cur_fb_index = 0;
}
static void dw_gdma_channel_enable_ctrl(void *channel, bool enabled)
{ (void)channel; assert(enabled); ++restarts; }
static bool observer(esp_lcd_panel_t *panel, const p4desk_lcd_frame_event_t *event, void *context)
{
    assert(panel == &running_panel->base && context == &context_value);
    assert(lock_depth == 0);
    if (running_panel->scan_stop_state != P4DESK_SCAN_STOPPING)
        assert(selected_list == running_panel->link_lists[event->next_index]);
    assert(event->completed_fb == running_panel->fbs[event->completed_index]);
    assert(event->next_fb == running_panel->fbs[event->next_index]);
    last = *event; ++observed;
    return observer_wakes;
}
static bool refresh(esp_lcd_panel_t *panel, void *event, void *context)
{ (void)panel; (void)event; (void)context; ++refreshes; return refresh_wakes; }
'''
        checks = r'''
int main(void)
{
    uint8_t storage[3];
    esp_lcd_dpi_panel_t panel = {
        .scanning_fb_index = 0, .cur_fb_index = 0,
        .frame_observer_callback = observer, .frame_observer_context = &context_value,
        .fbs = {&storage[0], &storage[1], &storage[2]},
        .link_lists = {&storage[0], &storage[1], &storage[2]}, .on_refresh_done = refresh,
    };
    running_panel = &panel;
    assert(!mipi_dsi_dma_trans_done_cb(NULL, NULL, &panel));
    assert(last.completed_index == 0 && last.next_index == 0 && last.counter == 1);
    observer_wakes = true; panel.cur_fb_index = 1;
    assert(mipi_dsi_dma_trans_done_cb(NULL, NULL, &panel));
    assert(last.completed_index == 0 && last.next_index == 1 && last.counter == 2);
    observer_wakes = false; panel.cur_fb_index = 2;
    assert(!mipi_dsi_dma_trans_done_cb(NULL, NULL, &panel));
    assert(last.completed_index == 1 && last.next_index == 2 && last.counter == 3);
    panel.cur_fb_index = 1; publish_during_restart = true;
    assert(!mipi_dsi_dma_trans_done_cb(NULL, NULL, &panel));
    assert(last.completed_index == 2 && last.next_index == 1 && panel.cur_fb_index == 0);
    publish_during_restart = false;
    panel.frame_observer_counter = UINT32_MAX;
    assert(!mipi_dsi_dma_trans_done_cb(NULL, NULL, &panel));
    assert(last.completed_index == 1 && last.next_index == 0 && last.counter == 0);
    panel.frame_observer_callback = NULL;
    refresh_wakes = true;
    bool result = mipi_dsi_dma_trans_done_cb(NULL, NULL, &panel);
#if !MIPI_DSI_BRG_LL_EVENT_VSYNC
    assert(result && refreshes == 6);
#else
    assert(!result && refreshes == 0);
#endif
    assert(lock_depth == 0 && observed == 5 && restarts == 6);
    // Pending source 2 must not be selected when stopping source 0. The real
    // completion increments the counter and emits A,A, keeping ownership A.
    panel.frame_observer_callback = observer;
    refresh_wakes = false;
    panel.cur_fb_index = 2;
    panel.scan_stop_state = P4DESK_SCAN_REQUESTED;
    assert(!mipi_dsi_dma_trans_done_cb(NULL, NULL, &panel));
    assert(panel.scan_stop_state == P4DESK_SCAN_STOPPED);
    assert(last.completed_index == 0 && last.next_index == 0 && last.counter == 2);
    assert(panel.scanning_fb_index == 0 && panel.cur_fb_index == 2 && restarts == 6);
    // Task restart uses A, retaining B; the next normal completion changes A->B.
    panel.scan_stop_state = P4DESK_SCAN_RUNNING;
    assert(!mipi_dsi_dma_trans_done_cb(NULL, NULL, &panel));
    assert(last.completed_index == 0 && last.next_index == 2 && last.counter == 3);
    assert(restarts == 7 && observed == 7);
    return 0;
}
'''
        with tempfile.TemporaryDirectory(prefix="p4desk-dpi-callback-") as directory:
            source = Path(directory) / "probe.c"
            executable = Path(directory) / "probe"
            source.write_text(fixture + "\n" + callback + "\n" + checks)
            for has_vsync in (0, 1):
                result = subprocess.run([compiler, "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                                         "-Wno-unused-parameter", f"-DMIPI_DSI_BRG_LL_EVENT_VSYNC={has_vsync}",
                                         str(source), "-o", str(executable)], capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                subprocess.run([str(executable)], check=True, capture_output=True, text=True)

    def test_board_diagnostic_logs_are_bounded_and_preserve_zero_and_delta(self) -> None:
        compiler = shutil.which("cc")
        if not compiler:
            self.skipTest("host C compiler unavailable")
        board_source = (COMPONENT.parent / "board_p4/board_p4.c").read_text()
        logger = function_source(board_source, "void board_p4_log_display_diagnostics(void)",
                                 "static esp_err_t backlight_init(void)")
        fixture = r'''
#include <assert.h>
#include <inttypes.h>
#include <stddef.h>
#include <stdint.h>
typedef int esp_err_t;
typedef void *esp_lcd_panel_handle_t;
typedef struct { uint32_t count; int64_t first_us, last_us; } p4desk_lcd_underrun_stats_t;
#define ESP_OK 0
static esp_lcd_panel_handle_t s_diagnostic_panel;
static int64_t s_display_log_us, now_us;
static uint32_t s_logged_underruns;
static unsigned queried, logged, warned;
static esp_err_t query_result;
static p4desk_lcd_underrun_stats_t available;
static uint32_t logged_count, logged_added;
static int64_t logged_first, logged_last, logged_age;
static int64_t esp_timer_get_time(void) { return now_us; }
static esp_err_t p4desk_lcd_underrun_stats(esp_lcd_panel_handle_t panel,
                                         p4desk_lcd_underrun_stats_t *out)
{ assert(panel == &available); ++queried; *out = available; return query_result; }
static void record_log(uint32_t count, uint32_t added, int64_t first, int64_t last, int64_t age)
{ ++logged; logged_count = count; logged_added = added; logged_first = first; logged_last = last; logged_age = age; }
#define ESP_LOGI(tag, format, ...) record_log(__VA_ARGS__)
#define ESP_LOGW(tag, format, ...) do { ++warned; } while(0)
'''
        checks = r'''
int main(void)
{
    now_us = 30000000; board_p4_log_display_diagnostics();
    assert(queried == 0 && logged == 0); // No initialized panel.
    s_diagnostic_panel = &available;
    now_us = 29999999; board_p4_log_display_diagnostics();
    assert(queried == 0);
    now_us = 30000000; board_p4_log_display_diagnostics();
    assert(queried == 1 && logged == 1 && logged_count == 0 && logged_added == 0);
    assert(logged_first == 0 && logged_last == 0 && logged_age == -1);
    now_us = 59999999; board_p4_log_display_diagnostics();
    assert(queried == 1 && logged == 1);
    available = (p4desk_lcd_underrun_stats_t){.count = 3, .first_us = 41000000, .last_us = 55000000};
    now_us = 60000000; board_p4_log_display_diagnostics();
    assert(queried == 2 && logged == 2 && logged_count == 3 && logged_added == 3);
    assert(logged_first == 41000000 && logged_last == 55000000 && logged_age == 5000);
    now_us = 90000000; board_p4_log_display_diagnostics();
    assert(logged_count == 3 && logged_added == 0 && logged_age == 35000);
    available = (p4desk_lcd_underrun_stats_t){.count = 1, .first_us = 110000000, .last_us = 110000000};
    now_us = 120000000; board_p4_log_display_diagnostics();
    assert(logged_count == 1 && logged_added == 1); // Counter reset never wraps delta.
    query_result = 1;
    now_us = 150000000; board_p4_log_display_diagnostics();
    now_us = 150000001; board_p4_log_display_diagnostics();
    assert(queried == 5 && logged == 4 && warned == 1);
    query_result = ESP_OK; available.count = 2; available.last_us = 175000000;
    now_us = 180000000; board_p4_log_display_diagnostics();
    assert(queried == 6 && logged == 5 && logged_added == 1 && logged_age == 5000);
    return 0;
}
'''
        with tempfile.TemporaryDirectory(prefix="p4desk-board-underrun-") as directory:
            source = Path(directory) / "probe.c"
            executable = Path(directory) / "probe"
            source.write_text(fixture + logger + checks)
            result = subprocess.run([compiler, "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                                     str(source), "-o", str(executable)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            subprocess.run([str(executable)], check=True, capture_output=True, text=True)

    def test_actual_resync_preserves_pending_source_and_bounds_cancel_races(self) -> None:
        compiler = shutil.which("cc")
        if not compiler:
            self.skipTest("host C compiler unavailable")
        callback = function_source(self.patched_text, "bool mipi_dsi_dma_trans_done_cb(",
                                   "void mipi_dsi_bridge_isr_handler")
        owner_check = function_source(self.patched_text, "static esp_err_t p4desk_scan_owner_check(",
                                      "esp_err_t p4desk_lcd_scan_timing(")
        resync = function_source(self.patched_text, "esp_err_t p4desk_lcd_scan_resync(",
                                 "// Project-local extension.")
        header = (COMPONENT / "include/p4desk_lcd_frame_observer.h").read_text()
        result_start = header.index("typedef struct {", header.index("/** Result of a bounded scan resynchronization"))
        result_end = header.index("} p4desk_lcd_resync_result_t;", result_start) + len("} p4desk_lcd_resync_result_t;")
        host_members = "\n".join(f"    reg_t {name};" for name in patch.HOST_RESYNC_REGS + ["int_st0", "int_st1", "pwr_up"])
        bridge_members = "\n".join(f"    reg_t {name};" for name in patch.BRIDGE_RESYNC_REGS + ["int_ena", "en"])
        fixture = r'''
#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>
typedef int esp_err_t;
#define ESP_OK 0
#define ESP_ERR_INVALID_ARG 1
#define ESP_ERR_INVALID_STATE 2
#define ESP_ERR_TIMEOUT 3
#define ESP_ERR_NOT_SUPPORTED 4
#define HAL_CONFIG(x) 300
#define MIPI_DSI_BRG_LL_EVENT_VSYNC 1
#define ESP_RETURN_ON_FALSE(c, e, ...) do { if (!(c)) return (e); } while(0)
#define ESP_RETURN_ON_ERROR(c, ...) do { int e = (c); if(e) return e; } while(0)
#define __containerof(p, t, f) ((t *)((char *)(p) - offsetof(t, f)))
enum { P4DESK_SCAN_RUNNING, P4DESK_SCAN_REQUESTED, P4DESK_SCAN_STOPPING, P4DESK_SCAN_STOPPED };
typedef union { uint32_t val, dpi_en, shutdownz; } reg_t;
@REG_TYPES@
typedef struct { fake_host_t *host; fake_bridge_t *bridge; } mipi_dsi_hal_context_t;
typedef struct { mipi_dsi_hal_context_t hal; } fake_bus_t;
typedef void *TaskHandle_t;
typedef struct { int (*draw_bitmap)(void), (*draw_bitmap_2d)(void); } esp_lcd_panel_t;
typedef esp_lcd_panel_t *esp_lcd_panel_handle_t;
typedef void *dw_gdma_channel_handle_t;
typedef void *dw_gdma_link_list_handle_t;
typedef struct { int unused; } dw_gdma_trans_done_event_data_t;
typedef struct { bool is_valid, is_last; } dw_gdma_block_markers_t;
typedef struct {
    const void *completed_fb, *next_fb; uint32_t counter;
    uint8_t completed_index, next_index;
} p4desk_lcd_frame_event_t;
typedef bool (*p4desk_lcd_frame_observer_cb_t)(esp_lcd_panel_t *, const p4desk_lcd_frame_event_t *, void *);
@RESULT_TYPE@
typedef struct {
    esp_lcd_panel_t base; fake_bus_t *bus;
    int frame_observer_lock;
    uint8_t scanning_fb_index, cur_fb_index, scan_stop_state;
    uint32_t frame_observer_counter;
    TaskHandle_t host_error_owner;
    p4desk_lcd_frame_observer_cb_t frame_observer_callback;
    void *frame_observer_context;
    uint8_t *fbs[3];
    dw_gdma_link_list_handle_t link_lists[3];
    dw_gdma_channel_handle_t dma_chan;
} esp_lcd_dpi_panel_t;
static esp_lcd_dpi_panel_t *running_panel;
static int lock_depth, restarts, observed, power_downs, power_ups, bridge_resets, context;
static bool in_isr;
static int64_t now_us;
static void *selected_list;
static p4desk_lcd_frame_event_t last;
static TaskHandle_t caller = (void *)1;
enum { COMPLETE_AFTER_DELAY, NEVER_COMPLETE, COMMIT_WITHOUT_CALLBACK_TAIL };
static int delay_mode;
#define portENTER_CRITICAL(lock) do { (void)(lock); assert(!in_isr && !lock_depth); ++lock_depth; } while(0)
#define portEXIT_CRITICAL(lock) do { (void)(lock); assert(!in_isr && lock_depth == 1); --lock_depth; } while(0)
#define portENTER_CRITICAL_ISR(lock) do { (void)(lock); assert(in_isr && !lock_depth); ++lock_depth; } while(0)
#define portEXIT_CRITICAL_ISR(lock) do { (void)(lock); assert(in_isr && lock_depth == 1); --lock_depth; } while(0)
static bool xPortInIsrContext(void) { return in_isr; }
static TaskHandle_t xTaskGetCurrentTaskHandle(void) { return caller; }
static int64_t esp_timer_get_time(void) { return now_us; }
static int dpi_panel_draw_bitmap(void) { return 0; }
static int dpi_panel_draw_bitmap_2d(void) { return 0; }
static void *dw_gdma_link_list_get_item(void *list, int index) { assert(index == 0); return list; }
static void dw_gdma_lli_set_block_markers(void *item, dw_gdma_block_markers_t markers)
{ assert(item && markers.is_valid && markers.is_last && !lock_depth); }
static int dw_gdma_channel_use_link_list(void *channel, void *list)
{ assert(channel == running_panel->dma_chan); selected_list = list; return ESP_OK; }
static int dw_gdma_channel_enable_ctrl(void *channel, bool enabled)
{ assert(channel == running_panel->dma_chan && enabled); ++restarts; return ESP_OK; }
static bool observer(esp_lcd_panel_t *panel, const p4desk_lcd_frame_event_t *event, void *arg)
{
    assert(panel == &running_panel->base && arg == &context && !lock_depth);
    last = *event; ++observed; return false;
}
static void mipi_dsi_brg_ll_enable_dpi_output(fake_bridge_t *dev, bool enabled)
{ dev->dpi_misc_config.val = (dev->dpi_misc_config.val & ~1U) | enabled; }
static void mipi_dsi_brg_ll_update_dpi_config(fake_bridge_t *dev) { (void)dev; }
static void mipi_dsi_brg_ll_enable(fake_bridge_t *dev, bool enabled) { dev->en.val = enabled; }
static void mipi_dsi_brg_ll_clear_interrupt_status(fake_bridge_t *dev, uint32_t mask)
{ (void)dev; assert(mask == UINT32_MAX); }
static void mipi_dsi_host_ll_power_on_off(fake_host_t *dev, bool enabled)
{
    if (enabled) ++power_ups; else ++power_downs;
    dev->pwr_up.val = enabled;
}
static void mipi_dsi_brg_ll_reset(fake_bridge_t *dev)
{
    // Model reset destroying config: every saved R/W field must be restored.
    assert(!in_isr && !lock_depth); ++bridge_resets;
    memset(dev, 0, sizeof(*dev));
}
bool mipi_dsi_dma_trans_done_cb(void *, const dw_gdma_trans_done_event_data_t *, void *);
static void vTaskDelay(int ticks)
{
    assert(ticks == 1 && !in_isr && !lock_depth);
    now_us += 1000;
    if (delay_mode == COMPLETE_AFTER_DELAY && running_panel->scan_stop_state == P4DESK_SCAN_REQUESTED) {
        in_isr = true; mipi_dsi_dma_trans_done_cb(running_panel->dma_chan, NULL, running_panel); in_isr = false;
    } else if (delay_mode == COMMIT_WITHOUT_CALLBACK_TAIL) {
        // Model deadline racing the ISR after its commitment lock, before its
        // observer has returned: timeout cannot cancel or restart hardware.
        running_panel->scan_stop_state = P4DESK_SCAN_STOPPING;
    }
}
'''
        fixture = fixture.replace("@REG_TYPES@", f"typedef struct {{\n{host_members}\n}} fake_host_t;\ntypedef struct {{\n{bridge_members}\n}} fake_bridge_t;")
        fixture = fixture.replace("@RESULT_TYPE@", header[result_start:result_end])
        checks = r'''
int main(void)
{
    uint8_t storage[3];
    fake_host_t host = {.pwr_up.val = 1, .vid_hline_time.val = 3250, .int_st0.val = 0x40, .int_st1.val = 0x80000};
    fake_bridge_t bridge = {.dpi_misc_config.val = 1, .dpi_h_cfg0.val = 1248, .pixel_type.val = 2, .int_ena.val = 3};
    fake_bus_t bus = {.hal = {.host = &host, .bridge = &bridge}};
    esp_lcd_dpi_panel_t panel = {
        .base = {.draw_bitmap = dpi_panel_draw_bitmap, .draw_bitmap_2d = dpi_panel_draw_bitmap_2d},
        .bus = &bus, .host_error_owner = (void *)1,
        .scanning_fb_index = 0, .cur_fb_index = 2, .frame_observer_counter = 40,
        .frame_observer_callback = observer, .frame_observer_context = &context,
        .fbs = {&storage[0], &storage[1], &storage[2]},
        .link_lists = {&storage[0], &storage[1], &storage[2]}, .dma_chan = &context,
    };
    running_panel = &panel;
    p4desk_lcd_resync_result_t result;
    assert(p4desk_lcd_scan_resync(&panel.base, 0, &result) == ESP_ERR_INVALID_ARG);
    assert(p4desk_lcd_scan_resync(&panel.base, 1001, &result) == ESP_ERR_INVALID_ARG);
    in_isr = true;
    assert(p4desk_lcd_scan_resync(&panel.base, 100, &result) == ESP_ERR_INVALID_STATE);
    in_isr = false; caller = (void *)2;
    assert(p4desk_lcd_scan_resync(&panel.base, 100, &result) == ESP_ERR_INVALID_STATE);
    caller = (void *)1;
    // Full stop/restore/resume: A,A is delivered, pending B remains selected,
    // but resumed DMA reads A. Configuration and completion counter survive.
    delay_mode = COMPLETE_AFTER_DELAY;
    assert(p4desk_lcd_scan_resync(&panel.base, 100, &result) == ESP_OK);
    assert(result.stopped && result.resumed && !result.request_cancelled);
    assert(result.counter == 41 && result.scanning_index == 0 && result.selected_index == 2);
    assert(last.completed_index == 0 && last.next_index == 0 && last.counter == 41);
    assert(selected_list == panel.link_lists[0] && panel.cur_fb_index == 2 && panel.scanning_fb_index == 0);
    assert(host.vid_hline_time.val == 3250 && bridge.dpi_h_cfg0.val == 1248 && bridge.pixel_type.val == 2);
    assert(bridge.int_ena.val == 3 && bridge.dpi_misc_config.dpi_en && host.pwr_up.shutdownz);
    assert(result.recovery_status0 == 0x40 && result.recovery_status1 == 0x80000);
    assert(power_downs == 1 && power_ups == 1 && bridge_resets == 1 && restarts == 1 && observed == 1);
    in_isr = true; mipi_dsi_dma_trans_done_cb(panel.dma_chan, NULL, &panel); in_isr = false;
    assert(last.completed_index == 0 && last.next_index == 2 && last.counter == 42);
    // Cancellation wins before ISR commitment. No hardware is stopped/reset.
    delay_mode = NEVER_COMPLETE;
    assert(p4desk_lcd_scan_resync(&panel.base, 3, &result) == ESP_ERR_TIMEOUT);
    assert(result.request_cancelled && !result.stopped && !result.resumed && result.elapsed_us == 3000);
    assert(panel.scan_stop_state == P4DESK_SCAN_RUNNING && bridge_resets == 1 && restarts == 2);
    // A late real completion after cancellation is allowed to restart normally.
    in_isr = true; mipi_dsi_dma_trans_done_cb(panel.dma_chan, NULL, &panel); in_isr = false;
    assert(panel.scan_stop_state == P4DESK_SCAN_RUNNING && restarts == 3);
    // Once the ISR commits stopping, deadline must fail closed. Caller keeps
    // ownership; no unsafe cancellation, reinitialization, or DMA abort occurs.
    delay_mode = COMMIT_WITHOUT_CALLBACK_TAIL;
    assert(p4desk_lcd_scan_resync(&panel.base, 3, &result) == ESP_ERR_TIMEOUT);
    assert(result.stopped && !result.resumed && !result.request_cancelled);
    assert(panel.scan_stop_state == P4DESK_SCAN_STOPPING && bridge_resets == 1 && restarts == 3);
    assert(p4desk_lcd_scan_resync(&panel.base, 3, &result) == ESP_ERR_INVALID_STATE);
    assert(lock_depth == 0);
    return 0;
}
'''
        with tempfile.TemporaryDirectory(prefix="p4desk-dpi-resync-") as directory:
            source = Path(directory) / "probe.c"
            executable = Path(directory) / "probe"
            source.write_text(fixture + callback + "\n" + owner_check + "\n" + resync + checks)
            result = subprocess.run([compiler, "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                                     "-Wno-unused-parameter", str(source), "-o", str(executable)],
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            result = subprocess.run([str(executable)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_actual_timing_snapshot_uses_live_clock_and_matches_cross_product(self) -> None:
        compiler = shutil.which("cc")
        if not compiler:
            self.skipTest("host C compiler unavailable")
        timing = function_source(self.patched_text, "esp_err_t p4desk_lcd_scan_timing(",
                                 "esp_err_t p4desk_lcd_scan_resync(")
        header = (COMPONENT / "include/p4desk_lcd_frame_observer.h").read_text()
        start = header.index("typedef struct {", header.index("/** Live scan registers"))
        end = header.index("} p4desk_lcd_scan_timing_t;", start) + len("} p4desk_lcd_scan_timing_t;")
        fixture = r'''
#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
typedef int esp_err_t;
#define ESP_OK 0
#define ESP_ERR_INVALID_ARG 1
#define ESP_ERR_INVALID_STATE 2
#define ESP_RETURN_ON_FALSE(c, e, ...) do { if (!(c)) return (e); } while(0)
#define ESP_RETURN_ON_ERROR(c, ...) do { int e=(c); if(e) return e; } while(0)
enum { MIPI_DSI_DPI_CLK_SRC_XTAL, MIPI_DSI_DPI_CLK_SRC_PLL_F240M,
       MIPI_DSI_DPI_CLK_SRC_PLL_F160M, MIPI_DSI_DPI_CLK_SRC_APLL,
       ESP_CLK_TREE_SRC_FREQ_PRECISION_CACHED };
typedef int mipi_dsi_dpi_clock_source_t;
typedef struct {
    struct { uint32_t htotal, hdisp; } dpi_h_cfg0;
    struct { uint32_t vtotal, vdisp; } dpi_v_cfg0;
    uint32_t depth;
} bridge_t;
typedef struct {
    struct { uint32_t vid_hline_time; } vid_hline_time;
    struct { uint32_t vsa_lines; } vid_vsa_lines;
    struct { uint32_t vbp_lines; } vid_vbp_lines;
    struct { uint32_t vfp_lines; } vid_vfp_lines;
    struct { uint32_t v_active_lines; } vid_vactive_lines;
    struct { uint32_t val; } vid_pkt_status;
} host_t;
typedef struct { bridge_t *bridge; host_t *host; float lane_bit_rate_mbps; } mipi_dsi_hal_context_t;
typedef struct { mipi_dsi_hal_context_t hal; } bus_t;
typedef struct {
    bus_t *bus; int frame_observer_lock;
    uint32_t frame_observer_counter; uint8_t scanning_fb_index, cur_fb_index;
} esp_lcd_dpi_panel_t;
typedef esp_lcd_dpi_panel_t *esp_lcd_panel_handle_t;
@TYPE@
static struct { struct { uint32_t reg_mipi_dsi_dpiclk_src_sel, reg_mipi_dsi_dpiclk_div_num; } peri_clk_ctrl03; } HP_SYS_CLKRST;
static bool owner = true;
static int lock_depth;
#define portENTER_CRITICAL(lock) do { (void)(lock); assert(!lock_depth); ++lock_depth; } while(0)
#define portEXIT_CRITICAL(lock) do { (void)(lock); assert(lock_depth == 1); --lock_depth; } while(0)
static int p4desk_scan_owner_check(esp_lcd_panel_handle_t panel, esp_lcd_dpi_panel_t **out)
{ if(!panel) return ESP_ERR_INVALID_ARG; if(!owner) return ESP_ERR_INVALID_STATE; *out=panel; return ESP_OK; }
static int esp_clk_tree_src_get_freq_hz(int source, int precision, uint32_t *frequency)
{ assert(source == MIPI_DSI_DPI_CLK_SRC_PLL_F240M && precision == ESP_CLK_TREE_SRC_FREQ_PRECISION_CACHED); *frequency=240000000; return ESP_OK; }
static uint32_t mipi_dsi_brg_ll_get_fifo_depth(bridge_t *bridge) { return bridge->depth; }
'''
        checks = r'''
int main(void)
{
    bridge_t bridge = {.dpi_h_cfg0 = {1250, 1024}, .dpi_v_cfg0 = {636, 600}, .depth = 512};
    host_t host = {.vid_hline_time = {3255}, .vid_vsa_lines = {1}, .vid_vbp_lines = {23},
                   .vid_vfp_lines = {12}, .vid_vactive_lines = {600}, .vid_pkt_status = {0x22}};
    bus_t bus = {.hal = {.bridge = &bridge, .host = &host, .lane_bit_rate_mbps = 1000.0f}};
    esp_lcd_dpi_panel_t panel = {.bus = &bus, .frame_observer_counter = 42, .scanning_fb_index = 0, .cur_fb_index = 2};
    HP_SYS_CLKRST.peri_clk_ctrl03.reg_mipi_dsi_dpiclk_src_sel = 1;
    HP_SYS_CLKRST.peri_clk_ctrl03.reg_mipi_dsi_dpiclk_div_num = 4;
    p4desk_lcd_scan_timing_t snapshot;
    assert(p4desk_lcd_scan_timing(&panel, &snapshot) == ESP_OK);
    assert(snapshot.source_hz == 240000000 && snapshot.divider == 5 && snapshot.dpi_hz == 48000000);
    assert(snapshot.lane_bit_rate_kbps == 1000000 && snapshot.bridge_h_total == 1250);
    assert(snapshot.bridge_v_total == 636 && snapshot.bridge_h_active == 1024 && snapshot.bridge_v_active == 600);
    assert(snapshot.host_hline_byte_clocks == 3255 && snapshot.host_v_total == 636);
    assert(snapshot.counter == 42 && snapshot.scanning_index == 0 && snapshot.selected_index == 2);
    assert(snapshot.bridge_fifo_depth == 512 && snapshot.host_vid_status == 0x22);
    uint64_t brg = (uint64_t)snapshot.bridge_h_total * snapshot.lane_bit_rate_kbps * 1000;
    uint64_t hst = (uint64_t)snapshot.host_hline_byte_clocks * 8 * snapshot.dpi_hz;
    assert(brg != hst); // independently rounded original line periods
    bridge.dpi_h_cfg0.htotal = 1248; host.vid_hline_time.vid_hline_time = 3250;
    assert(p4desk_lcd_scan_timing(&panel, &snapshot) == ESP_OK);
    brg = (uint64_t)snapshot.bridge_h_total * snapshot.lane_bit_rate_kbps * 1000;
    hst = (uint64_t)snapshot.host_hline_byte_clocks * 8 * snapshot.dpi_hz;
    assert(brg == hst); // matched line periods
    owner = false;
    assert(p4desk_lcd_scan_timing(&panel, &snapshot) == ESP_ERR_INVALID_STATE && snapshot.counter == 0);
    assert(p4desk_lcd_scan_timing(&panel, NULL) == ESP_ERR_INVALID_ARG);
    assert(panel.frame_observer_counter == 42 && panel.cur_fb_index == 2 && lock_depth == 0);
    return 0;
}
'''
        fixture = fixture.replace("@TYPE@", header[start:end])
        with tempfile.TemporaryDirectory(prefix="p4desk-dpi-timing-") as directory:
            source = Path(directory) / "probe.c"
            executable = Path(directory) / "probe"
            source.write_text(fixture + timing + checks)
            result = subprocess.run([compiler, "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror",
                                     str(source), "-o", str(executable)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            result = subprocess.run([str(executable)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_component_cmake_defers_exactly_one_source_replacement(self) -> None:
        cmake = shutil.which("cmake")
        if not cmake:
            self.skipTest("CMake unavailable")
        with tempfile.TemporaryDirectory(prefix="p4desk-dpi-cmake-") as directory:
            project = Path(directory)
            # This configures the real component with fake IDF registration;
            # it does not configure, build or mutate the shared IDF project.
            source = f'''
cmake_minimum_required(VERSION 3.22)
project(dpi_observer_probe C)
set(IDF_VERSION_MAJOR 6)
set(IDF_VERSION_MINOR 0)
set(IDF_VERSION_PATCH 2)
set(CONFIG_SOC_MIPI_DSI_SUPPORTED TRUE)
set(CONFIG_LCD_DSI_OBJ_FORCE_INTERNAL TRUE)
function(idf_component_register)
    add_library(__idf_lcd_frame_observer INTERFACE)
endfunction()
function(idf_component_get_property output component property)
    if(component STREQUAL "esp_lcd" AND property STREQUAL "COMPONENT_DIR")
        set(${{output}} "{SDK_SOURCE.parents[1]}" PARENT_SCOPE)
    elseif(component STREQUAL "esp_timer" AND property STREQUAL "COMPONENT_LIB")
        set(${{output}} __idf_esp_timer PARENT_SCOPE)
    else()
        message(FATAL_ERROR "unexpected component property lookup")
    endif()
endfunction()
function(idf_build_get_property output property)
    set(${{output}} "{sys.executable}" PARENT_SCOPE)
endfunction()
file(WRITE "${{CMAKE_CURRENT_BINARY_DIR}}/other.c" "int other;\\n")
add_library(__idf_esp_lcd STATIC "{SDK_SOURCE}" "${{CMAKE_CURRENT_BINARY_DIR}}/other.c")
add_library(__idf_esp_timer INTERFACE)
target_include_directories(__idf_esp_timer INTERFACE "${{CMAKE_CURRENT_BINARY_DIR}}/timer/include")
add_subdirectory("{COMPONENT}" observer)
function(check_replacement)
    get_target_property(sources __idf_esp_lcd SOURCES)
    list(LENGTH sources count)
    if(NOT count EQUAL 2)
        message(FATAL_ERROR "expected unchanged other source plus generated DPI")
    endif()
    if("{SDK_SOURCE}" IN_LIST sources)
        message(FATAL_ERROR "original SDK DPI still in target")
    endif()
    set(generated "${{CMAKE_CURRENT_BINARY_DIR}}/observer/generated/esp_lcd_panel_dpi.c")
    if(NOT generated IN_LIST sources OR NOT EXISTS "${{generated}}")
        message(FATAL_ERROR "generated DPI source missing")
    endif()
    get_target_property(includes __idf_esp_lcd INCLUDE_DIRECTORIES)
    if(NOT "{SDK_SOURCE.parent}" IN_LIST includes OR NOT "{COMPONENT / 'include'}" IN_LIST includes)
        message(FATAL_ERROR "DPI private or observer header include missing")
    endif()
    get_target_property(links __idf_esp_lcd LINK_LIBRARIES)
    if(NOT __idf_esp_timer IN_LIST links)
        message(FATAL_ERROR "generated DPI needs esp_timer's include and link dependency")
    endif()
    get_target_property(interface_links __idf_esp_lcd INTERFACE_LINK_LIBRARIES)
    if(NOT "$<LINK_ONLY:__idf_esp_timer>" IN_LIST interface_links)
        message(FATAL_ERROR "esp_timer must remain a private static-library dependency")
    endif()
endfunction()
cmake_language(DEFER CALL check_replacement)
'''
            (project / "CMakeLists.txt").write_text(source)
            subprocess.run([cmake, "-S", str(project), "-B", str(project / "build")],
                           check=True, capture_output=True, text=True)
            self.assertEqual(SDK_SOURCE.read_bytes(), self.source)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--idf-path", type=Path, default=DEFAULT_IDF_PATH,
                        help="ESP-IDF 6.0.2 root; overrides IDF_PATH and the local pinned SDK")
    args, remaining = parser.parse_known_args()
    SDK_SOURCE = args.idf_path.expanduser().resolve() / "components/esp_lcd/dsi/esp_lcd_panel_dpi.c"
    unittest.main(argv=[sys.argv[0], *remaining])
