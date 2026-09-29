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
        self.assertIn(bridge, self.patched_text)
        original_register = function_source(self.original_text, "esp_err_t esp_lcd_dpi_panel_register_event_callbacks")
        self.assertIn(original_register, self.patched_text)
        refresh = self.original_text.split("#if !MIPI_DSI_BRG_LL_EVENT_VSYNC\n", 1)[1].split("#endif", 1)[0]
        self.assertIn(refresh, self.patched_text)

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
    assert(lock_depth == 0 && restarts == observed + 1);
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
    set(${{output}} "{SDK_SOURCE.parents[1]}" PARENT_SCOPE)
endfunction()
function(idf_build_get_property output property)
    set(${{output}} "{sys.executable}" PARENT_SCOPE)
endfunction()
file(WRITE "${{CMAKE_CURRENT_BINARY_DIR}}/other.c" "int other;\\n")
add_library(__idf_esp_lcd STATIC "{SDK_SOURCE}" "${{CMAKE_CURRENT_BINARY_DIR}}/other.c")
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
