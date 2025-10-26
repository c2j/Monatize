# Monazite Phase‑1 总结报告

版本：v0.2.0‑p1  • 日期：YYYY‑MM‑DD  • 负责人：Monazite 团队

---

## 1. 交付概览（达成情况）
- 三进程架构跑通：browser‑main（主控）、content‑srv（网页渲染）、gpu‑srv（GPU 合成），并补充 network‑srv、ai‑runtime 模块，满足“能打开网页，AI 能回答，DOM 能注入”。
- 真实渲染链路落地：采用 WebKitGTK 离屏快照与窗口显示两种模式；CI/E2E 使用离屏 PNG 验证。
- 内置 GUI 主窗口（默认）：browser‑main 直接创建 GTK 窗口嵌入 WebKit WebView，新增“最小工具栏（后退/前进/刷新）+ 可见徽标（DOM 注入确认）”。
- E2E 冒烟通过：python ci/smoke.py 在 10s 内完成，摘要包含 “Example Domain”，同时生成有效 PNG 截图。
- 模块可独立编译/运行/测试，互不阻塞；Mock/Fallback 路径保留。

## 2. 架构与实现要点
- 真实渲染优先：
  - content‑srv 使用 WebKitGTK 实现两条路径：
    1) Offscreen（无头）渲染 + PNG 截图（供 CI/脚本使用）
    2) GUI 窗口展示（供本地交互）
  - browser‑main 默认内置 GUI 路径（检测 DISPLAY/WAYLAND_DISPLAY），也可显式 `--no-gui` 走无头路径或 `--real-screenshot` 触发 content‑srv 截图。
- DOM 注入：页面 LoadFinished 后注入 JS，在 documentElement 标记 `data-monazite="1"`；GUI 模式下额外插入右上角“Monazite”徽标，肉眼可见，避免“看起来是白屏”。
- 进程边界：
  - browser‑main 调用 content‑srv 二进制执行真实渲染（子进程），优先使用本仓构建产物 `target/debug|release/content-srv`，避免 PATH 中旧版本。
  - network‑srv/ai‑runtime 保持独立，可单独运行与测试。
- 兼容 Servo‑lite：
  - 作为演示/测试的 DL（DisplayList）生成与 CPU 光栅化仍保留（`--html` fallback），但 Phase‑1 主路径使用 WebKit 实际渲染。

## 3. 模块完成度清单（M1–M12）
- M1 ipc‑channel：已实现并单测通过（可靠通道抽象，后续零拷贝 Attachment 扩展点预留）
- M2 message‑defs：IDL→Rust 生成链路可用（round‑trip/哈希稳定性单测）
- M3 network‑srv：HTTP 下载（rustls），mock/真实均可测试
- M4 gpu‑srv：wgpu 最小渲染与截图（占位可视化/像素验证）
- M5 content‑srv：WebKitGTK 离屏 + 窗口渲染、截图、DOM 注入
- M6 ai‑runtime：AI 摘要 mock 路径（CPU），接口与错误码对齐
- M7 browser‑main：主控编排；默认 GUI 主窗口；可触发真实截图；可拉起分进程窗口
- M8 dom‑embed：注入库与过滤策略落地到最小可用集
- M9 window‑ai‑api：WASM stub（与 Phase‑1 mock AI 打通）
- M10 servo‑lite：极简布局引擎，仅用于测试/占位
- M11 build‑ops（xtask）：dist/test/fmt‑lint 基础命令
- M12 smoke‑test：端到端脚本，校验真实渲染 PNG 与摘要文本

## 4. 运行与验证（关键命令）
- 默认 GUI 主窗口（带工具栏与徽标）：
  ```bash
  cargo run -p browser-main -- https://example.com
  ```
- 无头真实截图（browser-main 触发 content‑srv）：
  ```bash
  cargo run -p browser-main -- https://example.com --real-screenshot /tmp/page.png
  ```
- 分进程窗口（可选，非阻塞）：
  ```bash
  cargo run -p browser-main -- https://example.com --show
  ```
- 直接验证 content‑srv：
  ```bash
  # 窗口显示
  cargo run -p content-srv -- --url https://example.com --show --width 1280 --height 800
  # 无头截图
  cargo run -p content-srv -- --url https://example.com --screenshot /tmp/page.png
  ```
- E2E 冒烟：
  ```bash
  python3 ci/smoke.py
  ```

## 5. UI 与交互（最小集）
- 工具栏：后退/前进/刷新；按钮随 `can_go_back/can_go_forward` 自动启用/禁用。
- 徽标：右上角“Monazite”浮层与样式，通过 JS 注入；重复加载不会重复插入。
- 降级：无图形环境（无 DISPLAY/WAYLAND）自动不弹窗；建议改用 `--real-screenshot`。

## 6. 系统依赖与构建
- 系统包（Debian/Ubuntu）：
  - `libwebkit2gtk-4.0-dev libgtk-3-dev libglib2.0-dev libgdk-pixbuf2.0-dev libsoup2.4-dev`
- Rust 依赖（关键）：
  - `webkit2gtk = "0.18"`、`gtk = "0.15"`、`gdk-pixbuf = "0.15"`、`glib = "0.15"`、`gio = "0.15"`
  - 其余：`tokio`、`wgpu`、`reqwest(rustls)`、`clap`、`anyhow/thiserror` 等

## 7. 已知问题与限制
- 仅优先支持 Linux（GTK3/WebKitGTK）；Windows 尚未启用 GUI 路径（已在设计中预留 sys 隔离）。
- 性能指标为目标值，尚未系统化 Benchmark（Phase‑2 进行专项压测与画像）。
- 真实渲染窗口当前为最小 UI；地址栏/快捷键/多标签页未纳入 Phase‑1 范畴。

## 8. 里程碑达成与指标
- Smoke（M12）：PASS（摘要含 “Example Domain” + 有效 PNG）。
- Dist（M11）：能产出核心二进制（browser-main、content-srv、gpu-srv、network-srv、ai-runtime）。
- 单测：各模块基础单测覆盖（详见各 crate）；E2E 冒烟走真实渲染。

## 9. 重要决策与经验
- 放弃“mock 渲染作为主路径”，转为“WebKit 真实渲染优先”；servo‑lite 仅作测试/占位。
- 默认 GUI 整合至 browser‑main，用户只看到一个主窗口；同时保留 content‑srv 用于 CI/隔离。
- 子进程二进制优先取本地构建产物，避免 PATH 版本漂移导致参数不兼容（如 `--show`）。
- 注入策略显式可视化（徽标）以降低“白屏/无反馈”的体验风险。

## 10. 下一步建议（Phase‑2 准备）
- UI/交互：地址栏与快捷键；基本导航状态展示；可配置主页/书签。
- 渲染侧：进程间帧传输优化（DMABUF/共享内存），减少 CPU 拷贝；Windows 支持计划。
- AI 能力：接入实际小模型（GGUF/ONNX），构建本地推理性能基线与缓存策略。
- 安全与隔离：完善 DOM 注入白名单与策略；引入站点隔离初步机制（设计与开关）。
- 测试与性能：
  - 构建更系统的基准脚本（首开耗时、截图耗时、AI 延迟指标）；
  - CI 分层（单测/集成/E2E）稳定运行与报告。

---
文档维护者：Monazite 团队（如需同步更多细节，参见 `design_phase-1/phase-1-design-spec.md` 与 `req/phase-1.md`）

