# Phase 0: AI+MCP基础框架 (8周)

## 📋 阶段目标

**核心目标**：建立AI大脑核心和基础MCP工具，实现最小可用的AI+MCP系统

- ✅ 构建AI大脑核心（意图解析、世界模型、规划引擎）
- ✅ 实现简化版MCP协议（统一物理世界接口）
- ✅ 开发3-5个基础MCP工具（摄像头、麦克风、文件系统、WebAPI）
- ✅ 整合浏览器画布（动态界面、AI思维可视化）

**用户可感知价值**：
- 用户说"帮我拍个照"，AI理解并调用摄像头工具执行
- AI通过浏览器展示思考过程（而非网页内容）
- 用户可以通过自然语言控制物理设备
- 系统展示AI从意图到执行再到结果的完整流程

**Demo场景**：
```
用户："帮我拍个照"
AI理解 → 制定计划 → 调用摄像头工具 → 显示结果

浏览器画布展示：
┌─────────────────────────────────┐
│ 📸 AI行动计划                    │
│                               │
│ 步骤1: 调用摄像头工具           │
│   ↓                            │
│ ✅ 已拍摄照片                   │
│                               │
│ 📷 拍摄结果                    │
│                               │
│ [查看照片] [再次拍摄]           │
└─────────────────────────────────┘
```

## 🎯 详细任务列表

### P0-T1: ai-core-brain - AI大脑核心 (2.5周)

**任务描述**
构建AI大脑核心组件：意图解析器、世界模型、规划引擎

**技术实现**
```rust
// crates/ai-core-brain/src/lib.rs
pub struct AICoreBrain {
    // 理解层
    intention_parser: DeepIntentionParser,
    context_analyzer: ContextAnalyzer,
    world_model: WorldModel,

    // 决策层
    planning_engine: PlanningEngine,
    tool_selector: ToolSelector,
    risk_assessor: RiskAssessor,

    // 执行层
    mcp_coordinator: MCPCoordinator,
    learning_engine: LearningEngine,
}

impl AICoreBrain {
    pub async fn new() -> Result<Self> {
        Ok(AICoreBrain {
            intention_parser: DeepIntentionParser::new().await?,
            context_analyzer: ContextAnalyzer::new(),
            world_model: WorldModel::new().await?,
            planning_engine: PlanningEngine::new(),
            tool_selector: ToolSelector::new(),
            risk_assessor: RiskAssessor::new(),
            mcp_coordinator: MCPCoordinator::new().await?,
            learning_engine: LearningEngine::new(),
        })
    }

    pub async fn process_intent(&mut self, user_input: &str) -> Result<ActionResult> {
        // 1. 深度理解用户意图
        let deep_intention = self.intention_parser.parse(user_input).await?;

        // 2. 分析当前上下文
        let context = self.context_analyzer.analyze(&deep_intention).await?;

        // 3. 获取世界状态
        let world_state = self.world_model.get_current_state().await?;

        // 4. 制定行动计划
        let plan = self.planning_engine.create_plan(
            &deep_intention,
            &context,
            &world_state,
        ).await?;

        // 5. 选择MCP工具
        let selected_tools = self.tool_selector.select_tools(&plan)?;

        // 6. 执行计划
        let result = self.mcp_coordinator.execute(&selected_tools, &plan).await?;

        // 7. 更新学习
        self.learning_engine.update(&deep_intention, &result).await?;

        Ok(ActionResult::Success(result))
    }
}
```

**意图解析器 (Intention Parser)**
```rust
// crates/ai-core-brain/src/intention_parser.rs
pub struct DeepIntentionParser {
    model: Arc<IntentModel>,
    context_extractor: ContextExtractor,
}

impl DeepIntentionParser {
    pub async fn parse(&self, input: &str) -> Result<DeepIntention> {
        // 基础意图分类
        let basic_intent = self.model.classify(input)?;

        // 上下文线索提取
        let context_clues = self.context_extractor.extract(input)?;

        // 深度意图挖掘
        match basic_intent.intent_type {
            IntentType::TakePhoto => {
                DeepIntention::PerceptionAction {
                    action: PerceptionAction::CapturePhoto,
                    parameters: self.extract_photo_params(&context_clues),
                }
            }
            IntentType::ControlDevice { device, action } => {
                DeepIntention::DeviceControl {
                    device,
                    action,
                    context: context_clues.context,
                }
            }
            IntentType::Query => {
                DeepIntention::InformationQuery {
                    query: input.to_string(),
                    sources: context_clues.sources,
                }
            }
            _ => DeepIntention::Unknown(input.to_string()),
        }
    }
}
```

**世界模型 (World Model)**
```rust
// crates/ai-core-brain/src/world_model.rs
pub struct WorldModel {
    device_states: Arc<RwLock<HashMap<DeviceId, DeviceState>>>,
    environment: Arc<RwLock<EnvironmentState>>,
    mcp_registry: Arc<RwLock<MCPRegistry>>,
}

impl WorldModel {
    pub async fn get_current_state(&self) -> Result<WorldState> {
        let devices = self.device_states.read().await.clone();
        let environment = self.environment.read().await.clone();
        let available_tools = self.mcp_registry.read().await.list_tools();

        Ok(WorldState {
            devices,
            environment,
            available_tools,
            timestamp: Utc::now(),
        })
    }

    pub async fn update_device_state(&self, device_id: &str, state: DeviceState) -> Result<()> {
        let mut states = self.device_states.write().await;
        states.insert(device_id.to_string(), state);
        Ok(())
    }
}
```

**规划引擎 (Planning Engine)**
```rust
// crates/ai-core-brain/src/planning_engine.rs
pub struct PlanningEngine {
    goal_decomposer: GoalDecomposer,
    task_scheduler: TaskScheduler,
}

impl PlanningEngine {
    pub async fn create_plan(
        &self,
        intention: &DeepIntention,
        context: &Context,
        world_state: &WorldState,
    ) -> Result<AIPlan> {
        // 1. 目标分解
        let goals = self.goal_decomposer.decompose(intention)?;

        // 2. 生成任务
        let tasks = self.generate_tasks(&goals, world_state)?;

        // 3. 调度任务
        let scheduled_tasks = self.task_scheduler.schedule(&tasks)?;

        Ok(AIPlan {
            goals,
            tasks: scheduled_tasks,
            estimated_duration: self.estimate_duration(&scheduled_tasks),
            fallback_plan: self.generate_fallback(&scheduled_tasks),
        })
    }
}
```

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 意图理解准确率 | > 90% | 100个测试用例 |
| 规划成功率 | > 85% | 自动化测试 |
| 响应延迟 | < 200ms | 性能测试 |
| 世界模型更新 | < 50ms | 状态同步测试 |
| 学习能力 | 3次交互掌握偏好 | 用户测试 |

---

### P0-T2: mcp-protocol - MCP协议实现 (2周)

**任务描述**
实现简化版MCP协议，建立工具注册表和调用机制

**技术实现**
```rust
// crates/mcp-protocol/src/lib.rs
pub struct MCPProtocol {
    // 工具注册表
    tools: Arc<RwLock<HashMap<String, Box<dyn MCPTool>>>>,

    // 传输层
    transport: Arc<dyn Transport>,

    // 权限管理
    permission_manager: PermissionManager,
}

pub trait MCPTool: Send + Sync {
    fn get_info(&self) -> ToolInfo;
    async fn invoke(&self, params: ToolParams) -> Result<ToolResult>;
    fn check_permission(&self, permission: Permission) -> bool;
}

impl MCPProtocol {
    pub fn new() -> Self {
        MCPProtocol {
            tools: Arc::new(RwLock::new(HashMap::new())),
            transport: Arc::new(LocalTransport::new()),
            permission_manager: PermissionManager::new(),
        }
    }

    pub async fn register_tool(&self, tool: Box<dyn MCPTool>) -> Result<()> {
        let info = tool.get_info();
        let mut registry = self.tools.write().await;
        registry.insert(info.id.clone(), tool);
        info!("Registered MCP tool: {}", info.name);
        Ok(())
    }

    pub async fn call_tool(
        &self,
        tool_id: &str,
        params: ToolParams,
    ) -> Result<ToolResult> {
        // 1. 查找工具
        let tools = self.tools.read().await;
        let tool = tools.get(tool_id)
            .ok_or(MCPError::ToolNotFound(tool_id.to_string()))?;

        // 2. 权限检查
        if !tool.check_permission(params.required_permission) {
            return Err(MCPError::PermissionDenied);
        }

        // 3. 执行工具
        let result = tool.invoke(params).await?;

        Ok(result)
    }

    pub async fn list_tools(&self) -> Vec<ToolInfo> {
        let tools = self.tools.read().await;
        tools.values().map(|t| t.get_info()).collect()
    }
}
```

**工具注册表**
```rust
// crates/mcp-protocol/src/registry.rs
pub struct MCPRegistry {
    tools: Arc<RwLock<HashMap<String, RegisteredTool>>>,
    categories: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl MCPRegistry {
    pub fn register(&self, tool: RegisteredTool) -> Result<()> {
        // 1. 验证工具
        self.validate_tool(&tool)?;

        // 2. 注册工具
        let mut tools = self.tools.write().await;
        tools.insert(tool.info.id.clone(), tool.clone());

        // 3. 更新分类
        let mut categories = self.categories.write().await;
        for category in &tool.info.categories {
            categories
                .entry(category.clone())
                .or_insert_with(Vec::new)
                .push(tool.info.id.clone());
        }

        Ok(())
    }

    pub async fn find_tools(&self, capability: &str) -> Vec<RegisteredTool> {
        let tools = self.tools.read().await;
        tools.values()
            .filter(|t| t.info.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect()
    }
}
```

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 工具注册成功率 | 100% | 单元测试 |
| 工具调用成功率 | > 98% | 集成测试 |
| 并发调用数 | 支持10个 | 压力测试 |
| 权限检查正确率 | 100% | 安全测试 |
| 响应延迟 | < 100ms | 性能测试 |

---

### P0-T3: 基础MCP工具开发 (2.5周)

**任务描述**
开发3-5个基础MCP工具：摄像头、麦克风、文件系统、WebAPI

**1. 摄像头工具 (Camera Tool)**
```rust
// crates/mcp-tools/src/camera.rs
pub struct CameraTool {
    device_id: String,
    capabilities: CameraCapabilities,
    permission_scope: PermissionScope,
}

impl MCPTool for CameraTool {
    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            id: "camera.default".to_string(),
            name: "Default Camera".to_string(),
            description: "Default system camera".to_string(),
            capabilities: vec![
                "capture".to_string(),
                "stream".to_string(),
            ],
            categories: vec!["perception".to_string()],
            permissions: vec![Permission::CameraAccess],
        }
    }

    async fn invoke(&self, params: ToolParams) -> Result<ToolResult> {
        let action = params.get_string("action")?;

        match action.as_str() {
            "capture" => {
                // 权限检查
                if !self.check_permission(params.required_permission) {
                    return Err(ToolError::PermissionDenied);
                }

                // 拍摄照片
                let image = self.capture_image(&params.options).await?;

                Ok(ToolResult::Image(image))
            }
            "stream" => {
                let stream = self.start_stream(&params.options)?;
                Ok(ToolResult::Stream(stream))
            }
            _ => Err(ToolError::UnsupportedAction(action)),
        }
    }
}
```

**2. 麦克风工具 (Microphone Tool)**
```rust
// crates/mcp-tools/src/microphone.rs
pub struct MicrophoneTool {
    device_id: String,
    sample_rate: u32,
}

impl MCPTool for MicrophoneTool {
    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            id: "microphone.default".to_string(),
            name: "Default Microphone".to_string(),
            capabilities: vec!["record".to_string(), "asr".to_string()],
            permissions: vec![Permission::MicrophoneAccess],
            ..Default::default()
        }
    }

    async fn invoke(&self, params: ToolParams) -> Result<ToolResult> {
        let action = params.get_string("action")?;

        match action.as_str() {
            "record" => {
                let duration = params.get_u32("duration")?;
                let audio = self.record_audio(duration).await?;
                Ok(ToolResult::Audio(audio))
            }
            "asr" => {
                let audio = params.get_audio("audio")?;
                let text = self.speech_to_text(&audio).await?;
                Ok(ToolResult::Text(text))
            }
            _ => Err(ToolError::UnsupportedAction(action)),
        }
    }
}
```

**3. 文件系统工具 (File System Tool)**
```rust
// crates/mcp-tools/src/file_system.rs
pub struct FileSystemTool {
    base_path: PathBuf,
    permissions: Vec<Permission>,
}

impl MCPTool for FileSystemTool {
    async fn invoke(&self, params: ToolParams) -> Result<ToolResult> {
        let action = params.get_string("action")?;
        let path = params.get_string("path")?;

        match action.as_str() {
            "read" => {
                let content = self.read_file(&path).await?;
                Ok(ToolResult::Text(content))
            }
            "write" => {
                let content = params.get_string("content")?;
                self.write_file(&path, &content).await?;
                Ok(ToolResult::Success("File written".to_string()))
            }
            "list" => {
                let entries = self.list_directory(&path).await?;
                Ok(ToolResult::List(entries))
            }
            _ => Err(ToolError::UnsupportedAction(action)),
        }
    }
}
```

**4. WebAPI工具 (Web API Tool)**
```rust
// crates/mcp-tools/src/web_api.rs
pub struct WebAPITool {
    endpoint: String,
    api_key: String,
    rate_limiter: RateLimiter,
}

impl MCPTool for WebAPITool {
    async fn invoke(&self, params: ToolParams) -> Result<ToolResult> {
        // 速率限制检查
        if !self.rate_limiter.check().await {
            return Err(ToolError::RateLimited);
        }

        let method = params.get_string("method")?;
        let query = params.get_string("query")?;

        // 构建请求
        let request = self.build_request(&method, &query, &params.options)?;

        // 发送请求
        let response = self.send_request(request).await?;

        // 处理响应
        let data = self.process_response(response)?;

        Ok(ToolResult::Data(data))
    }
}
```

**验收标准**
| 工具类型 | 功能 | 测试用例 | 成功率要求 |
|----------|------|----------|------------|
| **摄像头** | 拍照/录像 | 100次调用 | > 98% |
| **麦克风** | 录音/ASR | 100次调用 | > 95% |
| **文件系统** | 读/写/列目录 | 200次调用 | > 99% |
| **WebAPI** | GET/POST/查询 | 150次调用 | > 97% |

---

### P0-T4: 浏览器画布整合 (1周)

**任务描述**
整合浏览器作为AI的表达器官，展示AI思维过程

**技术实现**
```rust
// crates/browser-canvas/src/lib.rs
pub struct BrowserCanvas {
    ai_brain: Arc<RwLock<AICoreBrain>>,
    canvas_renderer: CanvasRenderer,
    event_listener: EventListener,
}

impl BrowserCanvas {
    pub fn present_ai_thought(&self, thought: AIThought) -> CanvasView {
        match thought {
            AIThought::Plan(plan) => {
                // 展示AI制定的计划
                CanvasView::Plan(PlanView {
                    title: "AI制定的行动计划".to_string(),
                    tasks: plan.tasks,
                    timeline: plan.estimated_duration,
                })
            }
            AIThought::Action(action) => {
                // 展示AI正在执行的动作
                CanvasView::Action(ActionView {
                    title: "正在执行".to_string(),
                    current_step: action.current_step,
                    progress: action.progress,
                })
            }
            AIThought::Result(result) => {
                // 展示AI执行的结果
                CanvasView::Result(ResultView {
                    title: "执行结果".to_string(),
                    success: result.success,
                    data: result.data,
                    message: result.message,
                })
            }
            AIThought::Error(error) => {
                CanvasView::Error(ErrorView {
                    title: "发生错误".to_string(),
                    error_message: error.message,
                    suggestions: error.suggestions,
                })
            }
        }
    }
}
```

**动态界面生成**
```rust
// crates/browser-canvas/src/renderer.rs
pub struct CanvasRenderer {
    template_engine: TemplateEngine,
    animation_engine: AnimationEngine,
}

impl CanvasRenderer {
    fn render_action(&self, action: &ActionView) -> CanvasView {
        CanvasView {
            title: action.title.clone(),
            elements: vec![
                Element::ProgressBar {
                    value: action.progress,
                    label: format!("步骤 {}/{}", action.current_step, action.total_steps),
                },
                Element::Animation {
                    type_: AnimationType::Loading,
                    duration: 1000,
                },
                Element::StatusMessage {
                    message: "AI正在执行操作...".to_string(),
                    status: Status::InProgress,
                },
            ],
            animations: vec![
                Animation::AnimateProgress,
                Animation::FadeInElements,
            ],
        }
    }
}
```

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 界面渲染延迟 | < 50ms | 性能测试 |
| AI思维可视化 | 100%覆盖 | 功能测试 |
| 动态界面生成 | 支持5种视图 | 单元测试 |
| 实时交互 | 响应<100ms | 集成测试 |
| 动画流畅度 | > 60FPS | 视觉测试 |

---

## 📦 模块结构

```
crates/
├── ai-core-brain/              # AI大脑核心
│   ├── src/
│   │   ├── lib.rs
│   │   ├── intention_parser.rs  # 意图解析器
│   │   ├── world_model.rs       # 世界模型
│   │   ├── planning_engine.rs  # 规划引擎
│   │   └── learning_engine.rs  # 学习引擎
│   └── Cargo.toml
├── mcp-protocol/               # MCP协议
│   ├── src/
│   │   ├── lib.rs
│   │   ├── registry.rs         # 工具注册表
│   │   ├── coordinator.rs      # 工具协调器
│   │   └── permission.rs       # 权限管理
│   └── Cargo.toml
├── mcp-tools/                  # MCP工具集合
│   ├── src/
│   │   ├── lib.rs
│   │   ├── camera.rs           # 摄像头工具
│   │   ├── microphone.rs       # 麦克风工具
│   │   ├── file_system.rs      # 文件系统工具
│   │   └── web_api.rs          # WebAPI工具
│   └── Cargo.toml
└── browser-canvas/             # 浏览器画布
    ├── src/
    │   ├── lib.rs
    │   ├── renderer.rs          # 界面渲染器
    │   ├── interaction.rs       # 交互处理
    │   └── visualization.rs     # 可视化
    └── Cargo.toml
```

## 🎬 Demo场景

### Demo-1: 基础意图执行
```
场景：用户说"帮我拍个照"

处理流程：
1. 用户输入："帮我拍个照"
2. AI大脑理解意图
   → 意图类型：PerceptionAction::CapturePhoto
   → 需要工具：camera.default
3. 制定计划
   → 步骤1: 调用摄像头工具
   → 步骤2: 获取拍摄结果
4. MCP执行
   → camera工具 → 拍摄照片
5. 浏览器展示结果
   ✅ 已拍摄照片
   📷 [照片预览]

验收：照片成功拍摄并显示
```

### Demo-2: 多工具协作
```
场景：用户说"查看这个文档，然后告诉我内容"

处理流程：
1. 用户输入："查看这个文档，然后告诉我内容"
2. AI理解意图
   → 动作1: 读取文件
   → 动作2: 分析内容
3. 制定计划
   → 步骤1: 文件系统工具读取文件
   → 步骤2: AI分析文档内容
4. MCP执行
   → file_system工具 → 读取文件
   → AI大脑 → 分析文档
5. 浏览器展示
   📄 文档已读取
   ✅ 分析完成
   💡 文档摘要显示

验收：文件读取、内容分析、结果展示全部成功
```

### Demo-3: 错误处理
```
场景：用户说"开灯"，但没有智能灯设备

处理流程：
1. 用户输入："开灯"
2. AI理解意图
   → 意图类型：DeviceControl
   → 目标设备：light
3. 查询可用工具
   → 搜索"control_light"工具
   → 未找到匹配工具
4. AI生成建议
   → 没有找到智能灯设备
   → 建议：1. 检查设备连接 2. 手动开灯
5. 浏览器展示
   ❌ 无法执行
   💡 没有找到"灯"设备
   [查看可用设备] [添加设备]

验收：错误处理友好，建议清晰
```

## ⚡ 性能指标

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| **意图理解延迟** | < 200ms | 100个测试用例平均 |
| **MCP工具调用延迟** | < 100ms | 各工具基准测试 |
| **浏览器渲染延迟** | < 50ms | 界面渲染测试 |
| **端到端响应时间** | < 300ms | 完整流程测试 |
| **并发处理能力** | 10个请求 | 并发压力测试 |

## 🎯 成功定义

### 必须达到
- ✅ 所有4个任务验收标准达标
- ✅ 3个Demo场景可正常运行
- ✅ 意图理解准确率>90%
- ✅ 基础MCP工具调用成功率>98%
- ✅ 无内存泄漏，崩溃率<0.1%

### 期望达到
- 🎯 意图理解准确率>95%
- 🎯 5个MCP工具全部可用
- 🎯 浏览器界面流畅度>60FPS
- 🎯 代码覆盖率>80%

### 超预期
- 🚀 支持10类意图类型
- 🚀 MCP工具调用延迟<50ms
- 🚀 AI思维可视化效果优秀
- 🚀 用户体验流畅自然

## ⚠️ 风险与应对

| 风险 | 概率 | 影响 | 应对策略 |
|------|------|------|----------|
| 模型理解错误 | 中 | 中 | 置信度阈值+用户确认 |
| 工具兼容性 | 中 | 中 | 标准化接口+fallback |
| 性能不达标 | 低 | 高 | 缓存优化+懒加载 |
| 权限问题 | 高 | 高 | 最小权限+用户教育 |

---

**Phase 0总结：建立AI+MCP基础框架，让AI能够理解意图并调用物理世界工具！** ✅
