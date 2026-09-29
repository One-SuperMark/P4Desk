import Foundation

public enum FontResources {
    /// Migrate the old app-bundled default, preserving externally selected fonts.
    public static func resolvePath(savedPath: String?, bundledURL: URL?) -> String {
        let bundledPath = bundledURL?.path ?? ""
        guard let savedPath, !savedPath.isEmpty else { return bundledPath }
        let components = URL(fileURLWithPath: savedPath).pathComponents
        let legacyResource = ["P4Desk.app", "Contents", "Resources", "NotoSansSC-Regular.otf"]
        if components.suffix(4).elementsEqual(legacyResource) { return bundledPath }
        return savedPath
    }
}
