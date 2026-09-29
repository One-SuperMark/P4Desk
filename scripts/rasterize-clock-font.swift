// Generate alpha8 clock digits using the font's real OpenType tabular figures.
// macOS only; normal firmware builds embed the resulting atlas directly.
import Foundation
import CoreText
import CoreGraphics

let characters = Array("0123456789-")
let cellWidth = 144
let cellHeight = 208
let minimumMargin = 3
let scratchSize = 1024
let anchor = 512

struct Bounds {
    var left: Int
    var top: Int
    var right: Int
    var bottom: Int
    var width: Int { right - left }
    var height: Int { bottom - top }
    var array: [Int] { [left, top, right, bottom] }
}

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data((message + "\n").utf8))
    exit(1)
}

func makeFont(_ base: CTFontDescriptor, size: Int) -> CTFont {
    // AAT selectors are mapped by CoreText to the OpenType tnum and lnum tags.
    let settings: [[String: Any]] = [
        [kCTFontFeatureTypeIdentifierKey as String: 6,
         kCTFontFeatureSelectorIdentifierKey as String: 0],
        [kCTFontFeatureTypeIdentifierKey as String: 21,
         kCTFontFeatureSelectorIdentifierKey as String: 1],
    ]
    let descriptor = CTFontDescriptorCreateCopyWithAttributes(
        base, [kCTFontFeatureSettingsAttribute as String: settings] as CFDictionary
    )
    return CTFontCreateWithFontDescriptor(descriptor, CGFloat(size), nil)
}

func shape(_ character: Character, font: CTFont) -> (CGGlyph, Double) {
    let attributed = NSAttributedString(
        string: String(character),
        attributes: [NSAttributedString.Key(kCTFontAttributeName as String): font]
    )
    let runs = CTLineGetGlyphRuns(CTLineCreateWithAttributedString(attributed)) as! [CTRun]
    guard runs.count == 1, let run = runs.first, CTRunGetGlyphCount(run) == 1 else {
        fail("Each clock character must shape to exactly one glyph without fallback.")
    }
    guard let fontValue = (CTRunGetAttributes(run) as NSDictionary)[kCTFontAttributeName],
          CFGetTypeID(fontValue as CFTypeRef) == CTFontGetTypeID() else {
        fail("Missing shaped font.")
    }
    let runFont = fontValue as! CTFont
    guard CFEqual(CTFontCopyPostScriptName(runFont), CTFontCopyPostScriptName(font)) else {
        fail("Unexpected fallback font.")
    }
    var glyph: CGGlyph = 0
    var advance = CGSize.zero
    CTRunGetGlyphs(run, CFRange(location: 0, length: 1), &glyph)
    CTRunGetAdvances(run, CFRange(location: 0, length: 1), &advance)
    return (glyph, advance.width)
}

func inkBounds(_ bytes: [UInt8], width: Int, height: Int) -> Bounds {
    var result = Bounds(left: width, top: height, right: 0, bottom: 0)
    for y in 0..<height {
        for x in 0..<width where bytes[y * width + x] != 0 {
            result.left = min(result.left, x)
            result.top = min(result.top, y)
            result.right = max(result.right, x + 1)
            result.bottom = max(result.bottom, y + 1)
        }
    }
    guard result.right > result.left, result.bottom > result.top else {
        fail("Clock glyph contains no ink.")
    }
    return result
}

func raster(_ font: CTFont, glyph: CGGlyph) -> [UInt8] {
    guard let path = CTFontCreatePathForGlyph(font, glyph, nil) else {
        fail("Clock glyph has no outline.")
    }
    var bytes = [UInt8](repeating: 0, count: scratchSize * scratchSize)
    bytes.withUnsafeMutableBytes { storage in
        guard let context = CGContext(
            data: storage.baseAddress, width: scratchSize, height: scratchSize,
            bitsPerComponent: 8, bytesPerRow: scratchSize,
            space: CGColorSpaceCreateDeviceGray(), bitmapInfo: CGImageAlphaInfo.none.rawValue
        ) else { fail("Cannot allocate grayscale font rasterizer.") }
        context.setShouldAntialias(true)
        context.setAllowsAntialiasing(true)
        context.setFillColor(gray: 1, alpha: 1)
        context.translateBy(x: CGFloat(anchor), y: CGFloat(anchor))
        context.addPath(path)
        context.fillPath()
    }
    return bytes
}

guard CommandLine.arguments.count == 3 else {
    fail("Usage: swift scripts/rasterize-clock-font.swift FONT.ttf OUTPUT.alpha")
}
let sourceURL = URL(fileURLWithPath: CommandLine.arguments[1])
guard let descriptors = CTFontManagerCreateFontDescriptorsFromURL(sourceURL as CFURL) as? [CTFontDescriptor],
      let descriptor = descriptors.first else { fail("Cannot load source font.") }

let measurement = makeFont(descriptor, size: 1000)
let measured = characters.map { shape($0, font: measurement) }
let digitAdvances = Array(measured.prefix(10)).map { $0.1 }
guard digitAdvances.max()! - digitAdvances.min()! < 0.000001 else {
    fail("The selected tabular lining digit glyphs do not have equal advances.")
}
let paths = measured.map { CTFontCreatePathForGlyph(measurement, $0.0, nil)!.boundingBoxOfPath }
let maxWidth = paths.map { $0.width }.max()!
let upper = paths.prefix(10).map { $0.maxY }.max()!
let lower = paths.prefix(10).map { $0.minY }.min()!
var fontSize = Int(min(
    Double(cellWidth - minimumMargin * 2) / maxWidth,
    Double(cellHeight - minimumMargin * 2) / (upper - lower)
) * 1000)

var font: CTFont!
var glyphs: [(CGGlyph, Double)] = []
var scratch: [[UInt8]] = []
var bounds: [Bounds] = []
while fontSize > 1 {
    font = makeFont(descriptor, size: fontSize)
    glyphs = characters.map { shape($0, font: font) }
    scratch = glyphs.map { raster(font, glyph: $0.0) }
    bounds = scratch.map { inkBounds($0, width: scratchSize, height: scratchSize) }
    let top = bounds.prefix(10).map { $0.top }.min()!
    let bottom = bounds.prefix(10).map { $0.bottom }.max()!
    if bounds.map({ $0.width }).max()! <= cellWidth - minimumMargin * 2,
       bottom - top <= cellHeight - minimumMargin * 2 { break }
    fontSize -= 1
}
guard fontSize > 1 else { fail("Cannot fit complete glyphs into the clock atlas.") }
let top = bounds.prefix(10).map { $0.top }.min()!
let bottom = bounds.prefix(10).map { $0.bottom }.max()!
let shiftY = (cellHeight - (bottom - top)) / 2 - top
var atlas = Data()
var glyphMetadata: [[String: Any]] = []
for index in characters.indices {
    let sourceBounds = bounds[index]
    let shiftX = (cellWidth - sourceBounds.width) / 2 - sourceBounds.left
    // Flip cards center each visible numeral on their hinge rather than on a
    // text baseline. Lining figures can still have unequal optical overshoots.
    let opticalShiftY = (cellHeight - sourceBounds.height) / 2 - sourceBounds.top
    var cell = [UInt8](repeating: 0, count: cellWidth * cellHeight)
    for y in sourceBounds.top..<sourceBounds.bottom {
        for x in sourceBounds.left..<sourceBounds.right {
            let alpha = scratch[index][y * scratchSize + x]
            let destinationX = x + shiftX
            let destinationY = y + opticalShiftY
            guard (0..<cellWidth).contains(destinationX), (0..<cellHeight).contains(destinationY) else {
                if alpha != 0 { fail("Clock glyph would be clipped.") }
                continue
            }
            cell[destinationY * cellWidth + destinationX] = alpha
        }
    }
    let box = inkBounds(cell, width: cellWidth, height: cellHeight)
    let margins = [box.left, box.top, cellWidth - box.right, cellHeight - box.bottom]
    guard margins.min()! >= minimumMargin else { fail("Clock glyph lacks a transparent margin.") }
    let center = Double(box.top + box.bottom) / 2
    if index < 10, abs(center - 104) > 1.5 {
        fail("Digit \(characters[index]) ink bounds \(box.array) center \(center) is not centered on the flip hinge.")
    }
    atlas.append(contentsOf: cell)
    glyphMetadata.append([
        "character": String(characters[index]), "glyph_id": glyphs[index].0,
        "advance": glyphs[index].1, "bounds": box.array, "margins": margins,
        "ink_center_y": center, "pen_x": anchor + shiftX,
        "baseline": anchor + opticalShiftY, "optical_y_adjustment": opticalShiftY - shiftY,
    ])
}
let outputURL = URL(fileURLWithPath: CommandLine.arguments[2])
try FileManager.default.createDirectory(at: outputURL.deletingLastPathComponent(), withIntermediateDirectories: true)
try atlas.write(to: outputURL, options: .atomic)
let metadata: [String: Any] = [
    "font_name": CTFontCopyFullName(font!), "postscript_name": CTFontCopyPostScriptName(font!),
    "font_size": fontSize, "characters": String(characters), "width": cellWidth,
    "height": cellHeight, "bytes": atlas.count, "nominal_baseline": anchor + shiftY,
    "features": ["tnum", "lnum"], "digit_advance_units": digitAdvances[0] * Double(CTFontGetUnitsPerEm(measurement)) / 1000,
    "numeric_ink_center_y": Double(top + bottom) / 2 + Double(shiftY),
    "glyphs": glyphMetadata, "rasterizer": "macOS CoreText outlines / CoreGraphics grayscale",
]
let metadataData = try JSONSerialization.data(withJSONObject: metadata, options: [.prettyPrinted, .sortedKeys])
try metadataData.write(to: outputURL.appendingPathExtension("json"), options: .atomic)
print(String(data: metadataData, encoding: .utf8)!)
