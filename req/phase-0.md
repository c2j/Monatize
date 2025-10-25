阶段 0 只做一件事：**把「WebKit ⇄ GPUI」这条最小闭环跑通**，其余模块全是 mock 或空壳，但每个 crate 必须能独立 `cargo test` 并通过 CI。  
所有交付物在 **4 周内**完成，最终产出一张「Hello WebKit」截图 + 单元测试报告。

---

### 阶段 0 模块清单（9 个，可独立开发）

| ID | 模块名 | 交付物 | 独立验证命令 |
|----|--------|--------|--------------|
| P1 | **wpe-sys** | 纯净 WebKit-WPE FFI 绑定 crate | `cargo test --lib` （0 行 C++） |
| P2 | **gpui-wpe-bridge** | 纹理共享与事件转发 glue | `cargo test --features mock` |
| P3 | **min-web-process** | 单进程内嵌 WPE 的 mini 渲染器 | `./target/debug/min-web-process --url https://example.com --headless` |
| P4 | **gpui-app-host** | GPUI 窗口 + 外部纹理 quad | `cargo run --bin gpui-app-host` （弹出窗口显示网页） |
| P5 | **event-packet** | 统一输入事件 IDL + serde | `cargo test --lib` |
| P6 | **mock-network** | 返回静态 HTML 的伪网络 | `cargo run --bin mock-network` |
| P7 | **texture-verify** | 像素级比对工具库 | `cargo test --test pixel` |
| P8 | **build-strap** | 阶段 0 专用构建脚本 | `cargo xtask dist-0` |
| P9 | **smoke-0** | 阶段 0 端到端验收脚本 | `python ci/smoke-0.py` |

---

### 各模块详细设计（含单元测试）

#### P1 wpe-sys
- **职责**：把所选 WebKit Port 的头文件翻译成 Rust（阶段 0：WebKitGTK 或 WPE），**不包逻辑**。
- **API 示例**  
  ```rust
  pub fn wpe_view_backend_create() -> *mut wpe_view_backend;
  pub fn wpe_web_view_load_url(view: *mut wpe_web_view, url: *const c_char);
  ```
- **平台范围**：阶段 0 仅 Linux（WebKitGTK 或 WPE；优先 GTK）
- **单元测试**  
  1. 符号加载成功 `assert!(!wpe_init().is_null())`  
  2. 版本号 ≥ 2.6  
  3. 头文件哈希锁定，防止升级 break  

#### P2 gpui-wpe-bridge
- **职责**：平台无关句柄包装 + 事件序列化。  
- **核心结构**  
  ```rust
  pub struct ExternalFrame {
      pub handle: PlatformHandle, // EGLImage / HANDLE / IOSurface
      pub size: (u32, u32),
  }
  pub fn translate_event(gpui: &gpui::KeyEvent) -> WpeInputEvent;
  ```
- **单元测试**  
  1. mock 句柄 round-trip  
  2. 100 次键码转译零误差  
  3. 异常句柄返回 `Err` 而非 panic  

#### P3 min-web-process
- **职责**：最小渲染进程，内嵌 WebKit（GTK/WPE，阶段 0 限 Linux），输出 `ExternalFrame`。
- **主循环**  
  ```
  接收 URL → 调用 Port API 加载 →
  帧完成回调 → 发送纹理句柄（可拷贝回退）→
  阻塞等待 quit 信号
  ```
- **单元测试**  
  1. headless 加载 `data:text/html,<div style="background:red"></div>` → 纹理哈希比对红色  
  2. 无效 URL 不 crash  
  3. 内存泄漏 < 2 MB (valgrind)  

#### P4 gpui-app-host
- **职责**：纯 GPUI 窗口，创建 800×600 面板，采样 P2 句柄并全屏 quad。  
- **入口**  
  ```rust
  fn main() -> gpui::Result<()> {
      let frame_rx = bridge::receiver();
      App::new().run(|cx: &mut AppContext| {
          cx.spawn(|mut cx| async move {
              while let Ok(frame) = frame_rx.recv() {
                  cx.update(|_, cx| cx.request_redraw()).ok();
              }
          }).detach();
      })
  }
  ```
- **单元测试**  
  1. 窗口创建不 panic  
  2. 外部纹理尺寸变化时 quad UV 同步  
  3. CI headless 使用 `swiftshader` 截图比对  

#### P5 event-packet
- **职责**：定义键盘/鼠标/滚轮/resize 事件，供 P2 序列化。  
- **IDL 例**  
  ```rust
  #[derive(Serialize, Deserialize)]
  pub enum InputEvent {
      KeyDown { key: String, code: u32 },
      MouseMove { x: f32, y: f32 },
  }
  ```
- **单元测试**  
  1. bincode 往返  
  2. 未知事件 variant 反序列化不 panic  

#### P6 mock-network
- **职责**：阶段 0 无真正网络，只返回 **两条** 静态 HTML（红/蓝方块），用于像素测试。  
- **API**  
  ```rust
  pub async fn fetch(_url: &str) -> &'static str {
      match rand::bool() {
          true => RED_BLOCK_HTML,
          false => BLUE_BLOCK_HTML,
      }
  }
  ```
- **单元测试**  
  1. 两次调用返回不同颜色  
  2. 并发 100 请求无 data race  

#### P7 texture-verify
- **职责**：像素级比对库，支持 EGL/Metal/D3D 纹理下载到 CPU buffer 后算哈希。  
- **API**  
  ```rust
  pub fn assert_texture_eq(texture: &wgpu::Texture, expected: &[u8; 4]);
  ```
- **单元测试**  
  1. 红色 (255,0,0,255) PASS  
  2. 蓝色 (0,0,255,255) FAIL 红色期望  
  3. 4 K 纹理下载时间 < 80 ms  

#### P8 build-strap
- **职责**：一键安装系统依赖 + 生成 CMake 配置。  
- **子命令**  
  – `cargo xtask sys-deps` // apt/brew/pacman/vcpkg 自动检测  
  – `cargo xtask cmake-gen` // 生成 `build/cmake` 目录  
- **单元测试**  
  1. 重复运行幂等  
  2. 离线模式下提示友好  

#### P9 smoke-0
- **职责**：阶段 0 验收脚本，**无需源码**。  
- **步骤**  
  1. 启动 `mock-network`  
  2. 启动 `min-web-process --headless`  
  3. 启动 `gpui-app-host`（CI 用 Xvfb/headless）  
  4. 截图 → 与基准红/蓝 PNG 像素比对 → PASS/FAIL  
- **自身测试**  
  pytest 模拟三种失败路径（进程未启动 / 截图超时 / 哈希不符）

---

### 交付准则

1. 每 crate `cargo test` 必须全绿，CI 三平台 nightly 阻断。  
2. 最终 `cargo xtask dist-0` 输出：  
   – `min-web-process`  
   – `gpui-app-host`  
   – `smoke-0.py`  
3. 运行 `python smoke-0.py` 10 s 内弹出（或 headless 生成）「红色方块」截图即阶段 0 通关。

---

