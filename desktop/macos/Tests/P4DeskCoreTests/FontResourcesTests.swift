import XCTest
@testable import P4DeskCore

final class FontResourcesTests: XCTestCase {
    private let bundled = URL(fileURLWithPath: "/Applications/P4Desk.app/Contents/Resources/HarmonyOS_Sans_SC_Regular.ttf")

    func testNewAndEmptyPreferencesUseCurrentBundledFont() {
        XCTAssertEqual(FontResources.resolvePath(savedPath: nil, bundledURL: bundled), bundled.path)
        XCTAssertEqual(FontResources.resolvePath(savedPath: "", bundledURL: bundled), bundled.path)
    }

    func testOldAppBundledDefaultMigratesAcrossInstallLocations() {
        for prefix in ["/Applications", "/tmp/build"] {
            let old = prefix + "/P4Desk.app/Contents/Resources/NotoSansSC-Regular.otf"
            XCTAssertEqual(FontResources.resolvePath(savedPath: old, bundledURL: bundled), bundled.path)
        }
    }

    func testExternalFontSelectionIsPreservedEvenWithLegacyFilename() {
        for path in ["/tmp/Custom.ttf", "/tmp/NotoSansSC-Regular.otf"] {
            XCTAssertEqual(FontResources.resolvePath(savedPath: path, bundledURL: bundled), path)
        }
    }

    func testMissingBundleProducesExplicitEmptyPathOnlyForDefault() {
        XCTAssertEqual(FontResources.resolvePath(savedPath: nil, bundledURL: nil), "")
        XCTAssertEqual(FontResources.resolvePath(savedPath: "/tmp/Custom.ttf", bundledURL: nil), "/tmp/Custom.ttf")
    }
}
