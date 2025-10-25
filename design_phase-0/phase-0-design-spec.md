# Monazite 阶段 0 详细设计规格书

> 本文对照 req/phase-0.md 要求，并采用 design/design_specs-template.md 的结构裁剪而来，指导阶段 0（4 周）研发落地。“阶段 0 只做一件事：把 WebKit ⇄ GPUI 的最小闭环跑通”，其他均以 Mock/空壳支撑，但每个 crate 必须可独立 `cargo test` 并通过 CI。

---

## 1. 项目概述

### 1.1 项目基本信息
- 项目名称：Monazite 浏览器（阶段 0）
- 项目代号：monazite-p0
- 项目类型：桌面应用（PoC）
- 开发语言：Rust（少量绑定到 WebKit Port C/C++）
- 目标平台：Linux（阶段 0 仅限）
- 版本标识：p0
- 预期发布日期：自立项起 4 周内

### 1.2 背景与目标
- 背景：在 Rust/GPUI 基础上集成 WebKit（按 OS Port），验证浏览器渲染核心“引擎→纹理→UI”的闭环可行性。
- 主要目标：
  - WebKit（Linux Port：WebKitGTK 或 WPE）输出帧 → GPUI/wgpu 纹理采样显示（窗口内）
  - 事件通路：GPUI 输入事件 → 翻译 → WebKit Port 注入 → 页面响应
  - E2E 验收：Hello WebKit 截图（红/蓝块）+ 单元测试报告
- 成功标准（阶段 0）：
  - 功能：`gpui-app-host` 弹出窗口正确显示 `min-web-process` 的网页帧
  - 性能：Headless 首帧 ≤ 3 s，窗口端到端输入回显 ≤ 200 ms（非严格 SLO）
  - 稳定性：长时间循环不 crash（≥ 10 分钟）

### 1.3 范围与约束
- 包含：P1–P9 九个模块（见下文），各自可 `cargo test`，提供最小 CLI/示例
- 排除（阶段 0 不做）：真实网络栈、JS 引擎完整性、GPU 零拷贝强制、跨平台
- 假设：Linux 桌面具备 Vulkan/合适 GPU 驱动；可使用 Xvfb/headless
- 约束：
  - Port 选择：优先 WebKitGTK，可替换 WPE；API 层用特征门隔离（`port_gtk`/`port_wpe`）
  - 纹理共享：以“可拷贝回退”为默认路径；零拷贝（EGL 外部纹理/DMABUF）作为可选实验

---

## 2. 需求分析

### 2.1 功能需求（核心）
| 模块 | 功能描述 | 优先级 | 验收标准 |
|---|---|---|---|
| P1 wpe-sys | WebKit Port FFI 绑定（无业务逻辑） | 高 | `cargo test` 加载符号成功，版本校验，头文件哈希锁定 |
| P2 gpui-wpe-bridge | 纹理句柄包装 + 事件序列化 | 高 | mock 句柄 round-trip、键码 100 次零误差、异常返回 Err |
| P3 min-web-process | 内嵌 WebKit（GTK/WPE），输出帧 | 高 | headless 加载 data:html 红块，纹理哈希正确；无效 URL 不崩溃 |
| P4 gpui-app-host | GPUI 窗口采样外部纹理 | 高 | 窗口创建稳定；尺寸变化 UV 同步；CI 可截图比对 |
| P5 event-packet | 输入事件 IDL + serde/bincode | 中 | 往返序列化稳定；未知变体不 panic |
| P6 mock-network | 返回静态 HTML（红/蓝块） | 中 | 两次返回不同颜色；并发 100 请求无数据竞争 |
| P7 texture-verify | 像素比对库（下载到 CPU） | 中 | 红 PASS/蓝 FAIL 用例覆盖；4K 纹理下载 < 80 ms |
| P8 build-strap | 阶段 0 构建脚本/依赖探测 | 中 | 幂等、离线提示友好 |
| P9 smoke-0 | 阶段 0 E2E 验收脚本 | 高 | 10 s 内得到红/蓝方块截图并像素比对 PASS |

### 2.2 非功能需求
- 性能（阶段 0 目标）：
  - 首帧（headless）≤ 3 s；窗口绘制帧间隔稳定（非卡死即可）
  - 文本/颜色块页面：渲染正确优先于 FPS
- 可靠性：
  - 子流程失败可检测并退出非 0；重复启动幂等
  - 资源释放：纹理/句柄/管道关闭无泄漏（粗略）
- 兼容性：Linux 桌面（X11/Wayland 任一，优先 X11）
- 安全性：无真实网络；仅本地 IPC；避免写系统敏感路径
- 可维护性：模块化 crate；错误用 anyhow/thiserror；日志 tracing

### 2.3 目标用户
- 内部研发与 CI（非面向终端用户），用于验证阶段 1 之前的引擎/合成关键路径

---

## 3. 技术架构设计

### 3.1 技术选型
- GUI/渲染：GPUI + wgpu（Zed 衍生），窗口与 quad 渲染
- Web 引擎：WebKitGTK 或 WPE（二选一，由 feature gate 控制）
- IPC：
  - 进程内：`crossbeam-channel`/`tokio::mpsc`
  - 进程间（阶段 0）：Unix Domain Socket（seqpacket）+ 长度前缀帧；默认传 CPU 像素 buffer；预留 SCM_RIGHTS（未来零拷贝）
- 序列化：`serde` + `bincode`
- 日志：`tracing` + `tracing-subscriber`
- 测试：内置单测 + `pytest`（smoke-0）+ `xvfb-run`
- 错误处理：`anyhow`/`thiserror`

选择理由（阶段 0）：实现成本最低、可在 4 周内达成闭环；零拷贝复杂度延后，优先“可拷贝回退”保证可交付。

### 3.2 系统架构

#### 3.2.1 整体数据/事件流
- 显示流：
  1) P3 min-web-process 通过 WebKit Port 渲染页面帧
  2) 将帧导出为 CPU 像素 buffer（RGBA8）→ 通过 UDS 发送至 P4
  3) P4 gpui-app-host 接收 → 上传至 wgpu 纹理 → quad 显示
- 输入流：
  1) GPUI 捕获键盘/鼠标事件 → P2 翻译为 Port 输入事件 IDL
  2) 通过 UDS 发送到 P3 → 调用 WebKit Port 注入事件

#### 3.2.2 模块划分与依赖
| 模块 | 职责 | 依赖 |
|---|---|---|
| P1 wpe-sys | WebKit Port FFI | Port C headers、bindgen（或手写绑定）|
| P2 gpui-wpe-bridge | 句柄/事件 glue | P1、serde/bincode |
| P3 min-web-process | 内嵌 WebKit 渲染 | P1、P6、P7（测试）|
| P4 gpui-app-host | GPUI 窗口/纹理采样 | GPUI/wgpu、P2 |
| P5 event-packet | 输入事件 IDL | serde/bincode |
| P6 mock-network | 静态 HTML 源 | rand（仅测试）|
| P7 texture-verify | 像素比对 | wgpu（下载）、image |
| P8 build-strap | 构建/依赖探测 | clap、which、std::process |
| P9 smoke-0 | E2E 验收 | python3、pytest、xvfb-run |

#### 3.2.3 关键接口与数据结构
- ExternalFrame（P2）：
```rust
pub struct ExternalFrame {
    pub pixels: Vec<u8>,      // RGBA8，阶段 0 采用拷贝路径
    pub size: (u32, u32),
    pub stride: u32,
}
```
- InputEvent（P5）：
```rust
#[derive(Serialize, Deserialize)]
pub enum InputEvent {
    KeyDown { key: String, code: u32 },
    KeyUp { key: String, code: u32 },
    MouseMove { x: f32, y: f32 },
    MouseDown { button: u8, x: f32, y: f32 },
    MouseUp { button: u8, x: f32, y: f32 },
    Wheel { delta_x: f32, delta_y: f32 },
    Resize { w: u32, h: u32 },
}
```
- UDS 帧格式（阶段 0）：
  - 帧头：`u32` 长度（LE）
  - 帧体：`bincode`(Message)
  - Message：`Frame(ExternalFrame)`｜`Event(InputEvent)`｜`Quit`

---

## 4. 用户界面（最小化）
- 单窗口 800×600，居中显示外部纹理 quad；窗口 resize 时上传新纹理并调节 UV
- 状态栏（可选）：显示帧率/尺寸（调试信息）
- 键盘：Esc 退出；F12 触发截图（开发用）

---

## 5. 数据与存储
- 阶段 0 无持久化；截图比对缓存（可选，`/tmp/monazite-p0/`）
- 临时文件在退出时清理；错误日志写入 `RUST_LOG=info` 控制台

---

## 6. 性能设计
- 目标：
  - 首帧（headless）≤ 3 s
  - 单帧上传（1080p RGBA8）≤ 25 ms（CPU 拷贝路径参考值）
- 策略：
  - 仅颜色块页面，避免复杂布局带来的变量
  - wgpu 纹理复用，减少反复分配
  - 后台线程异步接收帧，UI 线程仅上传与绘制

---

## 7. 安全与故障处理
- 无网络访问（仅 P6 mock-network 返回内置字符串）
- 进程退出码语义：0 正常；1 初始化失败；2 运行时致命
- 对策：
  - 输入非法/无效 URL：记录日志并忽略，不 crash
  - UDS 断开：主循环尝试优雅退出

---

## 8. 测试策略

### 8.1 单元测试（按模块）
- P1：符号加载、版本校验、头文件哈希
- P2：句柄 round-trip、键码映射 100 次零误差、异常 Err
- P3：`data:` 红块 headless、无效 URL 不崩溃、valgrind 泄漏 < 2MB（允许跳过 CI）
- P4：窗口创建、尺寸变化 UV 同步、headless 截图（xvfb-run）
- P5：bincode 往返、未知变体不 panic
- P6：两次返回不同颜色、并发 100 请求无数据竞争
- P7：红 PASS/蓝 FAIL；4K 下载 < 80 ms
- P8：幂等、离线模式提示
- P9：三类失败路径（未启动/超时/哈希不符）

### 8.2 集成与 E2E
- smoke-0（CI 调用顺序）：
  1) 启动 `mock-network`
  2) 启动 `min-web-process --headless`（监听 UDS）
  3) 启动 `gpui-app-host`（连接 UDS，显示帧）
  4) 截图 → `texture-verify` 比对 → 生成报告

### 8.3 CI 建议
- Linux-only job；安装系统依赖（见下）
- `xvfb-run -s "-screen 0 1280x800x24" cargo test --all`（示例）

---

## 9. 构建与依赖（阶段 0）
- 基本工具：Rust 1.80+、CMake、pkg-config、Python3、xvfb（CI）
- Linux 依赖（按实际 Port）：
  - WebKitGTK 方向：`libwebkit2gtk-4.1-dev`（或 4.0 系列），`libsoup`，`gtk` 等
  - 或 WPE 方向：`wpewebkit`、`wpebackend-fdo`
- cargo feature：`port_gtk`（默认）/`port_wpe`
- `cargo xtask`：
  - `sys-deps`：探测并提示安装
  - `cmake-gen`：生成必要构建脚手架（如需）

> 注意：不在 CI 全量构建 WebKit，仅使用系统/预编译包。

---

## 10. 里程碑与任务分解（4 周）
- W1：P5/P6/P7 基础库 & P8 构建工具可用；P1 绑定 PoC（选定 Port）
- W2：P3 渲染 headless 首帧；P2 事件/帧结构稳定；集成本地 UDS copy 路径
- W3：P4 窗口显示；尺寸/输入通路打通；像素比对通过
- W4：smoke-0 稳定；文档/截图/报告；缓慢/异常路径回归

---

## 11. 风险与回退
- Port 能力差异（GTK vs WPE）：统一以 `port_*` 特征门包装；CI 仅跑 GTK
- 零拷贝复杂度高：阶段 0 默认 CPU 拷贝；零拷贝放至阶段 1 研究
- X11/Wayland 差异：优先 X11；Wayland 下 fallback 可能性能更差
- 驱动/环境差异：提供 `--headless` 首选路径，降低耦合

---

## 12. 验收与交付物
- 每 crate `cargo test` 全绿
- `cargo xtask dist-0` 产出：`min-web-process`、`gpui-app-host`、`smoke-0.py`
- 运行 `python smoke-0.py`：10 s 内生成红/蓝块截图并比对 PASS
- 附：Hello WebKit 截图、测试报告摘要

---

## 13. 开放问题（阶段 0 内可暂缓）
- 是否引入最小 IPC framing crate（阶段 1 复用）？
- 是否在 CI 内置 swiftshader 强制 GPU 路径一致性？
- 是否预置一组小型示例页面集用于回归？

