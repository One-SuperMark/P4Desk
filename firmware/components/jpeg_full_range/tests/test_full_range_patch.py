#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Host-only checks of the actual generated decoder; never opens hardware."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unittest


COMPONENT = Path(__file__).resolve().parents[1]
TOOL = COMPONENT / "tools/generate_patch.py"
SPEC = importlib.util.spec_from_file_location("jpeg_full_range_patch", TOOL)
PATCH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PATCH)
IDF_PATH = Path(os.environ.get("IDF_PATH") or "/Volumes/work/esp/esp-idf-v6.0.2").expanduser().resolve()


def function_source(text: str, name: str) -> str:
    cursor = 0
    while True:
        start = text.index(name, cursor)
        brace = text.index("{", start)
        semicolon = text.find(";", start, brace)
        if semicolon == -1:
            break
        # Skip the forward declaration rather than extracting the next body.
        cursor = semicolon + 1
    depth = 0
    for index in range(brace, len(text)):
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                return text[start:index + 1]
    raise ValueError(f"unclosed C function {name}")


def signed(value: int, width: int) -> int:
    return value - (1 << width) if value & (1 << (width - 1)) else value


class FullRangePatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.sdk_source = IDF_PATH / "components/esp_driver_jpeg/jpeg_decode.c"
        if not cls.sdk_source.is_file():
            raise unittest.SkipTest(f"Pinned SDK source unavailable: {cls.sdk_source}; set --idf-path or IDF_PATH")
        cls.source = cls.sdk_source.read_bytes()
        cls.dma_source = IDF_PATH / "components/esp_driver_dma/src/dma2d.c"
        cls.generated = PATCH.patch_source(cls.source)
        cls.original_text = cls.source.decode("utf-8")
        cls.generated_text = cls.generated.decode("utf-8")

    def test_pinned_source_and_exact_change_scope(self) -> None:
        self.assertEqual(hashlib.sha256(self.source).hexdigest(), PATCH.PINNED_SHA256)
        self.assertEqual(self.source.split(b"*/", 1)[0], self.generated.split(b"*/", 1)[0])
        restored = self.generated_text
        for _, before, after in reversed(PATCH.REPLACEMENTS):
            self.assertEqual(restored.count(after), 1)
            restored = restored.replace(after, before, 1)
        self.assertEqual(restored.encode("utf-8"), self.source)
        callback = function_source(self.original_text, "static bool jpeg_dec_transaction_on_picked(uint32_t channel_num")
        self.assertIn(callback, self.generated_text)
        self.assertLess(callback.index("jpeg_dec_config_dma_csc("), callback.index("dma2d_start(tx_chan)"))
        self.assertLess(callback.index("jpeg_dec_config_dma_csc("), callback.index("dma2d_start(rx_chan)"))

    def test_rejects_sha_and_every_anchor_drift(self) -> None:
        with self.assertRaisesRegex(ValueError, "SHA256 mismatch"):
            PATCH.patch_source(self.source + b"\n")
        anchors = [(name, before) for name, before, _ in PATCH.REPLACEMENTS] + list(PATCH.OWNERSHIP_ANCHORS)
        for name, anchor in anchors:
            self.assertEqual(self.original_text.count(anchor), 1, name)
            with self.subTest(anchor=name, kind="missing"):
                altered = self.source.replace(anchor.encode(), b"", 1)
                with self.assertRaisesRegex(ValueError, "found 0"):
                    PATCH.patch_source(altered, verify_sha=False)
            with self.subTest(anchor=name, kind="duplicate"):
                with self.assertRaisesRegex(ValueError, "found 2"):
                    PATCH.patch_source(self.source + anchor.encode(), verify_sha=False)
        with self.assertRaisesRegex(ValueError, "already contains"):
            PATCH.patch_source(self.generated, verify_sha=False)

    def test_actual_coefficients_fit_sdk_fields(self) -> None:
        helper = PATCH.render_helper()
        for version in ("hw_ver1", "hw_ver3"):
            header = IDF_PATH / f"components/soc/esp32p4/register/{version}/soc/dma2d_struct.h"
            text = header.read_text()
            body = text.split("} dma2d_color_param_reg_t;", 1)[0].rsplit("typedef union {", 1)[1]
            widths = {name: int(width) for name, width in re.findall(r"uint32_t\s+([abcd])\s*:\s*(\d+)", body)}
            self.assertEqual(tuple(widths[name] for name in "abcd"), PATCH.COEFFICIENT_WIDTHS)
        for name, row in zip("hml", PATCH.FULL_RANGE_COEFFICIENTS):
            for field, expected, width in zip("abcd", row, PATCH.COEFFICIENT_WIDTHS):
                match = re.search(rf"params\.param_{name}\.{field} = (\d+)u;", helper)
                self.assertIsNotNone(match)
                actual = int(match.group(1))
                self.assertLess(actual, 1 << width)
                self.assertEqual(signed(actual, width), expected)
        for width in PATCH.COEFFICIENT_WIDTHS:
            for invalid in (-(1 << (width - 1)) - 1, 1 << (width - 1)):
                with self.assertRaises(ValueError):
                    PATCH.encode_coefficient(invalid, width)

    def test_jfif_formula_precision_and_all_neutral_grays(self) -> None:
        # Independently specified JFIF coefficients, not a copy of Q8 values.
        jfif = ((1.0, 0.0, 1.402), (1.0, -0.34414, -0.71414), (1.0, 1.772, 0.0))
        bounds = (0.044, 0.140, 0.184)
        for row, reference, bound in zip(PATCH.FULL_RANGE_COEFFICIENTS, jfif, bounds):
            a, b, c, d = row
            maximum_error = 0.0
            for cb in range(256):
                for cr in range(256):
                    expected = reference[1] * (cb - 128) + reference[2] * (cr - 128)
                    actual = (b * cb + c * cr + d) / 256
                    maximum_error = max(maximum_error, abs(actual - expected))
            self.assertLessEqual(maximum_error, bound + 1e-10)
            self.assertEqual(a, 256)
            self.assertEqual(d, -128 * (b + c))
            for y in range(256):
                self.assertEqual((a * y + b * 128 + c * 128 + d) // 256, y)
        # Q8 truncation versus independently rounded JFIF has <=1 level error
        # over a range grid; final hardware rounding/565 packing are separate.
        for y in range(256):
            for cb in (0, 16, 64, 128, 192, 240, 255):
                for cr in (0, 16, 64, 128, 192, 240, 255):
                    for row, reference in zip(PATCH.FULL_RANGE_COEFFICIENTS, jfif):
                        actual = min(255, max(0, sum(v * x for v, x in zip(row, (y, cb, cr, 1))) // 256))
                        expected = min(255, max(0, math.floor(y + reference[1] * (cb - 128) + reference[2] * (cr - 128) + 0.5)))
                        self.assertLessEqual(abs(actual - expected), 1)

    def test_rx_zero_scheduler_and_scope_guards(self) -> None:
        PATCH.verify_dma_source(self.dma_source.read_bytes())
        with self.assertRaisesRegex(ValueError, "DMA2D scheduler SHA256 mismatch"):
            PATCH.verify_dma_source(self.dma_source.read_bytes() + b"\n")
        ll = (IDF_PATH / "components/esp_hal_dma/esp32p4/include/hal/dma2d_ll.h").read_text()
        self.assertRegex(ll, r"#define DMA2D_LL_INST_NUM\s+1\b")
        self.assertRegex(ll, r"#define DMA2D_LL_RX_CHANNEL_SUPPORT_RO_MASK\s+\(0U \| BIT0\)")
        self.assertRegex(ll, r"#define DMA2D_LL_RX_CHANNEL_SUPPORT_CSC_MASK\s+\(0U \| BIT0\)")
        allocator = (IDF_PATH / "components/esp_driver_dma/src/dma2d.c").read_text()
        self.assertIn("trans_desc->channel_flags & DMA2D_CHANNEL_FUNCTION_FLAG_RX_REORDER) ? DMA2D_LL_RX_CHANNEL_SUPPORT_RO_MASK", allocator)
        helper = PATCH.render_helper()
        self.assertIn("dma2d_color_param_group_chn_reg_t params = {0};", helper)
        self.assertEqual(helper.count("dev->in_channel[0].in_color_param_group."), 6)
        for guard in ("DMA2D_LL_INST_NUM", "DMA2D_LL_RX_CHANNEL_SUPPORT_RO_MASK", "DMA2D_LL_RX_CHANNEL_SUPPORT_CSC_MASK"):
            self.assertIn(f"_Static_assert({guard}", helper)
        csc = function_source(self.generated_text, "static void jpeg_dec_config_dma_csc(")
        self.assertLess(csc.index("dma2d_configure_color_space_conversion(rx_chan"), csc.index("p4desk_jpeg_config_full_range_bt601();"))
        self.assertEqual(csc.count("p4desk_jpeg_config_full_range_bt601();"), 1)
        self.assertIn("decoder_engine->conv_std == JPEG_YUV_RGB_CONV_STD_BT601 &&", csc)

    def test_generated_c_registers_endian_gray_and_next_transaction(self) -> None:
        compiler = shutil.which("cc")
        if not compiler:
            self.skipTest("host C compiler unavailable")
        # Use the actual pinned SDK bitfields and generated configuration body.
        register_header = (IDF_PATH / "components/soc/esp32p4/register/hw_ver3/soc/dma2d_struct.h").read_text()
        param_end = register_header.index("} dma2d_color_param_reg_t;") + len("} dma2d_color_param_reg_t;")
        param_start = register_header.rfind("typedef union {", 0, param_end)
        group_end = register_header.index("} dma2d_color_param_group_chn_reg_t;", param_end) + len("} dma2d_color_param_group_chn_reg_t;")
        group_start = register_header.index("typedef struct {", param_end)
        bitfields = register_header[param_start:param_end] + "\n" + register_header[group_start:group_end]
        csc = function_source(self.generated_text, "static void jpeg_dec_config_dma_csc(")
        options = sorted(set(re.findall(r"\bDMA2D_CSC_RX_[A-Z0-9_]+\b", csc)))
        mock = r'''
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>
#define CONFIG_ESP_REV_MIN_FULL 300
#define SOC_IS(target) 1
#define DMA2D_LL_INST_NUM 1
#define DMA2D_LL_RX_CHANNEL_SUPPORT_RO_MASK 1u
#define DMA2D_LL_RX_CHANNEL_SUPPORT_CSC_MASK 1u
enum { JPEG_DEC_RGB_ELEMENT_ORDER_BGR, JPEG_DEC_RGB_ELEMENT_ORDER_RGB };
enum { JPEG_DECODE_OUT_FORMAT_RGB565, JPEG_DECODE_OUT_FORMAT_RGB888,
       JPEG_DECODE_OUT_FORMAT_GRAY, JPEG_DECODE_OUT_FORMAT_YUV444, JPEG_DECODE_OUT_FORMAT_YUV420 };
enum { JPEG_YUV_RGB_CONV_STD_BT601, JPEG_YUV_RGB_CONV_STD_BT709 };
enum { JPEG_DOWN_SAMPLING_YUV444, JPEG_DOWN_SAMPLING_YUV422, JPEG_DOWN_SAMPLING_YUV420, JPEG_DOWN_SAMPLING_GRAY };
enum { DMA2D_SCRAMBLE_ORDER_BYTE2_1_0, DMA2D_SCRAMBLE_ORDER_BYTE2_0_1, DMA2D_SCRAMBLE_ORDER_BYTE0_1_2 };
typedef int dma2d_scramble_order_t;
typedef int dma2d_csc_rx_option_t;
typedef void *dma2d_channel_handle_t;
typedef struct { int post_scramble, rx_csc_option; } dma2d_csc_config_t;
typedef struct { int rgb_order, output_format, conv_std, sample_method; } decoder_t;
typedef decoder_t *jpeg_decoder_handle_t;
'''
        state = r'''
typedef struct { struct { dma2d_color_param_group_chn_reg_t in_color_param_group; } in_channel[1]; } dma2d_dev_t;
static dma2d_dev_t device;
#define DMA2D_LL_GET_HW(pool) ((pool) == 0 ? &device : 0)
static dma2d_csc_config_t last_config;
static unsigned config_count;
static int dma2d_configure_color_space_conversion(dma2d_channel_handle_t chan, const dma2d_csc_config_t *config)
{
    assert(chan == (void *)1);
    last_config = *config;
    ++config_count;
    // Simulate each new owner's normal CSC setup replacing all six params.
    for (unsigned row = 0; row < 3; ++row) {
        volatile dma2d_color_param_reg_t *p = row == 0 ? &device.in_channel[0].in_color_param_group.param_h :
                                            row == 1 ? &device.in_channel[0].in_color_param_group.param_m :
                                                       &device.in_channel[0].in_color_param_group.param_l;
        p->val[0] = 0x00123456u + row;
        p->val[1] = 0x01234567u + row;
    }
    return 0;
}
static void check_full_range(void)
{
    static const uint32_t expected[3][2] = {
        {256u, 359u | (216192u << 10)},
        {256u | (1960u << 10), 841u | (34688u << 10)},
        {256u | (454u << 10), 204032u << 10},
    };
    for (unsigned row = 0; row < 3; ++row) {
        volatile dma2d_color_param_reg_t *p = row == 0 ? &device.in_channel[0].in_color_param_group.param_h :
                                            row == 1 ? &device.in_channel[0].in_color_param_group.param_m :
                                                       &device.in_channel[0].in_color_param_group.param_l;
        assert(p->val[0] == expected[row][0] && p->val[1] == expected[row][1]);
        assert((p->val[0] >> 21) == 0 && (p->val[1] >> 28) == 0);
    }
}
static void check_original(void)
{
    assert(device.in_channel[0].in_color_param_group.param_h.val[0] == 0x00123456u);
    assert(device.in_channel[0].in_color_param_group.param_h.val[1] == 0x01234567u);
    assert(device.in_channel[0].in_color_param_group.param_m.val[0] == 0x00123457u);
    assert(device.in_channel[0].in_color_param_group.param_m.val[1] == 0x01234568u);
    assert(device.in_channel[0].in_color_param_group.param_l.val[0] == 0x00123458u);
    assert(device.in_channel[0].in_color_param_group.param_l.val[1] == 0x01234569u);
}
'''
        checks = r'''
int main(void)
{
    decoder_t decoder = {.output_format = JPEG_DECODE_OUT_FORMAT_RGB565,
        .rgb_order = JPEG_DEC_RGB_ELEMENT_ORDER_BGR, .conv_std = JPEG_YUV_RGB_CONV_STD_BT601,
        .sample_method = JPEG_DOWN_SAMPLING_YUV420};
    jpeg_dec_config_dma_csc(&decoder, (void *)1);
    assert(config_count == 1 && last_config.rx_csc_option == DMA2D_CSC_RX_YUV420_TO_RGB565_601);
    assert(last_config.post_scramble == DMA2D_SCRAMBLE_ORDER_BYTE2_1_0); check_full_range();
    decoder.rgb_order = JPEG_DEC_RGB_ELEMENT_ORDER_RGB;
    jpeg_dec_config_dma_csc(&decoder, (void *)1);
    assert(last_config.post_scramble == DMA2D_SCRAMBLE_ORDER_BYTE2_0_1); check_full_range();
    decoder.output_format = JPEG_DECODE_OUT_FORMAT_RGB888;
    jpeg_dec_config_dma_csc(&decoder, (void *)1);
    assert(last_config.rx_csc_option == DMA2D_CSC_RX_YUV420_TO_RGB888_601);
    assert(last_config.post_scramble == DMA2D_SCRAMBLE_ORDER_BYTE0_1_2); check_full_range();
    decoder.rgb_order = JPEG_DEC_RGB_ELEMENT_ORDER_BGR;
    jpeg_dec_config_dma_csc(&decoder, (void *)1);
    assert(last_config.post_scramble == DMA2D_SCRAMBLE_ORDER_BYTE2_1_0); check_full_range();
    decoder.conv_std = JPEG_YUV_RGB_CONV_STD_BT709;
    jpeg_dec_config_dma_csc(&decoder, (void *)1);
    assert(last_config.rx_csc_option == DMA2D_CSC_RX_YUV420_TO_RGB888_709); check_original();
    decoder.output_format = JPEG_DECODE_OUT_FORMAT_RGB565;
    jpeg_dec_config_dma_csc(&decoder, (void *)1);
    assert(last_config.rx_csc_option == DMA2D_CSC_RX_YUV420_TO_RGB565_709); check_original();
    decoder.conv_std = JPEG_YUV_RGB_CONV_STD_BT601;
    decoder.output_format = JPEG_DECODE_OUT_FORMAT_GRAY;
    jpeg_dec_config_dma_csc(&decoder, (void *)1);
    assert(last_config.rx_csc_option == DMA2D_CSC_RX_NONE); check_original();
    decoder.output_format = JPEG_DECODE_OUT_FORMAT_YUV444;
    decoder.sample_method = JPEG_DOWN_SAMPLING_YUV422;
    jpeg_dec_config_dma_csc(&decoder, (void *)1);
    assert(last_config.rx_csc_option == DMA2D_CSC_RX_YUV422_TO_YUV444); check_original();
    assert(config_count == 8);
    return 0;
}
'''
        license_header = self.original_text.split("*/", 1)[0] + "*/\n"
        fixture = license_header + mock + "enum {" + ",".join(options) + "};\n" + bitfields + state + PATCH.render_helper() + csc + checks
        with tempfile.TemporaryDirectory(prefix="p4desk-jpeg-registers-") as directory:
            source = Path(directory) / "test.c"
            binary = Path(directory) / "test"
            source.write_text(fixture)
            result = subprocess.run([compiler, "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", str(source), "-o", str(binary)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            subprocess.run([str(binary)], check=True, capture_output=True, text=True)
            for guard in ("DMA2D_LL_INST_NUM", "DMA2D_LL_RX_CHANNEL_SUPPORT_RO_MASK", "DMA2D_LL_RX_CHANNEL_SUPPORT_CSC_MASK"):
                invalid_source = Path(directory) / "invalid.c"
                invalid_source.write_text(re.sub(rf"#define {guard} 1u?\n", f"#define {guard} 2u\n", fixture))
                result = subprocess.run([compiler, "-std=c11", "-fsyntax-only", str(invalid_source)], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0, guard)

    def test_cli_is_repeatable_and_sdk_entire_tree_is_read_only(self) -> None:
        with tempfile.TemporaryDirectory(prefix="p4desk-jpeg-generator-") as directory:
            root = Path(directory)
            output = root / "generated/jpeg_decode.c"
            manifest = root / "manifest.json"
            command = [sys.executable, str(TOOL), "--input", str(self.sdk_source), "--dma-source", str(self.dma_source), "--output", str(output), "--manifest", str(manifest)]
            subprocess.run(command, check=True, capture_output=True, text=True)
            self.assertEqual(output.read_bytes(), self.generated)
            timestamp = output.stat().st_mtime_ns
            subprocess.run(command, check=True, capture_output=True, text=True)
            self.assertEqual(output.stat().st_mtime_ns, timestamp)
            provenance = json.loads(manifest.read_text())
            self.assertEqual(provenance["source_sha256"], PATCH.PINNED_SHA256)
            self.assertEqual(provenance["dma2d_scheduler_sha256"], PATCH.PINNED_DMA2D_SHA256)
            self.assertEqual(provenance["generated_sha256"], hashlib.sha256(self.generated).hexdigest())
            self.assertEqual(provenance["coefficients_q8"], [list(row) for row in PATCH.FULL_RANGE_COEFFICIENTS])
            self.assertFalse(provenance["byte_order_modified"])
            self.assertFalse(provenance["extra_pixel_copy"])
            # Copy the pinned source into a disposable SDK tree for misuse tests;
            # no test requests a write in the real SDK, even when rejection fails.
            fake_original = root / "sdk/components/esp_driver_jpeg/jpeg_decode.c"
            fake_original.parent.mkdir(parents=True)
            fake_original.write_bytes(self.source)
            wrong_output = root / "sdk/components/esp_driver_dma/jpeg_decode.c"
            wrong_manifest = root / "sdk/components/esp_lcd/manifest.json"
            link = root / "sdk-link"
            link.symlink_to(root / "sdk", target_is_directory=True)
            for destination, manifest_path, reason in (
                (fake_original, manifest, "must not overwrite"),
                (wrong_output, manifest, "entire SDK tree"),
                (root / "other/jpeg_decode.c", wrong_manifest, "entire SDK tree"),
                (root / "other/wrong_name.c", manifest, "original basename"),
                (link / "components/esp_driver_dma/jpeg_decode.c", manifest, "entire SDK tree"),
            ):
                with self.subTest(destination=destination, manifest=manifest_path):
                    result = subprocess.run([sys.executable, str(TOOL), "--input", str(fake_original), "--dma-source", str(self.dma_source), "--output", str(destination), "--manifest", str(manifest_path)], capture_output=True, text=True)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(reason, result.stderr)
            self.assertFalse(wrong_output.exists())
            self.assertFalse(wrong_manifest.exists())
            self.assertFalse((root / "other/jpeg_decode.c").exists())
            self.assertEqual(fake_original.read_bytes(), self.source)
            self.assertEqual(self.sdk_source.read_bytes(), self.source)
            changed_dma = root / "changed-dma2d.c"
            changed_dma.write_bytes(self.dma_source.read_bytes() + b"\n")
            refused_output = root / "scheduler-drift/jpeg_decode.c"
            refused_manifest = root / "scheduler-drift/manifest.json"
            result = subprocess.run([sys.executable, str(TOOL), "--input", str(fake_original), "--dma-source", str(changed_dma), "--output", str(refused_output), "--manifest", str(refused_manifest)], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("DMA2D scheduler SHA256 mismatch", result.stderr)
            self.assertFalse(refused_output.exists())
            self.assertFalse(refused_manifest.exists())

    def test_cmake_replaces_exactly_one_source_and_preserves_encoder(self) -> None:
        cmake = shutil.which("cmake")
        if not cmake:
            self.skipTest("host CMake unavailable")
        with tempfile.TemporaryDirectory(prefix="p4desk-jpeg-cmake-") as directory:
            root = Path(directory)
            encoder = root / "encoder.c"
            encoder.write_text("int encoder_is_preserved(void) { return 1; }\n")
            generated = root / "generated/jpeg_decode.c"
            generated.parent.mkdir()
            generated.write_bytes(self.generated)
            helper = COMPONENT / "cmake/replace_jpeg_source.cmake"
            for count in (1, 0, 2):
                project = root / f"project-{count}"
                project.mkdir()
                originals = " ".join(f'"{self.sdk_source}"' for _ in range(count))
                checks = f'''
get_target_property(sources __idf_esp_driver_jpeg SOURCES)
if(NOT "{generated}" IN_LIST sources OR "{self.sdk_source}" IN_LIST sources OR NOT "{encoder}" IN_LIST sources)
    message(FATAL_ERROR "incorrect decoder source swap")
endif()
get_target_property(includes __idf_esp_driver_jpeg INCLUDE_DIRECTORIES)
if(NOT "{self.sdk_source.parent}" IN_LIST includes)
    message(FATAL_ERROR "missing original private headers")
endif()
'''
                (project / "CMakeLists.txt").write_text(f'''
cmake_minimum_required(VERSION 3.19)
project(JpegPatchFixture C)
add_library(__idf_esp_driver_jpeg STATIC {originals} "{encoder}")
include("{helper}")
p4desk_jpeg_full_range_replace_source(__idf_esp_driver_jpeg "{self.sdk_source}" "{generated}")
{checks}
''')
                result = subprocess.run([cmake, "-S", str(project), "-B", str(project / "build")], capture_output=True, text=True)
                if count == 1:
                    self.assertEqual(result.returncode, 0, result.stderr)
                else:
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(f"found {count}", result.stderr)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--idf-path", type=Path)
    args, remaining = parser.parse_known_args()
    if args.idf_path:
        IDF_PATH = args.idf_path.expanduser().resolve()
    unittest.main(argv=[sys.argv[0], *remaining], verbosity=2)
