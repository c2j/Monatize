# Monazite Phase‑0 总结

## 范围与目标
- 目标：在 Linux 上打通 Web 内容 → 像素缓冲（RGBA8）→ UDS → GPUI（winit + wgpu）显示与验证的闭环。
- 范围：以最小可行实现为主，优先稳定性与端到端可验证；零拷贝、跨平台与完善的端口抽象留到后续阶段。

## 交付概览（P1–P9）
- P1 wpe-sys
  - 提供最小 skeleton 与 `selected_port()`；单测通过。
- P2 gpui-wpe-bridge
  - `ExternalFrame` 基本结构与事件翻译 helper（resize/mouse/key/wheel）；稳定性单测通过。
- P3 min-web-process
  - 默认 headless 路径：生成红色 RGBA8 帧并经 UDS 发送；
  - 可选 WebKitGTK 路径（feature: `port_gtk`）：GTK OffscreenWindow + WebKit2GTK 离屏渲染，pixbuf 同步抓帧；E2E 验证通过。
- P4 gpui-app-host
  - winit 0.30 + wgpu 27，接收帧上传纹理并渲染，支持截图与像素校验；E2E 验证通过。
- P5 event-packet
  - InputEvent/Message + serde/bincode 序列化；单测通过。
- P6 mock-network
  - 静态/交替 HTML 生成（为未来阶段预留）；单测通过。
- P7 texture-verify
  - RGBA8 像素近似比对工具；单测通过。
- P8 build-strap（xtask）
  - `sys-deps` 探针、`dist-0` 打包、`dist-gtk` 打包、新增 `demo` 一键演示。
- P9 smoke-0（scripts/smoke-0.py）
  - 端到端验收脚本；在 10 秒内完成截图比对并输出 PASS。

## 快速验证（本地）
1) 单元测试
```
cargo test --all
```
期望：所有 crate 通过。

2) E2E（默认 headless 路径）
```
python3 scripts/smoke-0.py
```
期望关键输出：
- FRAME 64x64 OK
- SCREENSHOT OK
- QUIT
- smoke-0 PASS

3) 一键演示（Xvfb）
```
# Headless（默认）
cargo run -p xtask -- demo

# WebKitGTK（需安装 dev 包，见“系统依赖”）
cargo run -p xtask -- demo gtk
```
GTK 模式期望关键输出：
- SCREENSHOT OK
- FRAME 64x64 OK
- QUIT

## 打包产物
- 默认打包（headless 演示）：
```
cargo run -p xtask -- dist-0
ls -la dist/phase-0
```
包含：`gpui-app-host`、`min-web-process`、`smoke-0.py`

- WebKitGTK 演示打包：
```
cargo run -p xtask -- dist-gtk
ls -la dist/phase-0-gtk
```
包含：`gpui-app-host`、`min-web-process`（启用 `port_gtk` 构建）、`smoke-0.py`

## 系统依赖
- 基础工具：`pkg-config`、`cmake`、`python3`、`xvfb-run`（推荐）。
- WebKitGTK（仅在 GTK 模式或 dist-gtk 构建时需要）：
  - Debian/Ubuntu：`sudo apt-get install -y libwebkit2gtk-4.0-dev libgtk-3-dev`
  - Fedora：`sudo dnf install -y webkit2gtk3-devel gtk3-devel`
  - Arch：`sudo pacman -S --needed webkit2gtk gtk3`
- 可运行 `cargo run -p xtask -- sys-deps` 查看提示。

## 单测与集成测试
- 单测覆盖：P2/P5/P6/P7/P1 等核心库均有单测；
- 集成测试（最小 Xvfb 验证）：
  - 在 `gpui-app-host` 与 `min-web-process` 各有 1 个最小化的 Xvfb 集成测试；
  - 自动检查 `xvfb-run` 与 `python3`，缺失则跳过（不失败）；
  - 实际调用 `scripts/smoke-0.py` 跑端到端帧传输、渲染与截图比对。

## 关键技术点回顾
- IPC：UDS（seqpacket）+ 长度前缀帧（u32 LE + bincode）。
- 渲染：winit 0.30 `ApplicationHandler` + wgpu 27；纹理上传 + fullscreen quad。
- 截图比对：`copy_texture_to_buffer` → CPU → 颜色近似比对；`bytes_per_row` 256 对齐已处理。
- WebKitGTK 离屏：GTK3 OffscreenWindow + WebView；`LoadEvent::Finished` 后延迟 ~120ms 保证绘制完成；`offscreen.pixbuf()` 同步抓帧；RGB→RGBA8 统一。

## 已知限制（Phase‑0 允许的范围）
- P1 为最小 skeleton，未做大规模 FFI 生成与符号哈希锁定；
- 仅 Linux（X11 优先）验证；
- 走 CPU copy 路径，零拷贝与跨进程共享纹理留待后续；
- 错误处理与日志仅满足 PoC 需求；
- WebKitGTK 路径需系统 dev 库，CI/默认流程仍以 headless 为主。

## 下一步建议（Phase‑1 候选）
- 完善端口抽象（WPE/GTK 可配置）与 FFI 绑定；
- 引入零拷贝/共享内存或 DMA‑BUF 路径；
- 增强窗口与输入事件打通（交互/滚动/IME）；
- 将 E2E 集成入 CI（xvfb-run）并产出工件；
- 增强日志与指标（帧率、复制耗时等）。

---
如需我将本摘要中的命令加入到项目根 README 或设计文档目录的索引中，请告知。
