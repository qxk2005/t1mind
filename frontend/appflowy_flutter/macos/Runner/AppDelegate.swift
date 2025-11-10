import Cocoa
import FlutterMacOS
import Darwin

// 在模块加载时立即忽略 SIGHUP 信号（静态初始化器）
private let _ignoreSIGHUP: Void = {
    signal(SIGHUP, SIG_IGN)
}()

@main
class AppDelegate: FlutterAppDelegate {
  // 在类初始化时立即忽略 SIGHUP 信号
  override init() {
    // 触发静态初始化器
    _ = _ignoreSIGHUP
    // 在应用启动的最早阶段忽略 SIGHUP 信号
    signal(SIGHUP, SIG_IGN)
    super.init()
  }
  
  override func applicationDidFinishLaunching(_ notification: Notification) {
    // 确保信号被忽略（双重保险）
    signal(SIGHUP, SIG_IGN)
    super.applicationDidFinishLaunching(notification)
  }

  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    return false
  }

  override func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
    if !flag {
        for window in sender.windows {
            window.makeKeyAndOrderFront(self)
        }
    }

    return true
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }
}
