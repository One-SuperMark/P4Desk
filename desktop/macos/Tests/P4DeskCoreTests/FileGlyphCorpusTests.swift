import XCTest
@testable import P4DeskCore

final class FileGlyphCorpusTests: XCTestCase {
    func testPreviewBoundNamesAndPreviouslyUsedGlyphs() throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let url = directory.appendingPathComponent("中文🙃.txt")
        let body = Data("正文".utf8) + Data(repeating: 0x61, count: 32_768 - 6) + Data("尾".utf8)
        try body.write(to: url)
        let corpus = FileGlyphCorpus.collect(urls: [url], directoryNames: ["图片"], previous: "此前")
        for character in "中文🙃正文图片此前" { XCTAssertTrue(corpus.text.contains(character)) }
        XCTAssertFalse(corpus.text.contains("尾")); XCTAssertFalse(corpus.truncated)
        XCTAssertEqual(corpus.text.unicodeScalars.count, Set(corpus.text.unicodeScalars).count)
    }
    func testUTF8BoundaryDoesNotGenerateReplacementGlyphsAndBinaryIsSkipped() {
        let data = Data("abc中".utf8)
        XCTAssertEqual(FileGlyphCorpus.decodeUTF8Prefix(data.dropLast()), "abc")
        XCTAssertEqual(FileGlyphCorpus.decodeUTF8Prefix(data.dropLast(2)), "abc")
        XCTAssertNil(FileGlyphCorpus.decodeUTF8Prefix(Data([0, 65, 66])))
        XCTAssertNil(FileGlyphCorpus.decodeUTF8Prefix(Data([0xff, 65, 66, 67, 68])))
    }
    func testCorpusLimitIsReportedToPreserveInstalledFont() {
        let oversized = String(String.UnicodeScalarView((0x4e00..<(0x4e00 + 4_050)).compactMap(Unicode.Scalar.init)))
        let corpus = FileGlyphCorpus.collect(urls: [], directoryNames: [oversized], previous: "")
        XCTAssertTrue(corpus.truncated)
        XCTAssertEqual(corpus.text.unicodeScalars.count, 4_000)
    }
    func testCollectorDoesNotReadDirectoriesOrFollowSymlinks() throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let target = directory.appendingPathComponent("target.txt"), link = directory.appendingPathComponent("alias.txt")
        try "秘密".write(to: target, atomically: true, encoding: .utf8)
        try FileManager.default.createSymbolicLink(at: link, withDestinationURL: target)
        let corpus = FileGlyphCorpus.collect(urls: [directory, link], directoryNames: [], previous: "")
        XCTAssertFalse(corpus.text.contains("秘")); XCTAssertFalse(corpus.text.contains("密"))
        XCTAssertTrue(corpus.text.contains("alias.txt".first!))
    }
}
