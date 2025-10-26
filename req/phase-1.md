阶段 1 目标（3 个月）  
“先把三进程跑起来，网页能打开，AI 能回答，DOM 能注入”——所有模块必须可独立编译、单测、Mock 集成，不互相 block。

---

### 阶段 1 模块清单（12 个）
| ID | 模块名 | 交付物 | 独立验证命令 |
|----|--------|--------|--------------|
| M1 | **ipc-channel** | 跨进程字节管道 crate | `cargo test --lib` |
| M2 | **message-defs** | IDL→Rust 代码生成 | `cargo test --lib` |
| M3 | **network-srv** | 网络子进程 | `cargo run --bin network-srv -- --mock` |
| M4 | **gpu-srv** | GPU 光栅化 & 合成 | `cargo run --bin gpu-srv -- --triangle` |
| M5 | **content-srv** | 内容渲染进程 | `cargo run --bin content-srv -- --url https://example.com --screenshot /tmp/page.png` |
| M6 | **ai-runtime** | 端侧推理进程 | `cargo run --bin ai-runtime -- --bench` |
| M7 | **browser-main** | 主控进程 | `cargo run --bin browser -- https://example.com --real-screenshot /tmp/page.png` |
| M8 | **dom-embed** | 影子根注入库 | `cargo test --lib` |
| M9 | **window-ai-api** | 前端 JS ↔ Rust 粘合 | `wasm-pack test --node` |
| M10 | **servo-lite** | 极简布局引擎 | `cargo test --lib` |
| M11 | **build-ops** | 构建脚本 | `cargo xtask dist` |
| M12 | **smoke-test** | 端到端回归 | `python ci/smoke.py` |

---

### 各模块详细设计（含单元测试）

#### M1 ipc-channel
- **职责**：提供“可靠、零拷贝、跨 OS”字节流通道，屏蔽 Unix 域套接字 / Windows 命名管道差异 |
- **公共 API** | `pub fn unbound() -> (Sender<T>, Receiver<T>)` <br> `pub fn bounded(cap: usize) -> (Sender<T>, Receiver<T>)` <br> 支持 `T: Serialize + for<'de> Deserialize<'de>` |
- **内部实现** | Unix：`seqpacket`（SOCK_SEQPACKET）；Windows：Tokio 命名管道；均以 `bincode` 编码 |
- **单测（必须）** | 1. 同进程回环 ping-pong < 1 ms <br> 2. 1 MiB 大消息零拷贝 <br> 3. 断开触发 `RecvError::Disconnected` |
- **平台 cfg** | 零业务层 cfg，差异封装在 `sys::{unix,windows}` 子模块 |
- **外依** | `tokio`,`bincode`,`serde` — 全跨 OS |

#### M2 message-defs
- **职责**：定义并生成所有跨进程消息类型，保证“改 IDL 不手工改 Rust” |
- **IDL 示例** | `struct HttpRequest { url: String }` |
- **生成脚本** | `build.rs` 调用 `codegen/src/ipdl_gen.rs` → 输出 `src/generated.rs` |
- **单测** | 1. round-trip serde 100 % 类型覆盖 <br> 2. 哈希稳定性（同 IDL 两次生成字节一致） |
- **外依** | 仅 `serde`；生成阶段用 `syn/quote`（dev-dependency） |

#### M3 network-srv
- **职责**：1. 域名解析 + DoH 可选 <br> 2. HTTP/1.1-2-3 下载 <br> 3. 返回 `HttpResponse` 给父进程 |
- **主循环** | `while let Ok(req) = rx.recv() { let resp = fetch(req).await?; tx.send(resp).await?; }` |
- **单测** | 1. Mock 本地 `wiremock` 验证头/状态码 <br> 2. 失败重试 3 次策略 <br> 3. 大文件 (>2 GiB) 分块流式传输 |
- **外依** | `reqwest`(rustls-tls),`M1`,`M2` |

#### M4 gpu-srv
- **职责**：1. 接收 `DisplayList` → wgpu 光栅化 <br> 2. 生成 `SurfaceTexture` 句柄回传 <br> 3. 支持 headless CI 截图 |
- **API 入口** | `fn render(dl: DisplayList, size: (u32,u32)) -> Result<TextureHandle,GpuError>` |
- **单测** | 1. 单红色矩形像素比对 <br> 2. 空列表不崩溃 <br> 3. 极限 8 K 尺寸 OOM 优雅降级 |
- **后端回落** | Vulkan→Metal→D3D12→llvmpipe/WARP |
- **外依** | `wgpu`,`raw-window-handle`(可选),`M1`,`M2` |

#### M5 content-srv
- **职责**：1. 封装 WebKit 内容进程（按 OS Port）与 GPU/IPC 桥接 <br> 2. 将 AI 请求路由到 AI-Runtime；引出 DOM 注入点 <br> 3. 网页渲染由 WebKit 完成 |
- **主循环** | `select! { html = network_rx => handle_html(), ai_resp = ai_rx => inject_shadow_dom(), }` |
- **单测** | 1. 纯 HTML 无 CSS 不崩溃 <br> 2. 收到 AI 响应后 DOM 节点数量 +1 <br> 3. 异常 HTML 触发错误通道 |
| 外依 | `M1`,`M2`,`M8`(注入) |

#### M6 ai-runtime
- **职责**：1. 模型仓管理（ONNX/GGUF） <br> 2. 连续批处理推理 <br> 3. 返回 `AiResponse` |
- **API 入口** | `fn run(req: AiRequest) -> Result<AiResponse,TractError>` |
- **单测** | 1. 50 token 延迟 < 800 ms <br> 2. 批量 4 条吞吐提升 ≥ 2× <br> 3. 模型文件缺失返回可识别错误码 |
- **外依** | `tract-onnx` 或 `ggml-rs`,`M1`,`M2` |

#### M7 browser-main
- **职责**：1. 拉起/监视子进程（重启策略） <br> 2. 全局服务注册表 <br> 3. 前向用户输入到对应子进程 |
- **单测** | 1. 子进程 crash 自动重启 ≤ 3 s <br> 2. 重复端口冲突优雅自增 <br> 3. 1000 次 IPC ping 无泄漏 |
- **外依** | `M1`,`M2`,`tokio::process` |

#### M8 dom-embed
- **职责**：把 AI 返回文本安全注入 DOM，支持 Shadow DOM 隔离 |
- **API 入口** | `pub fn inject_shadow(parent: &Element, html: &str) -> Result<ShadowRoot, InjectError>` |
- **单测** | 1. XSS 脚本标签被过滤 <br> 2. 重复注入 ID 自增 <br> 3. 超大文本 (>1 MiB) 不 OOM |
- **外依** | `html5ever`,`scraper`(dev) |

#### M9 window-ai-api
- **职责**：暴露 `window.ai.ask()` Promise；底层走 IPC 到 M6 |
- **编译目标** | WASM；`wasm-pack` 导出 |
- **单测** | 1. Node.js 环境 Mock M6 返回 <br> 2. 并发 10 调用无竞态 <br> 3. 拒绝非 String 输入 |
- **外依** | `wasm-bindgen`,`js-sys`,`web-sys`,`M1`(IPC) |

#### M10 servo-lite
- **职责**：HTML→DOM→Layout→DisplayList；阶段 1 只支持块级+颜色（仅用于测试/验证，不承担真实网页渲染） |
- **API 入口** | `fn layout(html: &str, css: &str) -> Result<DisplayList, LayoutError>` |
- **单测** | 1. 无 css 时宽=100 % <br> 2. 嵌套 div 深度 1024 不溢出 <br> 3. 非法 CSS 跳过不崩溃 |
- **外依** | `html5ever`,`cssparser`,`euclid`(几何) |

#### M11 build-ops
- **职责**：1. 生成本地可执行 <br> 2. 代码生成触发 <br> 3. CI 打包 |
- **子命令** | `cargo xtask dist` / `cargo xtask test` / `cargo xtask fmt-lint` |
- **单测** | 1. 生成的二进制存在且非零字节 <br> 2. 重复构建哈希一致（repro） |
- **外依** | `clap`,`xshell` |

#### M12 smoke-test
- **职责**：Python 脚本，零源码依赖， nightly 跑二进制 |
| 主要 case | 输入 `https://example.com` → 断言摘要含 “Example Domain”，并生成有效 PNG 截图 |
| 单测 | 自身用 `pytest`；Mock 子进程超时/崩溃路径 |
| 外依 | `pytest`,`requests`(拉本地 health 探针) |

---

### 交付准则

1. 每 crate `cargo test` 必须全绿，CI 三平台 nightly 阻断。  
2. 最终 `cargo xtask dist` 输出：  
   – `network-srv`  
   – `gpu-srv`  
   – `content-srv`  
   – `ai-runtime`  
   – `browser`  
3. 运行 `python ci/smoke.py` 后 10 s 内输出包含 “Example Domain” 摘要，并生成有效 PNG 截图，即阶段 1 通关。

---

