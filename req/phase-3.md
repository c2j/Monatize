阶段 3 目标（6 周）
“PWA / ServiceWorker / 登录&数据同步 / 企业策略 / 性能调优 / 安全加固”，全部模块可独立编译、单测、Mock 集成，零依赖回炉。阶段 1/2 二进制保持兼容，仅新增或热插拔子模块。

---

### 阶段 3 模块清单（16 个）
| ID | 模块名 | 交付物 | 独立验证命令 |
|----|--------|--------|--------------|
| T1 | **sw-manager** | ServiceWorker 生命周期 & 更新 | `cargo test --lib` |
| T2 | **pwa-installer** | WebApp 安装、图标、清单校验 | `cargo run --bin pwa-install -- --manifest https://x.com/manifest.json` |
| T3 | **push-service** | PushManager + 本地通知 | `cargo run --bin push-daemon -- --mock-push` |
| T4 | **offline-cache** | CacheStorage + 网络回退策略 | `cargo test --lib` |
| T5 | **login-sync** | 账号登录、端到端加密书签/偏好同步 | `cargo test --lib` |
| T6 | **sync-engines** | 各数据类型（历史、扩展、标签）同步引擎 | `cargo test --lib` |
| T7 | **enterprise-policy** | 企业策略 Schema + 强制下发 | `cargo test --lib` |
| T8 | **cert-transparency** | CT 日志校验 + EV 标签 | `cargo test --lib` |
| T9 | **sandbox-policy** | 第二级沙盒（seccomp/BPF/Win32k Lockdown） | `cargo test --lib` |
| T10 | **memory-tuner** | 内存压缩 / 丢弃策略 / GPU 回收 | `cargo test --lib` |
| T11 | **power-monitor** | 电池感知降频、后台标签冻结 | `cargo test --lib` |
| T12 | **perf-profiler** | 内置 profiler（tracing + Tracy） | `cargo run --bin profiler -- --record 10s` |
| T13 | **translation-service** | 本地模型翻译（100 MB 量化） | `cargo run --bin translate -- --text "hello"` |
| T14 | **theme-store** | 主题 / 图标 / CSS 覆盖市场 | `cargo test --lib` |
| T15 | **release-sign** | 二进制签名 + 更新包验签 | `cargo test --lib` |
| T16 | **smoke-3** | 阶段 3 端到端回归 | `python ci/smoke-3.py` |

---

### 设计卡片（含单元测试）

#### T1 sw-manager
- **职责**：ServiceWorker 注册、更新、激活、卸载、事件派发。  
- **API**  
  ```rust
  pub fn register(scope: &str, script_url: &str) -> Result<SWRegId, SWError>;
  pub fn post_message(id: SWRegId, msg: Vec<u8>) -> Result<()>;
  ```
- **单测**  
  1. 重复注册返回同一 ID  
  2. 更新脚本后 skipWaiting 生效  
  3. 异常脚本不崩溃宿主  

#### T2 pwa-installer
- **职责**：读取 WebApp Manifest，生成桌面快捷方式（Win/macOS/Linux）。  
- **API**  
  ```rust
  pub fn install(manifest: Manifest) -> Result<DesktopShortcut, InstallError>;
  ```
- **单测**  
  1. 图标最小 192×192 校验  
  2. 离线启动 URL 正确写入快捷方式  
  3. 重复安装覆盖旧图标  

#### T3 push-service
- **职责**：PushManager 订阅、密钥对生成、本地通知回落。  
- **API**  
  ```rust
  pub fn subscribe(endpoint: &str) -> Result<Subscription, PushError>;
  ```
- **单测**  
  1. 公钥格式 = p256dh  
  2. 离线推送存入队列，网络恢复后展示  
  3. 用户拒绝通知权限返回错误  

#### T4 offline-cache
- **职责**：CacheStorage 查询/插入/淘汰 + 网络回退策略。  
- **API**  
  ```rust
  pub fn match_request(req: &Request) -> Option<Response>;
  ```
- **单测**  
  1. 精确匹配命中  
  2. 过期缓存自动删除  
  3. 配额满触发 LRU  

#### T5 login-sync
- **职责**：OAuth2 / WebAuthn 登录，端到端加密（RSA-OAEP + AES-GCM）。  
- **API**  
  ```rust
  pub fn sign_in(provider: Provider) -> Result<Jwt, LoginError>;
  pub fn encrypt_sync(data: &[u8], key: &PublicKey) -> Vec<u8>;
  ```
- **单测**  
  1.  wrong password 返回明确错误  
  2.  加密后密文 ≠ 原文  
  3.  本地无密钥时不可解密  

#### T6 sync-engines
- **职责**：书签、历史、扩展、标签等差异化同步（二进制 protobuf + bsdiff）。  
- **API**  
  ```rust
  pub fn sync_bookmark(last_timestamp: i64) -> Result<SyncBatch, SyncError>;
  ```
- **单测**  
  1. 双向冲突自动合并  
  2. 离线修改排队，上线后一次性上传  
  3. 大书签文件夹 (>10k) 分片上传  

#### T7 enterprise-policy
- **职责**：Chrome ADMX 兼容策略解析 + 强制下发 + 锁定灰显。  
- **API**  
  ```rust
  pub fn load_policy(json: &str) -> Result<Policy, PolicyError>;
  pub fn is_locked(key: &str) -> bool;
  ```
- **单测**  
  1. 非法 JSON 返回错误行号  
  2. 策略锁定后用户 UI 不可修改  
  3. 云策略覆盖本地策略  

#### T8 cert-transparency
- **职责**：CT 日志验证 + EV 标签展示 + 失败硬拦截（可旁路）。  
- **API**  
  ```rust
  pub fn verify_ct(chain: &[X509]) -> Result<(), CTError>;
  ```
- **单测**  
  1. 缺失 SCT 返回错误  
  2. 伪造日志签名不通过  
  3. 企业策略允许旁路时通过  

#### T9 sandbox-policy
- **职责**：第二级沙盒：Linux seccomp-bpf、macOS seatbelt、Win32k Lockdown。  
- **API**  
  ```rust
  pub fn apply_renderer_sandbox() -> Result<(), SandboxError>;
  ```
- **单测**  
  1. 执行 `execve` 被 kill  
  2. 访问 `/etc/passwd` 被拒绝  
  3. 失败时进程优雅退出并写 minidump  

#### T10 memory-tuner
- **职责**：后台标签内存压缩 (zstd)、GPU 纹理丢弃、AI 模型卸载。  
- **API**  
  ```rust
  pub fn compress_tab(id: TabId) -> Result<Compressed, MemoryError>;
  ```
- **单测**  
  1. 压缩率 > 50 %  
  2. 激活时解压缩 < 50 ms  
  3. OOM 时优先压缩而非 kill  

#### T11 power-monitor
- **职责**：电池状态监听 → 降帧率 / 暂停 SW / 降低 AI batch。  
- **API**  
  ```rust
  pub fn on_battery() -> bool;
  pub fn set_fps_limit(fps: u32);
  ```
- **单测**  
  1. 插入电源后帧率恢复 60  
  2. 电池模式下 GPU 频率下降  
  3. 模拟低电量 AI 首 token 延迟增加  

#### T12 perf-profiler
- **职责**：内置 tracing + Tracy 支持，一键输出火焰图。  
- **API**  
  ```rust
  pub fn start_record() -> Recorder;
  pub fn stop_record(rec: Recorder) -> Flamegraph;
  ```
- **单测**  
  1. 记录 10 s 生成非空 .tracy  
  2. 零性能时下采样关闭  
  3. 企业策略可禁用 profiler  

#### T13 translation-service
- **职责**：端侧 100 MB 量化模型，网页全文翻译。  
- **API**  
  ```rust
  pub fn translate(text: &str, from: Lang, to: Lang) -> String;
  ```
- **单测**  
  1. en→zh BLEU > 40  
  2. 无网络仍可用  
  3. 大文本 (>10k) 流式输出不 OOM  

#### T14 theme-store
- **职责**：主题市场 JSON + CSS 覆盖 + 图标包。  
- **API**  
  ```rust
  pub fn apply_theme(manifest: &ThemeManifest) -> Result<()>;
  ```
- **单测**  
  1. 无效 JSON 报错行号  
  2. 暗色主题对比度 > 4.5:1  
  3. 卸载主题恢复默认  

#### T15 release-sign
- **职责**：Ed25519 签名 + 更新包验签（cosign 兼容）。  
- **API**  
  ```rust
  pub fn verify(bin: &[u8], sig: &[u8], pk: &PublicKey) -> Result<(), SigError>;
  ```
- **单测**  
  1. 篡改二进制验签失败  
  2. 公钥过期返回特定错误  
  3. 企业策略可追加根证书  

#### T16 smoke-3
- **职责**：阶段 3 端到端验收，PWA + 登录 + 翻译一次过。  
- **Case**  
  1. 安装 PWA → 离线启动 → 本地通知  
  2. 登录账号 → 书签加密同步 → 第二设备出现  
  3. 网页翻译 → 无网络仍可用  
- **自身测试** | pytest 模拟失败路径 |

---

### 跨进程/线程总览（阶段 3 新增）

```
browser-main (M7)
├─ T1 sw-manager(线程)
├─ T3 push-service(线程)
├─ T5 login-sync(线程)
├─ T6 sync-engines(线程)
├─ T7 enterprise-policy(线程)
├─ T10 memory-tuner(线程)
├─ T11 power-monitor(线程)
├─ T12 perf-profiler(线程)
└─ 新增持久线程池（Rust rayon）供 AI/翻译/同步复用
```

所有新增线程通过 **M1 IPC + M2 消息** 与旧进程通信，零平台条件编译。

---

### 交付准则

1. 每 crate `cargo test` 全绿，CI 三平台 nightly 阻断。  
2. `cargo xtask dist-3` 产出：  
   – 内核 + PWA + 登录 + 更新器 + 签名验证  
3. `python ci/smoke-3.py` 60 s 内完成「PWA 离线 + 登录同步 + 网页翻译」即阶段 3 通关。

---

