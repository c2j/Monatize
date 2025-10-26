# Monazite Phase‑2 详细设计规格书

版本：v0.3.0‑p2  · 周期：6 周  · 平台：Linux（优先）/ Windows（设计预留）

---

## 1. 项目概述

### 1.1 项目基本信息
- 项目名称：Monazite 浏览器（Phase‑2）
- 项目代号：Monazite P2
- 项目类型：多进程桌面应用
- 开发语言：Rust（少量 JS/WASM）
- 目标平台：Linux（GTK/WebKitGTK），Windows（通过 cfg/sys 预留）
- 版本：v0.3.0‑p2

### 1.2 背景与目标
- 背景：Phase‑0/1 已实现三进程骨架、真实 WebKit 渲染、E2E 冒烟、browser‑main 内置 GUI（最小工具栏+徽标）。
- Phase‑2 总体目标（对齐 req/phase‑2.md）：
  1) 多标签（Tab）与站点隔离（per‑site content 进程 + 导航交换）；
  2) 扩展生态骨架（MV3 background/`browser.*` API/Userscript）；
  3) 安全与策略（权限、下载沙盒、崩溃报告、OTA 更新、打印）。
- 成功标准：
  - `python ci/smoke-2.py` 30 s 内完成「双标签 + 扩展 + PDF」；
  - `cargo xtask dist-2` 产出新增二进制（扩展宿主/更新器/崩溃报告器等）；
  - 各新增 crate `cargo test` 全绿；保持 Phase‑1 二进制兼容（仅新增或替换子模块）。

### 1.3 项目范围
- 包含：S1–S14（见 2.1）及其与 M1/M2/M5/M7/M11/M12 的集成。
- 排除：完整浏览器功能（多窗口/多配置文件/同步等）、高阶 GPU 零拷贝跨 OS 全适配。
- 假设：CI 可用 Xvfb；Linux 桌面具备 GTK/WebKitGTK；扩展以本地测试包为主（非商店分发）。
- 约束：6 周周期、尽量复用 Phase‑1 基础，避免大规模破坏性重构。

---

## 2. 需求分析

### 2.1 功能需求（S1–S14）
- S1 tab‑manager：多标签状态机，激活/冻结/关闭，前进后退栈。
- S2 site‑isolation：按 site=scheme+eTLD+port 分配 content 进程，必要时导航交换。
- S3 content‑srv‑factory：动态 spawn/kill content 进程与健康探针。
- S4 gpu‑compositor：多 Tab surface 合成，标签缩略图与滑动指示。
- S5 permission‑manager：camera/mic/geo/cookie 策略查询与持久化。
- S6 extension‑host：MV3 background service worker 宿主（WASM sandbox）。
- S7 extension‑api：`browser.*` Rust↔JS 绑定（WASM）。
- S8 user‑script：Userscript/GM_* 注入与生命周期。
- S9 pref‑store：偏好与扩展存储，JSON Schema 校验。
- S10 download‑manager：断点续传 + 沙盒 IO（下载目录）。
- S11 print‑manager：打印→PDF 预览（文件输出）。
- S12 update‑service：差分 OTA（签名校验、断电回滚）。
- S13 crash‑reporter：崩溃捕获与可选上传（尊重用户设置）。
- S14 smoke‑2：阶段 2 端到端回归。

### 2.2 非功能需求
- 性能：
  - S3 冷启动 < 300 ms；S4 100 表面 60 FPS；S10 500 MB 文件续传完整；
  - 双标签首开 ≤ 3 s，GUI 动作响应 < 50 ms。
- 可靠性：
  - 子进程崩溃 3 s 内自动清理并可重建；
  - 更新失败回滚稳定；崩溃日志 ≤ 3 份轮转。
- 兼容性：Linux 优先，Windows cfg 预留（打印/崩溃/更新以占位实现）；
- 安全：权限最小化、扩展/脚本隔离、下载目录沙盒、更新签名校验。
- 可维护性：新增模块均独立 crate，接口清晰、单测覆盖核心路径（≥70%）。

---

## 3. 技术架构设计

### 3.1 技术选型
- GUI/渲染：GTK 3（gtk 0.15）、WebKitGTK（webkit2gtk 0.18）、wgpu 27（S4）。
- 异步/并发：tokio，多线程调度；
- WASM 宿主：首选 wasmtime/wasmi（二选一，P2 以最小可用沙箱为目标）；
- 打印/PDF：GTK Print/Cairo PDF surface（或 WebKit Print API，如可用）；
- 更新：bsdiff 增量 + 签名（ED25519）校验；
- 崩溃：breakpad/rust‑minidump（Linux 先行）；
- 序列化与 IPC：serde/bincode + M1 ipc‑channel；消息定义复用 M2。

### 3.2 跨进程/线程拓扑（新增）
```
browser-main(M7)
├─ S1 TabManager(线程)
├─ S2 SiteIsolation(线程)
├─ S3 ContentFactory → Content[N](进程)
├─ S4 GpuCompositor(GPU 进程 or 线程)
├─ S5 PermissionManager(线程)
├─ S6 ExtensionHost → Extension[M](进程/沙箱)
├─ S12 UpdateService(线程)
└─ S13 CrashReporter(线程)
```
所有新增进程通过 M1/M2 接入，平台差异封装在 sys 层。

### 3.3 数据流（典型）
1) UI 触发 new_tab(url) → S2 计算 site → S3 分配/复用 content 进程；
2) Content 渲染输出 surface 句柄 → S4 合成到主窗口；
3) 权限请求 → S5 决策（持久化至 S9）；
4) 扩展 background（S6）监听事件 → 通过 S7 调用内核能力；
5) 下载（S10）与打印（S11）走独立服务，通知 UI；
6) 更新（S12）后台检查，用户同意后应用；崩溃（S13）生成报告。

### 3.4 项目结构（新增建议）
```
crates/
  tab-manager/          # S1
  site-isolation/       # S2
  content-srv-factory/  # S3 (bin: cs-factory)
  gpu-compositor/       # S4 (bin)
  permission-manager/   # S5
  extension-host/       # S6 (bin: ext-host)
  extension-api/        # S7 (wasm)
  user-script/          # S8
  pref-store/           # S9
  download-manager/     # S10 (bin: dl-mgr)
  print-manager/        # S11
  update-service/       # S12 (bin: updater)
  crash-reporter/       # S13
ci/
  smoke-2.py            # S14
```

---

## 4. 界面与交互（P2 最小集）
- 主窗新增 TabStrip：显示标签、激活/关闭、+（新建）。
- 权限提示：非阻塞气泡（origin + 能力 + 记住选择）。
- 下载：状态条目（名称、进度、暂停/继续/打开目录）。
- 打印：菜单项“打印为 PDF…”，生成后打开所在目录。
- 扩展：开发者菜单“加载扩展目录…（manifest.json）”。

---

## 5. 数据设计（关键结构）

### 5.1 核心类型（示例）
```rust
pub type TabId = u64;
pub struct TabState { pub url: String, pub title: String, pub frozen: bool }
pub enum Capability { Camera, Microphone, Geolocation, Cookie }
pub struct PermissionRecord { origin: String, cap: Capability, policy: Permission }
pub enum Permission { Allow, Deny, Prompt }
pub struct ProcessMap { /* site -> pid */ }
```

### 5.2 存储与配置
- S9 使用 JSON(TOML) 文件存储，Schema 校验；并发写加独占锁；
- 扩展存储与偏好分 namespace；上限 100 MB，超限报错；
- 下载仅允许写入下载目录（可配置且落在用户 home）。

### 5.3 IPC 消息（新增/复用）
- 复用 M2 的 `LoadUrl/SurfaceReady/PermissionQuery/Download*` 等消息；
- 新增：`NewTab/CloseTab/FreezeTab/PrintToPdf/ExtMsg/UpdateCheck/CrashDump`；
- 大对象通过 M1 Attachment/共享内存或文件句柄（平台抽象）。

---

## 6. 模块设计（含 API 与单测）

### S1 tab‑manager
职责：标签生命周期与历史栈；冻结释放 GPU 纹理。API：
```rust
pub struct TabManager;
impl TabManager {
  pub fn new_tab(&mut self, url: &str) -> TabId;
  pub fn close_tab(&mut self, id: TabId) -> anyhow::Result<()>;
  pub fn freeze_tab(&mut self, id: TabId) -> anyhow::Result<TabState>;
}
```
单测：100 标签内存增量 < 2 MB；关闭后 ID 不冲突；冻结后 VRAM 下降。

### S2 site‑isolation
职责：per‑site content 分配；超限复用最闲进程；导航交换。API：
```rust
pub fn allocate_process(url: &url::Url, map: &mut ProcessMap) -> ProcessId;
```
单测：a.com vs b.com 不同进程；同站点片段同进程；超限复用最闲。

### S3 content‑srv‑factory
职责：spawn/kill content；SIGCHLD 清理；端口冲突自增。API：
```rust
pub fn spawn(url: &url::Url) -> Result<std::process::Child, SpawnError>;
pub fn kill(child: &mut std::process::Child) -> Result<()>;
```
单测：冷启动 < 300 ms；kill‑9 后清理；端口冲突自增。

### S4 gpu‑compositor
职责：多 surface 合成；缩略图。API：
```rust
pub fn add_surface(id: TabId, view: TextureView);
pub fn remove_surface(id: TabId);
```
单测：100 表面 60 FPS；移除后 VRAM 下降；零表面不崩溃。

### S5 permission‑manager
职责：权限策略与持久化。API：
```rust
pub fn query(origin: &str, cap: Capability) -> Permission;
```
单测：拒绝后自动 block；非阻塞 UI；清空站点数据同步清除。

### S6 extension‑host
职责：MV3 background 宿主（WASM）。API：
```rust
pub fn load(manifest: &std::path::Path) -> ExtensionId;
pub fn unload(id: ExtensionId) -> anyhow::Result<()>;
```
单测：异常 wasm 不崩溃；unload 内存归零；溢出不阻塞主进程。

### S7 extension‑api
职责：`browser.tabs.* / browser.runtime.*` 绑定（WASM）。
单测：tabs.create JSON 兼容；并发 50 消息无丢序；非法参数抛 TypeError。

### S8 user‑script
职责：userscript 注入与生命周期；`@run-at` 与 `GM_*`。API：
```rust
pub fn inject(script: &str, run_at: RunAt, world: IsolatedWorld);
```
单测：document‑start 顺序正确；GM_xmlhttpRequest 走 M3；卸载清理定时器。

### S9 pref‑store
职责：偏好/扩展存储；并发安全；上限控制。API：
```rust
pub fn get<T: serde::de::DeserializeOwned>(key: &str) -> Option<T>;
pub fn set<T: serde::Serialize>(key: &str, value: &T) -> anyhow::Result<()>;
```
单测：并发写不损坏；只读目录降级；100 MB 上限生效。

### S10 download‑manager
职责：多线程下载、续传、沙盒 IO。API：
```rust
pub fn download(url: &str, dst: &std::path::Path) -> DownloadId;
pub fn pause(id: DownloadId) -> anyhow::Result<()>;
```
单测：500 MB 中断后续传一致；磁盘满暂停；路径跳转被 sanitize。

### S11 print‑manager
职责：打印→PDF（文件）。API：
```rust
pub fn print_to_pdf(tab: TabId) -> Result<std::path::PathBuf, PrintError>;
```
单测：空白 PDF < 5 kB；含图不失真；并发 10 标签不串台。

### S12 update‑service
职责：差分 OTA，签名校验与回滚。API：
```rust
pub fn check_update() -> Result<UpdateInfo, UpdateError>;
pub fn apply_patch(patch: &[u8]) -> Result<()>;
```
单测：旧→新哈希一致；断电回滚；签名失败拒绝。

### S13 crash‑reporter
职责：崩溃捕获 + 上传（可关）。API：
```rust
pub fn init(upload: Option<&str>) -> anyhow::Result<()>;
```
单测：segfault 生成 dump；上传失败本地留存 ≤3；关闭上传不联网。

### S14 smoke‑2（端到端）
- Case：
  1) 打开 a.com/b.com 两个标签 → 像素不同；
  2) 安装 AdBlock 测试扩展 → 网络拦截日志出现；
  3) 打印任意标签 → 生成非空 PDF；
- 自测：pytest 覆盖失败路径。

---

## 7. 性能设计与监控
- 目标：S3 冷启 <300 ms；双标签≤3 s；S4 100 surface 60 FPS；S10 续传完整；
- 策略：
  - 延迟加载扩展与打印；合成批处理；纹理回收；
  - IPC 大对象走 Attachment/共享内存；
  - 监控：加载/渲染/合成/下载/打印/更新时延与错误日志。

---

## 8. 测试与验收
- 单测：各 S1–S13 子模块 `cargo test`；S7 `wasm-pack test --node`；
- 集成：browser‑main + S1/S2/S3/S4 路径；权限/下载/打印/更新基础集成；
- E2E：`python ci/smoke-2.py`（30 s 内完成 3 个场景）；
- 交付：`cargo xtask dist-2` 产出扩展宿主/更新器/崩溃报告器等。

---

## 9. 安全设计
- 站点隔离：per‑site content 进程；导航交换切断跨站面状态；
- 权限策略：默认最小；记忆策略；清空站点数据时统一清理；
- 扩展/脚本：WASM 沙箱、IsolatedWorld；API 白名单；
- 下载沙盒：限制到下载目录，避免路径穿越；
- 更新签名：ED25519 校验；失败回滚；离线包校验。

---

## 10. 部署与分发
- 新增 `cargo xtask dist-2`：扩展宿主（ext-host）、更新器（updater）、合成器（可选）、崩溃报告器；
- Linux：继续使用 AppImage/脚本分发；Windows：生成占位构建（cfg 关闭不可用功能）。

---

## 11. 计划与里程碑（6 周）
- 里程碑‑A（W1–W2）：S1/S2/S3/S4（多标签、站点隔离、进程工厂、合成器）
- 里程碑‑B（W3–W4）：S5/S9/S10/S11（权限、存储、下载、打印）
- 里程碑‑C（W5）：S6/S7/S8（扩展宿主、API、用户脚本）
- 里程碑‑D（W6）：S12/S13/S14（更新、崩溃、E2E）与性能/稳定性收尾

---

## 12. 风险与对策
- WebKit 打印/PDF API 差异：优先 Linux 路径，Windows 以占位；
- WASM 宿主稳定性：先最小能力、严格资源限额；
- GPU/合成跨驱动差异：降低上限并提供降级路径；
- 更新回滚安全：先离线/沙箱验证，再原子替换；
- 时间风险：严格分阶段可交付，每阶段保留验证命令。

---
文档版本：v0.1（草案）— 与 `req/phase-2.md` 对齐；实施中若接口变更，将同步更新。

