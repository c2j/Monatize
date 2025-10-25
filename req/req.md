# Monazite 浏览器 ‑ Total 需求规格书
**版本**：v1.0 2025-06-25  
**状态**：阶段 0/1/2/3 已评审 | 阶段 4 预留  

---

## 1. 产品定位
- **名称**：Monazite（独居石，稀土矿）
- **Slogan**：Monazite / AI-Native / Rust-first / Cross-OS
- **目标用户**：  
  - C 端：追求性能、隐私、AI 辅助的极客与专业用户  
  - B 端：需要可定制、可审计、企业策略的政企市场  
- **差异化**：  
  - 渲染引擎：WebKit（按 OS 选择 Port），UI 框架：GPUI（Rust）
  - 端侧大模型常驻（默认 1–3B，可选 7B 独立下载）
  - 全链路 Rust（除 WebKit）（→ 内存安全、可审计）  
  - 企业可 100 % 离线部署（模型+更新+策略）  

---

## 2. 顶层功能总览（L0）

| 域 | 子系统 | 必须 | 阶段 |
|----|--------|------|------|
| 内核 | 多进程框架 | ✅ | 0-1 |
| 渲染 | WebKit（按 OS Port）合成 | ✅ | 0-1 |
| UI | GPUI 原生窗口 | ✅ | 0-1 |
| AI | 端侧推理引擎 | ✅ | 1 |
| 网络 | DoH/HTTP3/缓存 | ✅ | 1 |
| 安全 | Site-Isolation+Sandbox | ✅ | 2 |
| 扩展 | WebExtension 兼容 | ✅ | 2 |
| PWA | SW+Push+安装 | ✅ | 3 |
| 同步 | 端到端加密同步 | ✅ | 3 |
| 企业 | 策略/离线更新/签名 | ✅ | 3 |
| 性能 | Profiler+电源+内存调优 | ✅ | 3 |
| 质量 | 崩溃报告+遥测（可关） | ✅ | 3 |

---

## 3. 详细需求（L1-L3）

### 3.1 内核与进程模型
- **多进程**：Browser / Renderer / GPU / Network / Extension / AI / RDD（字体/解码）  
- **站点隔离**：每标签/每站点进程 + 导航时进程交换（按 Port 能力）；支持进程数上限与回收策略
- **IPC**：自研 Rust crate（M1），Unix seqpacket + Windows 命名管道，零拷贝 < 1 ms  
- **调度模型**：Browser 主线程 Reactor + 线程池；子进程健康心跳 + 3 s 重启  
- **线程安全**：所有 IPC 消息 `Send + Sync`，无裸指针跨进程  

### 3.2 渲染与合成
- **引擎**：WebKit（按 OS Port；Linux：WebKitGTK 或 WPE；macOS：WKWebView；Windows：WinCairo），硬件合成
- **UI 框架**：GPUI（Zed 衍生）wgpu 同源后端 → 纹理共享零拷贝为目标，提供拷贝/映射回退
- **多标签合成**：1080p 10 标签 60 FPS；4K 3–5 标签 60 FPS（平台相关）
- **输入事件**：≤ 8 ms 端到端；支持手势、触控、笔

### 3.3 网络与缓存
- **网络栈归属**：网页资源沿用 WebKit 原生网络栈；浏览器级下载/扩展/AI 使用自研 network-srv（可选 DoH/HTTP3）
- **协议**：HTTP/1.1-2-3，QUIC，WebSocket，DoH/DoT
- **缓存**：强缓存 + 协商缓存 + CacheStorage（SW）
- **离线**：ServiceWorker + CacheStorage + Push（VAPID）
- **下载**：多线程 + 断点续传 + 沙盒 IO（仅下载目录可写）

### 3.4 AI 引擎
- **模型格式**：ONNX / GGUF / Safetensors  
- **推理后端**：tract-onnx（CPU）+ ggml（GPU/NPU）  
- **常驻模型**：默认 1–3B；7B 为可选 Pro 档；首 token SLO 按硬件分档
- **AI API**：`window.ai.ask()` / `window.ai.translate()` / `window.ai.summarize()`  
- **内存**：mmap + 压缩 + 电池模式卸载  

### 3.5 安全与隐私
- **Sandbox**：  
  - Linux：seccomp-bpf + namespace + no-new-privs  
  - macOS：seatbelt + nosuid + JIT hardening  
  - Windows：Win32k Lockdown + LPAC  
- **站点隔离**：Spectre 防护，XSite 文档不可互访  
- **证书**：CT 日志校验 + EV 标签 + 企业根证书注入  
- **权限**：camera/mic/geo/通知/ cookie / 自动播放 逐项询问 + 策略锁定  
- **加密**：优先 TLS 1.3，兼容 TLS 1.2（可由企业策略关闭回退），HSTS 预加载，DoH
- **签名**：内核与更新包 Ed25519 + cosign 验证  

### 3.6 扩展与生态
- **规范**：兼容 WebExtension MV3  
- **宿主**：独立进程 + WASM sandbox + 资源上限  
- **API**：tabs / runtime / storage / alarms / fetch / notification  
- **商店**：自建市场 + 签名审核 + 企业旁路加载  
- **UserScript**：GM_* 兼容，隔离世界  

### 3.7 PWA 与安装
- **Manifest**：解析、图标生成、桌面快捷方式  
- **ServiceWorker**：生命周期、更新、后台同步  
- **Push**：VAPID 订阅、本地通知、点击启动 PWA  
- **离线启动**：无网络打开 PWA 仍可用  

### 3.8 登录与同步
- **账号**：OAuth2 + WebAuthn（Passkey）  
- **加密**：RSA-OAEP + AES-GCM，端到端，服务器零明文  
- **数据类型**：书签、历史、密码、扩展、偏好、主题  
- **冲突**：三路合并 + 时间戳 + 用户选择  
- **设备数**：≤ 10 台，可远程登出  

### 3.9 企业策略
- **格式**：Chrome ADMX 兼容 JSON + 自家 Schema  
- **下发**：云 MDM + 本地注册表 + GPO  
- **锁定**：UI 灰显、不可修改、不可卸载  
- **审计**：操作日志加密回传（可关）  

### 3.10 性能与电源
- **启动**：冷启动 < 1.5 s（SSD）  
- **内存**：单空白标签 ≈ 150–200 MB；10 标签压缩后 ≤ 1.2–1.5 GB
- **GPU**：电源模式自动 30 FPS；电池模式 AI batch=1  
- **调优**：后台标签 5 min 冻结；内存压缩 zstd 50 %  
- **Profiler**：内置 Tracy，10 s 录制，火焰图一键导出  

### 3.11 更新与签名
- **通道**：Stable / Beta / Dev / LTS（企业）  
- **差分**：bsdiff + zstd，平均节省 80 % 流量  
- **回滚**：失败自动回退，原子 rename  
- **签名**：Ed25519 + cosign 透明签名验证  
- **离线包**：企业可导入离线更新包  

### 3.12 质量与遥测
- **崩溃**：breakpad / minidump / 本地符号服务器  
- **遥测**：匿名指标 + 用户一键关闭 + 企业策略禁用  
- **LTS**：24 月安全更新；扩展兼容冻结；内核功能回port  

### 3.13 可访问性与国际化
- **A11y**：完整无障碍树，支持屏幕阅读器，键盘遍历  
- **i18n**：语言包市场，WebExtension i18n API 兼容  
- **主题**：暗色/浅色/高对比度 + 用户 CSS 覆盖  

### 3.14 构建与交付
- **语言**：Rust ≥ 1.80（edition 2021），少量 C++（WebKit）  
- **构建**：CMake + cargo-xtask；Rust 工程 CI < 45 min；WebKit 依赖使用系统/预编译，不在 CI 全量构建
- **包格式**：  
  - Windows：MSI + AppX + 便携 zip  
  - macOS：dmg + pkg（MAS）  
  - Linux：deb/rpm/AppImage/flatpak  
- **体积**：核心安装包 ≤ 120 MB（不含模型）；模型按档位独立下载
  - Lite（≤500 MB）：1–3B 量化（翻译/摘要）
  - Standard（1–2 GB）：3–4B 量化
  - Pro（≥4 GB）：7B 量化（可选）
- **依赖**：系统仅要求 Vulkan/Metal/D3D12 驱动 + fonts  

---

## 4. 阶段划分与里程碑

| 阶段 | 时间 | 核心交付 | 验收标准 |
|------|------|----------|----------|
| 0 | 4 周 | WebKit ⇄ GPUI 纹理闭环 | 弹出红色方块窗口 + 单元测试全绿 |
| 1 | 3 月 | 多进程 + AI 摘要 + 网页打开 | `python smoke-1.py` 10 s 出摘要 |
| 2 | 6 周 | 多标签 + 站点隔离 + 扩展骨架 | 双标签像素不同 + 扩展拦截日志 |
| 3 | 6 周 | PWA + 同步 + 企业策略 + 性能 | 离线 PWA 启动 + 登录同步成功 |
| 4 | 3 月 | AI 2.0 + 插件市场 + LTS 分支 | 市场安装扩展 ≥ 100 个零崩溃 |

---

## 5. 合规与授权

- **内核**：WebKit（按 OS Port） LGPL（动态链接）
- **源码**：Monazite 内核 MPL-2.0，允许闭源打包；WebExtension 及主题按各自许可证
- **隐私**：GDPR / CCPA 就绪，用户数据本地加密，可 100 % 离线运行  
- **加密出口**：使用 Rust-Crypto，符合 US EAR 742.15(b)(1) 豁免  

---

## 6. 维护与 SLA（LTS 版）

- **安全更新**：每月补丁，高危 7 天内  
- **功能回port**：每季度筛选稳定功能 cherry-pick  
- **支持周期**：24 个月 + 12 个月 ESR 过渡  
- **企业支持**：邮件/IM 48 h 响应，热补丁通道  

---

## 7. 附录 A：性能基线

| 场景 | 目标值 | 工具 |
|------|--------|------|
| 冷启动 | ≤ 1.5 s | Tracy |
| 新标签 | ≤ 120 ms | Tracy |
| 首帧 WebGL | ≤ 50 ms | Tracy |
| 内存 10 标签 | ≤ 600 MB | RSS |
| 电池模式 FPS | 30 ± 2 | PowerTOP |
| AI 首 token | ≤ 300 ms（Pro 硬件）/ ≤ 1200 ms（CPU） | Tracy |
| 差分更新 | ≤ 20 % 全量大小 | bsdiff |

---

## 8. 附录 B：文件/目录规范

```
Monazite/
├─ bin/monazite                          # 主程序
├─ lib/monazite-{renderer,gpu,network}  # 子进程
├─ share/monazite/                      # 静态资源
│  ├─ themes/                           # 主题包
│  ├─ models/                           # AI 模型
│  └─ extensions/                       # 扩展
├─ etc/monazite/policy.json             # 企业策略
└─ var/cache/monazite/                  # 缓存 & GPU 纹理
```

---

## 9. 附录 C：命令行与开关

```bash
monazite --no-sandbox           # 调试
monazite --headless             # CI
monazite --disable-ai           # 合规
monazite --policy-file=corp.json # 企业
monazite --profiling            # 内置 profiler
```

---

## 10. 结论

本需求文档覆盖 Monazite 浏览器从阶段 0 到 LTS 的**全部功能、性能、安全、合规、交付与维护要求**，每个条目均可向下拆解为独立 crate 与单元测试，支持跨 OS、离线部署与企业定制。