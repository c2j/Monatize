阶段 2 目标（6 周）
“多标签 + 站点隔离 + 扩展骨架 + 安全策略”，所有模块可独立编译、单测、Mock 集成，不阻塞他人开发。阶段 1 二进制 **保持兼容**，仅新增或替换子模块。

---

### 阶段 2 模块清单（14 个）
| ID | 模块名 | 交付物 | 独立验证命令 |
|----|--------|--------|--------------|
| S1 | **tab-manager** | 多标签状态机 crate | `cargo test --lib` |
| S2 | **site-isolation** | 站点级进程分配策略（每站点进程 + 导航交换） | `cargo test --lib` |
| S3 | **content-srv-factory** | 可动态 spawn 的 Content 进程工厂 | `cargo run --bin cs-factory -- --dry-run` |
| S4 | **gpu-compositor** | 多 surface 合成器 | `cargo run --bin gpu-compositor -- --tabs 3` |
| S5 | **permission-manager** | 能力（camera/geo/cookie）策略 | `cargo test --lib` |
| S6 | **extension-host** | MV3 background service worker 宿主 | `cargo run --bin ext-host -- --load ./addon.xpi` |
| S7 | **extension-api** | `browser.*` Rust → JS 绑定 | `wasm-pack test --node` |
| S8 | **user-script** | userscript 注入与生命周期 | `cargo test --lib` |
| S9 | **pref-store** | 用户偏好 / 扩展存储 | `cargo test --lib` |
| S10 | **download-manager** | 断点续传 + 沙盒 IO | `cargo run --bin dl-mgr -- --mock-url <file>` |
| S11 | **print-manager** | 打印 → PDF 预览 | `cargo test --lib` |
| S12 | **update-service** | 差分 OTA 更新 | `cargo run --bin updater -- --check` |
| S13 | **crash-reporter** | 崩溃捕获与上传 | `cargo test --lib` |
| S14 | **smoke-2** | 阶段 2 端到端回归 | `python ci/smoke-2.py` |

---

### 设计卡片（含单元测试）

#### S1 tab-manager
- **职责**：标签生命周期、激活/冻结/关闭、前进后退栈。  
- **API**  
  ```rust
  pub struct TabManager {
      pub fn new_tab(url: &str) -> TabId;
      pub fn close_tab(id: TabId) -> Result<()>;
      pub fn freeze_tab(id: TabId) -> FrozenTab;
  }
  ```
- **单测**  
  1. 新建 100 标签内存增量 < 2 MB  
  2. 关闭后复用 ID 不冲突  
  3. 冻结后 GPU 纹理释放  

#### S2 site-isolation
- **职责**：按「site=scheme+eTLD+port」分配每站点进程；必要时导航进程交换（按 Port 能力）。
- **API**  
  ```rust
  pub fn allocate_process(url: &Url, map: &ProcessMap) -> ProcessId;
  ```
- **单测**  
  1. `a.com` vs `b.com` 不同进程  
  2. `a.com` vs `a.com#fragment` 同进程  
  3. 超过最大进程数复用最闲进程  

#### S3 content-srv-factory
- **职责**：动态 spawn / kill Content 进程，提供健康探针。  
- **API**  
  ```rust
  pub fn spawn(url: &Url) -> Result<Child, SpawnError>;
  pub fn kill(&mut child) -> Result<()>;
  ```
- **单测**  
  1. 冷启动 < 300 ms  
  2. kill -9 后工厂收到 SIGCHLD 并清理表  
  3. 端口冲突自增 +1  

#### S4 gpu-compositor
- **职责**：多 Tab Surface 合成，支持滑动指示器、标签缩略图。  
- **API**  
  ```rust
  pub fn add_surface(id: TabId, texture: TextureView);
  pub fn remove_surface(id: TabId);
  ```
- **单测**  
  1. 100 表面帧率保持 60 FPS  
  2. 移除后 VRAM 下降  
  3. 零表面不崩溃  

#### S5 permission-manager
- **职责**：能力模型（camera/mic/geo/cookie）询问与持久化。  
- **API**  
  ```rust
  pub fn query(origin: &str, cap: Capability) -> Permission;
  ```
- **单测**  
  1. 用户拒绝后二次询问自动 block  
  2. UI 线程不阻塞  
  3. 清空站点数据同步清除权限记录  

#### S6 extension-host
- **职责**：MV3 background service worker 宿主（WASM sandbox）。
- **API**  
  ```rust
  pub fn load(manifest: PathBuf) -> ExtensionId;
  pub fn unload(id: ExtensionId) -> Result<()>;
  ```
- **单测**  
  1. 异常 wasm 不崩溃宿主  
  2. unload 后内存归零  
  3. 消息溢出不阻塞主进程  

#### S7 extension-api
- **职责**：`browser.tabs.* / browser.runtime.*` Rust → JS 绑定。  
- **编译目标** | WASM；`wasm-bindgen` |
- **单测**  
  1. tabs.create 返回 JSON 与 Chrome 格式误差 < 5 %  
  2. 并发 50 消息无丢序  
  3. 非法参数抛出 TypeError  

#### S8 user-script
- **职责**：userscript / GM_* 注入与生命周期管理。  
- **API**  
  ```rust
  pub fn inject(script: &str, run_at: RunAt, world: IsolatedWorld);
  ```
- **单测**  
  1. `@run-at document-start` 在 DOM 前执行  
  2. GM_xmlhttpRequest 拦截走 M3 网络  
  3. 卸载脚本后定时器自动清理  

#### S9 pref-store
- **职责**：用户偏好、扩展存储、JSON Schema 校验。  
- **API**  
  ```rust
  pub fn get<T: DeserializeOwned>(key: &str) -> Option<T>;
  pub fn set<T: Serialize>(key: &str, value: &T) -> Result<()>;
  ```
- **单测**  
  1. 并发写不损坏文件  
  2. 降级到只读目录优雅只读  
  3. 存储上限 100 MB 溢出报错  

#### S10 download-manager
- **职责**：多线程下载 + 断点续传 + 沙盒 IO（仅下载目录可写）。  
- **API**  
  ```rust
  pub fn download(url: &str, dst: &Path) -> DownloadId;
  pub fn pause(id: DownloadId) -> Result<()>;
  ```
- **单测**  
  1. 500 MB 文件网络中断后续传字节一致  
  2. 磁盘满自动暂停并通知  
  3. 恶意路径 `../../../etc/passwd` 被 sanitize  

#### S11 print-manager
- **职责**：打印 → PDF 预览 → 可选系统打印机。  
- **API**  
  ```rust
  pub fn print_to_pdf(tab: TabId) -> Result<PathBuf, PrintError>;
  ```
- **单测**  
  1. 空白页 PDF 大小 < 5 kB  
  2. 包含图片页无失真  
  3. 并发打印 10 标签不串台  

#### S12 update-service
- **职责**：差分 OTA（bsdiff）（内核 + 模型）。  
- **API**  
  ```rust
  pub fn check_update() -> Result<UpdateInfo, UpdateError>;
  pub fn apply_patch(patch: &[u8]) -> Result<()>;
  ```
- **单测**  
  1. 旧→新二进制哈希一致  
  2. 断电回滚（原子 rename）  
  3. 签名验证失败拒绝安装  

#### S13 crash-reporter
- **职责**：崩溃捕获 (breakpad/rust-minidump) + 上传（用户可关）。  
- **API**  
  ```rust
  pub fn init(upload_url: Option<&str>) -> Result<()>;
  ```
- **单测**  
  1. 故意 segfault 生成 minidump  
  2. 上传失败本地留存 ≤ 3 份  
  3. 用户关闭上传即不发起网络  

#### S14 smoke-2
- **职责**：阶段 2 端到端验收，多标签 + 扩展 + 打印一次过。  
- **Case**  
  1. 打开 `a.com` `b.com` 两个标签 → 像素不同  
  2. 安装 AdBlock 扩展 → 网络请求被拦截日志出现  
  3. 打印任意标签 → PDF 文件非空  
- **自身测试** | pytest 模拟失败路径 |

---

### 跨进程/线程模型（阶段 2 新增）

```
browser-main (M7)
├─ S1 tab-manager(线程)
├─ S2 site-isolation(线程)
├─ S3 content-srv-factory → 动态池 Content[N]
├─ S4 gpu-compositor(GPU 进程)
├─ S5 permission-manager(线程)
├─ S6 extension-host → Extension[M] 进程
├─ S12 update-service(线程)
└─ S13 crash-reporter(线程)
```

所有新增进程通过 **M1 IPC + M2 消息** 接入，零平台条件编译。

---

### 交付准则

1. 每 crate `cargo test` 全绿，CI 三平台 nightly 阻断。  
2. `cargo xtask dist-2` 产出：  
   – 内核 + 扩展宿主 + 更新器 + 崩溃报告器  
3. `python ci/smoke-2.py` 30 s 内完成「双标签 + 扩展 + PDF」即阶段 2 通关。

---

