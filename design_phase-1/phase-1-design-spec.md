# Monazite Phase‑1 详细设计规格书

## 1. 项目概述

### 1.1 项目基本信息
- 项目名称：Monazite 浏览器（Phase‑1）
- 项目代号：Monazite P1
- 项目类型：多进程桌面应用
- 开发语言：Rust（少量 JS/WASM）
- 目标平台：Linux（优先）/ Windows（设计预留）
- 版本：v0.2.0‑p1
- 预期周期：3 个月

### 1.2 背景与目标
- 背景：Phase‑0 已实现 WebKit ⇄ GPUI 闭环 PoC（headless + WebKitGTK 可选），端到端像素验证、打包、Xvfb 一键演示均可用。
- 目标（对齐 req/phase‑1.md）：
  1) 三进程架构跑起来：browser‑main、content‑srv、gpu‑srv（再加 network‑srv 与 ai‑runtime）；
  2) 能打开网页（network‑srv 下载 → content‑srv 渲染 → gpu‑srv 合成显示或截图）；
  3) AI 能回答（window.ai.ask → ai‑runtime → DOM 注入）；
  4) 模块可独立编译、单测、Mock 集成，不互相 block。
- 成功标准：
  - 每 crate `cargo test` 全绿；
  - `cargo xtask dist` 输出 P1 进程二进制；
  - `python ci/smoke.py` 10 s 内完成并输出包含 “Example Domain” 摘要。

### 1.3 项目范围
- 包含：IPC 通道、消息 IDL 生成、网络/内容/GPU/AI/主控五个（子）进程、DOM 注入库、window.ai wasm API、极简布局引擎（测试用）、构建脚本与 E2E 冒烟。
- 排除：完整浏览器功能、跨平台 UI 生态适配、GPU 零拷贝的全平台最优实现（按 OS 渐进）。
- 假设：CI 可用 Xvfb；内网环境可缓存依赖；Linux 桌面具备基础图形栈。
- 约束：3 个月交付；优先 Linux，Windows 路径采用 sys 模块隔离与特性门控留待后续。

## 2. 需求分析

### 2.1 功能需求（模块与验收）
| 模块 | 说明 | 优先级 | 验收标准 |
|---|---|---|---|
| M1 ipc‑channel | 跨进程可靠字节通道（零拷贝能力预留） | 高 | `cargo test --lib`；回环 <1ms、1MiB 大消息、断开错误 |
| M2 message‑defs | IDL→Rust 代码生成 | 高 | `cargo test --lib`；round‑trip 全类型、哈希稳定 |
| M3 network‑srv | HTTP/1.1‑3 下载，返回 HttpResponse | 高 | `cargo run --bin network-srv -- --mock`；wiremock 验证 |
| M4 gpu‑srv | wgpu 光栅化/合成/截图 | 高 | `cargo run --bin gpu-srv -- --triangle`；像素对比 |
| M5 content‑srv | 封装 WebKit/WPE 与 GPU/IPC 桥接 | 高 | `cargo run --bin content-srv -- --html <file>` |
| M6 ai‑runtime | 端侧推理（ONNX/GGUF 二选一） | 中 | `cargo run --bin ai-runtime -- --bench` |
| M7 browser‑main | 主控拉起/监视子进程，路由输入 | 高 | `cargo run --bin browser -- https://example.com` |
| M8 dom‑embed | Shadow DOM 注入库 | 高 | `cargo test --lib`；XSS 过滤 |
| M9 window‑ai‑api | `window.ai.ask()` wasm | 中 | `wasm-pack test --node` |
| M10 servo‑lite | 极简 layout 生成 DisplayList | 中 | `cargo test --lib` |
| M11 build‑ops | xtask：dist/test/fmt‑lint | 高 | `cargo xtask dist` 产物存在 |
| M12 smoke‑test | E2E 脚本 | 高 | `python ci/smoke.py` PASS |

### 2.2 非功能需求
- 性能：
  - 首次打开 example.com ≤ 2 s（缓存后 ≤ 1 s）；
  - GPU 截图用例 ≤ 200 ms；
  - AI 50 tokens 延迟 ≤ 800 ms（本地 CPU 路径，可 Mock）。
- 可靠性：
  - 子进程崩溃 3 s 内自动拉起；
  - IPC 断线错误可观测、可恢复。
- 兼容性：Linux/X11 首发，Wayland 次选；Windows 通过 cfg/sys 隔离占位。
- 安全：DOM 注入过滤脚本；IPC/进程边界不泄漏未定义句柄。
- 可维护性：生成代码与手写模块分离；单测覆盖核心路径（≥70% 行覆盖）。

## 3. 技术架构设计

### 3.1 技术选型
- GUI/图形：winit 0.30 + wgpu 27（gpu‑srv），Xvfb 用于 CI/headless。
- Web 内容：WebKitGTK/WPE（content‑srv）；按 OS feature 选择 port。
- IPC：M1 封装 UDS seqpacket（Linux）/ 命名管道（Windows）；serde+bincode。
- 异步：tokio；
- 代码生成：syn/quote（dev‑dep）在 M2；
- 错误：anyhow/thiserror；日志：tracing。

### 3.2 系统架构与数据流
- 进程：
  - browser‑main：入口与编排，创建/监视 M3/M4/M5/M6，转发输入与路由请求；
  - network‑srv：下载资源（HTTP），返回 HttpResponse；
  - content‑srv：调 WebKit/WPE 渲染，注入 DOM，产出帧/DisplayList；
  - gpu‑srv：接收 DisplayList 或像素帧 → 光栅化/合成 → surface/screenshot；
  - ai‑runtime：模型加载与推理。
- 数据流（典型）：
  1) browser 发起 `LoadUrl` → content；content 请求 network `HttpRequest`；
  2) network 回 `HttpResponse`（HTML/CSS/JS）；content 渲染 → 生成显示对象；
  3) content → gpu：`DisplayList` 或 `ExternalFrame` 句柄；gpu 呈现并可截图回送；
  4) window.ai.ask：M9（WASM）→ browser/main → ai‑runtime → content 注入（M8）。

### 3.3 项目结构（建议）
```
crates/
  ipc-channel/        # M1
  message-defs/       # M2 (build.rs 生成 src/generated.rs)
  network-srv/        # M3 (bin)
  gpu-srv/            # M4 (bin)
  content-srv/        # M5 (bin)
  ai-runtime/         # M6 (bin)
  browser-main/       # M7 (bin: browser)
  dom-embed/          # M8 (lib)
  window-ai-api/      # M9 (wasm)
  servo-lite/         # M10 (lib)
xtask/                # M11
ci/
  smoke.py            # M12
```

## 4. 界面与交互（最小集）
- browser 主窗：提供基础窗口与日志输出；输入事件路由到 content。
- 截图：gpu‑srv 暴露截图命令（供 E2E 比对）。

## 5. 数据设计

### 5.1 IDL（示例）
```
struct HttpRequest { url: String }
struct HttpResponse { status: u16, headers: Vec<(String,String)>, body: Bytes }
struct DisplayList { items: Vec<DrawCmd> }
struct AiRequest { prompt: String, max_tokens: u32 }
struct AiResponse { text: String }
```
- M2 负责从 `idl/*.idl` 生成 Rust：`message_defs::generated::*`。

### 5.2 关键消息与通道
- Network：`HttpRequest` → `HttpResponse`；
- Content→GPU：`DisplayList`（首选）或 `ExternalFrame{fd/handle, size, stride}`；
- AI：`AiRequest` ↔ `AiResponse`；
- 控制面：`LoadUrl`、`Screenshot`、`InjectShadowDom`、`WindowResized` 等。

### 5.3 零拷贝设计（Phase‑1 级别）
- Linux：优先 DMABUF/Fd 句柄（当不可用时退化为 shared‑mem/cpu‑copy）；
- Windows：命名管道 + shared texture（预研接口，Phase‑1 可用内存映射退化）。
- M1 为大对象定义 `Attachment` 抽象与内核句柄安全转移 API，业务无平台 cfg。

## 6. 性能设计
- 目标：
  - example.com 首次 ≤ 2 s；截图 ≤ 200 ms；AI 50t ≤ 800 ms；
- 策略：
  - IPC 大消息采用 `Attachment`（避免二次拷贝）；
  - GPU 后端回落：Vulkan→Metal→D3D12→llvmpipe/WARP；
  - content 与 gpu 分离、并行流水；
  - 监控：简单计时与日志埋点（加载、渲染、AI 往返）。

## 7. 安全设计
- DOM 注入库（M8）执行 HTML 过滤（<script> 去除、事件属性白名单）。
- IPC 序列化白名单（由 M2 生成的类型），拒绝未知变体。
- 进程边界：content/ai/network 最小权限运行；
- 敏感句柄（fd/handle）只经 M1 `Attachment` 传递并检查平台合法性。

## 8. 测试策略

### 8.1 单元测试（各模块）
- M1：回环延迟、1MiB、断开错误；
- M2：round‑trip 全覆盖、哈希稳定；
- M3：wiremock 头/状态码、重试、分块下载；
- M4：红色矩形像素比对、空列表不崩溃、8K 降级；
- M5：纯 HTML 无 CSS 不崩溃、AI 响应后 DOM +1、异常 HTML 走错误通道；
- M6：延迟/吞吐/缺失模型错误码；
- M7：子进程 crash 重启 ≤ 3 s、端口冲突自增、1000 次 IPC ping 无泄漏；
- M8：XSS 过滤、重复注入 ID 自增、1MiB 文本；
- M9（wasm）：Node 环境 Mock、并发 10 调用、类型拒绝；
- M10：基本布局性质、深度 1024、非法 CSS 跳过。

### 8.2 集成与 E2E
- `cargo xtask test`：聚合单测 + 关键集成（Xvfb 下 gpu 截图）；
- `python ci/smoke.py`：
  1) browser 拉起 network/gpu/content/ai；
  2) 打开 example.com；
  3) 触发 `window.ai.ask('summarize')`；
  4) 截图比对与摘要包含 “Example Domain”。

## 9. 部署与分发（M11）
- `cargo xtask dist`：生成以下二进制与脚本：
  - network-srv / gpu-srv / content-srv / ai-runtime / browser；
  - ci/smoke.py；
- `cargo xtask fmt-lint`：fmt + clippy；
- `cargo xtask test`：各 crate test + wasm-pack test + 关键集成。

## 10. 运行与运维
- 日志：tracing（env 级别 control）；
- 观测：简单计时指标（加载/渲染/AI）；
- 崩溃恢复：browser‑main 监督策略（指数退避、最多 N 次在窗口期内）。

## 11. 计划与里程碑（3 个月）
- Milestone‑1（第 1 个月）
  - 完成 M1/M2/M11（IPC、IDL 生成、构建脚本）；
  - 建立骨架仓结构；CI 运行基础单测；
- Milestone‑2（第 2 个月）
  - 完成 M3/M4/M10（网络、GPU、布局基线）；
  - 小规模集成：content→gpu 像素红屏；
- Milestone‑3（第 3 个月）
  - 完成 M5/M6/M7/M8/M9/M12；
  - E2E 通关：`python ci/smoke.py` PASS，`cargo xtask dist` 产物齐全。

## 12. 风险与对策
- GPU 零拷贝跨 OS 复杂：优先 Linux DMABUF，Windows 退化内存映射；
- WebKit/WPE API 差异：用 feature 与适配层隔离；
- wasm‑pack 环境与 CI：提供可跳过/降级策略，分层验证；
- AI 推理依赖体积与性能：默认 Mock 模式与小模型基线，真实路径在本地/CI 有开关。

## 13. 与 Phase‑0 的衔接
- 复用：
  - 像素验证与截图经验（texture‑verify 思路、Xvfb 流程）；
  - UDS + bincode 的 framing 方案（迁移到 M1 封装并抽象 Attachment）；
  - xtask 风格与 sys‑deps 提示、demo 思路（迁移至 M11）。
- 升级：
  - 从单机 PoC 走向多进程可独立验证；
  - 引入 IDL 生成，减少手工消息定义；
  - 将 headless 路径沉淀为 CI 稳定回路，WebKit 路径在 content‑srv 受控启用。

---
文档版本：v0.1（草案） — 评审意见将用于细化 M1/M2 接口与消息清单的最终定稿。
