// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "P4Desk",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "P4Desk", targets: ["P4Desk"])],
    targets: [
        .target(name: "P4DeskNative", publicHeadersPath: "include", cxxSettings: [.unsafeFlags(["-fobjc-arc"])],
                linkerSettings: [.linkedFramework("AppKit"), .linkedFramework("CoreGraphics"),
                                 .linkedFramework("IOKit"), .linkedFramework("IOUSBHost")]),
        .target(name: "P4DeskCore", linkerSettings: [.linkedLibrary("sqlite3")]),
        .executableTarget(name: "P4Desk", dependencies: ["P4DeskCore", "P4DeskNative"],
                          linkerSettings: [.linkedFramework("ScreenCaptureKit"), .linkedFramework("VideoToolbox"),
                                           .linkedFramework("ImageIO"), .linkedFramework("ApplicationServices")]),
        .testTarget(name: "P4DeskCoreTests", dependencies: ["P4DeskCore"])
    ],
    cxxLanguageStandard: .cxx17
)
