import AppKit
import CoreGraphics

// Detect whether tmux is running inside a terminal window that is fullscreen
// on a notched MacBook display. Prints "1" if so, "0" otherwise.
//
// Notched detection: NSScreen.safeAreaInsets.top > 0 — only notched displays
// (built-in MBP 14"/16" 2021+) have a non-zero top inset.
//
// Fullscreen detection: enumerate on-screen windows, find ones owned by a
// known terminal app whose bounds match a notched screen's size (within
// tolerance). This catches both native and non-native fullscreen modes
// without requiring accessibility permissions, and works regardless of
// which app is currently frontmost.

let notchedScreens = NSScreen.screens.filter { $0.safeAreaInsets.top > 0 }
if notchedScreens.isEmpty {
    print("0")
    exit(0)
}

let opts: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
guard let windows = CGWindowListCopyWindowInfo(opts, kCGNullWindowID) as? [[String: Any]] else {
    print("0")
    exit(0)
}

let terminals: Set<String> = ["ghostty", "wezterm"]

for w in windows {
    guard let layer = w[kCGWindowLayer as String] as? Int, layer == 0,
          let owner = w[kCGWindowOwnerName as String] as? String,
          terminals.contains(owner.lowercased()),
          let b = w[kCGWindowBounds as String] as? [String: CGFloat] else { continue }
    let width = b["Width"] ?? 0
    let height = b["Height"] ?? 0
    for screen in notchedScreens {
        let s = screen.frame.size
        if abs(width - s.width) < 2 && abs(height - s.height) < 2 {
            print("1")
            exit(0)
        }
    }
}

print("0")
