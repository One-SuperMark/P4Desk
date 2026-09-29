#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Generate the project-local full-range BT.601 JPEG decoder correction.

The SDK source is read only. Its SHA and all ownership/configuration anchors
must match before any output is written. The original Apache-2.0 header stays.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys


PINNED_SHA256 = "fa4ce744bfaea32e9becbe93db14f65f8f9a4386e637dc83d031f7c3d194c488"
PINNED_DMA2D_SHA256 = "510355c193cc5c651e3b9158c1c38055b7a2e7a7df4d189e55530a9e54003f26"
PATCH_VERSION = 1
FULL_RANGE_COEFFICIENTS = (
    (256, 0, 359, -45952),
    (256, -88, -183, 34688),
    (256, 454, 0, -58112),
)
COEFFICIENT_WIDTHS = (10, 11, 10, 18)


def encode_coefficient(value: int, width: int) -> int:
    """Represent the signed Q8 coefficient using the register's low bits."""
    if not -(1 << (width - 1)) <= value < (1 << (width - 1)):
        raise ValueError(f"coefficient {value} does not fit signed {width}-bit field")
    return value & ((1 << width) - 1)


def render_helper() -> str:
    lines = [
        "// Project-local JPEG correction. The pinned decoder owns pool 0 RX0",
        "// here: RX_REORDER selects only RX0, and on_job_picked has acquired it.",
        "// Configure after the normal CSC setup, before either DMA channel starts.",
        "// The callback may run in a task or ISR; this helper allocates nothing",
        "// and makes only six volatile register writes. No group lock is held.",
        '_Static_assert(DMA2D_LL_INST_NUM == 1, "JPEG correction requires DMA2D pool 0 only");',
        '_Static_assert(DMA2D_LL_RX_CHANNEL_SUPPORT_RO_MASK == 1u, "JPEG RX_REORDER must select RX0 only");',
        '_Static_assert(DMA2D_LL_RX_CHANNEL_SUPPORT_CSC_MASK == 1u, "JPEG CSC must belong to RX0 only");',
        '_Static_assert(sizeof(dma2d_color_param_reg_t) == 8, "JPEG CSC parameter layout changed");',
        '_Static_assert(sizeof(dma2d_color_param_group_chn_reg_t) == 24, "JPEG CSC group layout changed");',
        "",
        "static void p4desk_jpeg_config_full_range_bt601(void)",
        "{",
        "    dma2d_dev_t *dev = DMA2D_LL_GET_HW(0);",
        "    dma2d_color_param_group_chn_reg_t params = {0};",
        "    // Q8 JFIF inverse BT.601. Encoded negative fields use two's complement.",
    ]
    for name, row in zip(("h", "m", "l"), FULL_RANGE_COEFFICIENTS):
        for field, value, width in zip(("a", "b", "c", "d"), row, COEFFICIENT_WIDTHS):
            encoded = encode_coefficient(value, width)
            lines.append(f"    params.param_{name}.{field} = {encoded}u; // {value} ({width} bits)")
    lines.append("")
    for name in ("h", "m", "l"):
        for word in range(2):
            lines.append(
                f"    dev->in_channel[0].in_color_param_group.param_{name}.val[{word}] = "
                f"params.param_{name}.val[{word}];"
            )
    lines.extend(("}", "", ""))
    return "\n".join(lines)


OWNERSHIP_ANCHORS = (
    ("pool_zero", "        .pool_id = 0,\n"),
    ("rx_reorder_requirement", "        .channel_flags = DMA2D_CHANNEL_FUNCTION_FLAG_RX_REORDER,\n"),
    (
        "one_tx_one_rx_transaction",
        "    dma2d_trans_config_t trans_desc = {\n"
        "        .tx_channel_num = 1,\n"
        "        .rx_channel_num = 1,\n",
    ),
    (
        "configure_before_start",
        "    jpeg_dec_config_dma_trans_ability(decoder_engine);\n"
        "    jpeg_dec_config_dma_csc(decoder_engine, rx_chan);\n\n"
        "    dma2d_rx_event_callbacks_t jpeg_dec_cbs = {\n"
        "        .on_recv_eof = jpeg_rx_eof,\n"
        "    };\n\n"
        "    dma2d_register_rx_event_callbacks(rx_chan, &jpeg_dec_cbs, decoder_engine);\n\n"
        "    dma2d_set_desc_addr(tx_chan, (intptr_t)decoder_engine->txlink);\n"
        "    dma2d_set_desc_addr(rx_chan, (intptr_t)decoder_engine->rxlink);\n"
        "    dma2d_start(tx_chan);\n"
        "    dma2d_start(rx_chan);\n"
        "    jpeg_ll_process_start(hal->dev);\n",
    ),
)

REPLACEMENTS = (
    (
        "dma_register_header",
        '#include "hal/jpeg_ll.h"\n',
        '#include "hal/jpeg_ll.h"\n#include "hal/dma2d_ll.h"\n',
    ),
    (
        "full_range_helper",
        "static void jpeg_dec_config_dma_csc(jpeg_decoder_handle_t decoder_engine, dma2d_channel_handle_t rx_chan)\n",
        render_helper()
        + "static void jpeg_dec_config_dma_csc(jpeg_decoder_handle_t decoder_engine, dma2d_channel_handle_t rx_chan)\n",
    ),
    (
        "scoped_matrix_override",
        "    dma2d_configure_color_space_conversion(rx_chan, &rx_csc);\n",
        "    dma2d_configure_color_space_conversion(rx_chan, &rx_csc);\n"
        "    // Only the JPEG BT.601 RGB output uses full-range JFIF coefficients.\n"
        "    // Preserve GRAY/YUV output, BT.709 and the original byte scramble.\n"
        "    if (decoder_engine->conv_std == JPEG_YUV_RGB_CONV_STD_BT601 &&\n"
        "        (decoder_engine->output_format == JPEG_DECODE_OUT_FORMAT_RGB565 ||\n"
        "         decoder_engine->output_format == JPEG_DECODE_OUT_FORMAT_RGB888)) {\n"
        "        p4desk_jpeg_config_full_range_bt601();\n"
        "    }\n",
    ),
)


def patch_source(source: bytes, *, verify_sha: bool = True) -> bytes:
    actual_sha = hashlib.sha256(source).hexdigest()
    if verify_sha and actual_sha != PINNED_SHA256:
        raise ValueError(f"JPEG source SHA256 mismatch: expected {PINNED_SHA256}, got {actual_sha}")
    text = source.decode("utf-8")
    if "p4desk_jpeg_config_full_range_bt601(" in text:
        raise ValueError("JPEG source already contains the full-range correction")
    for name, anchor in OWNERSHIP_ANCHORS:
        count = text.count(anchor)
        if count != 1:
            raise ValueError(f"ownership anchor {name!r} must occur exactly once; found {count}")
    for name, before, _ in REPLACEMENTS:
        count = text.count(before)
        if count != 1:
            raise ValueError(f"anchor {name!r} must occur exactly once; found {count}")
    for _, before, after in REPLACEMENTS:
        text = text.replace(before, after, 1)
    return text.encode("utf-8")


def write_if_changed(path: Path, contents: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists() or path.read_bytes() != contents:
        path.write_bytes(contents)


def verify_dma_source(source: bytes) -> None:
    # RX0 ownership depends on this scheduler's mask filtering/acquisition,
    # not just its version number. Check the actual CMake component source.
    actual_sha = hashlib.sha256(source).hexdigest()
    if actual_sha != PINNED_DMA2D_SHA256:
        raise ValueError(f"DMA2D scheduler SHA256 mismatch: expected {PINNED_DMA2D_SHA256}, got {actual_sha}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True, help="original SDK JPEG source, read only")
    parser.add_argument("--dma-source", type=Path, required=True, help="actual DMA2D scheduler source, read only")
    parser.add_argument("--output", type=Path, required=True, help="generated jpeg_decode.c under the build directory")
    parser.add_argument("--manifest", type=Path, help="optional machine-readable patch provenance")
    args = parser.parse_args()
    try:
        original = args.input.resolve(strict=True)
        output = args.output.resolve()
        if original.name != "jpeg_decode.c" or original.parent.name != "esp_driver_jpeg" or original.parent.parent.name != "components":
            raise ValueError("the input must be the pinned SDK components/esp_driver_jpeg/jpeg_decode.c")
        sdk_root = original.parents[2]
        if original == output:
            raise ValueError("the output must not overwrite the SDK input")
        if output == sdk_root or sdk_root in output.parents:
            raise ValueError("the output must be outside the entire SDK tree")
        if output.name != "jpeg_decode.c":
            raise ValueError("the output must retain the original basename jpeg_decode.c")
        if args.manifest:
            manifest_path = args.manifest.resolve()
            if manifest_path in (original, output) or manifest_path == sdk_root or sdk_root in manifest_path.parents:
                raise ValueError("the manifest must be separate from source and outside the entire SDK tree")
        source = original.read_bytes()
        verify_dma_source(args.dma_source.resolve(strict=True).read_bytes())
        patched = patch_source(source)
        manifest = {
            "idf_version": "6.0.2",
            "patch_version": PATCH_VERSION,
            "source_sha256": PINNED_SHA256,
            "dma2d_scheduler_sha256": PINNED_DMA2D_SHA256,
            "generated_sha256": hashlib.sha256(patched).hexdigest(),
            "source_bytes": len(source),
            "generated_bytes": len(patched),
            "anchors": [name for name, _, _ in REPLACEMENTS],
            "ownership_anchors": [name for name, _ in OWNERSHIP_ANCHORS],
            "source_license": "Apache-2.0",
            "scope": "JPEG BT.601 RGB565/RGB888 output only",
            "coefficients_q8": FULL_RANGE_COEFFICIENTS,
            "coefficient_field_bits": COEFFICIENT_WIDTHS,
            "channel": "pool 0 RX0, acquired in on_job_picked, before DMA start",
            "byte_order_modified": False,
            "extra_pixel_copy": False,
            "hardware_color_validation": "pending",
        }
        write_if_changed(output, patched)
        if args.manifest:
            write_if_changed(args.manifest, (json.dumps(manifest, indent=2) + "\n").encode("utf-8"))
        print(f"JPEG full-range generated: {manifest['generated_sha256']} ({len(patched)} bytes)")
        return 0
    except (OSError, UnicodeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
