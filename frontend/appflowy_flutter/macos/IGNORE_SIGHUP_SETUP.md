# 配置 Xcode 忽略 SIGHUP 信号

## 问题
在 Xcode 调试时，SIGHUP 信号会导致调试器中断并显示 `_sigtramp` 调用栈。

## 解决方案

### 方法 1：代码层面（已自动配置）✅
代码中已经在多个位置设置了忽略 SIGHUP 信号：
- 静态初始化器（模块加载时）
- AppDelegate 初始化时
- applicationDidFinishLaunching 时

### 方法 2：LLDB 配置（推荐手动配置）

#### 选项 A：在用户主目录创建 `.lldbinit`（全局生效）

在终端执行：
```bash
echo "process handle SIGHUP -n false -p false -s false" >> ~/.lldbinit
```

#### 选项 B：在 Xcode 中手动配置（仅当前项目）

1. 在 Xcode 中运行项目
2. 当调试器停在 `_sigtramp` 时，在 LLDB 控制台输入：
   ```
   process handle SIGHUP -n false -p false -s false
   ```
3. 然后继续运行（按 `Continue` 按钮）

#### 选项 C：创建符号断点（自动化）

1. 在 Xcode 中，打开 **Breakpoint Navigator** (⌘8)
2. 点击左下角的 **+** 按钮
3. 选择 **Symbolic Breakpoint**
4. 设置：
   - **Symbol**: `main`
   - **Action**: 选择 **Debugger Command**
   - **Command**: 输入 `process handle SIGHUP -n false -p false -s false`
   - 取消勾选 **Automatically continue after evaluating actions**
5. 点击 **Done**

这样每次启动调试时，LLDB 会自动执行命令忽略 SIGHUP 信号。

### 方法 3：在 Scheme 中配置环境变量

虽然 scheme 文件不支持直接执行 LLDB 命令，但可以通过环境变量触发。

## 验证配置

运行项目后，在 LLDB 控制台输入：
```
process handle SIGHUP
```

应该看到类似输出：
```
NAME         PASS   STOP   NOTIFY
===========  =====  =====  ======
SIGHUP       false  false  false
```

## 说明

- **-n false**: 不通知（notify = false）
- **-p false**: 不打印（print = false）
- **-s false**: 不停止（stop = false）

这样 SIGHUP 信号会被完全忽略，不会中断调试。

