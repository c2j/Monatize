# Phase 1: MCP工具生态 (8周)

## 📋 阶段目标

**核心目标**：扩展MCP工具生态，支持智能家居、IoT设备、企业系统等物理世界交互

- ✅ 构建智能家居工具生态（灯光、空调、门锁、窗帘等）
- ✅ IoT设备全面接入（传感器、电器、自动化场景）
- ✅ 企业系统工具开发（CRM、ERP、OA系统接入）
- ✅ MCP工具协调器优化（并发执行、错误恢复、性能优化）
- ✅ 跨设备场景编排（多设备协同、复杂自动化流程）

**用户可感知价值**：
- 用户说"家里有点冷"，AI自动调节空调温度
- 用户说"查看家里情况"，AI调用摄像头+传感器生成安全报告
- 用户说"明早8点叫我起床"，AI设置智能家居闹钟场景
- 企业用户说"查询客户数据"，AI直接操作CRM系统并生成报表

## 🎯 详细任务列表

### P1-T1: 智能家居工具开发 (2周)

**任务描述**
开发智能家居设备控制工具，构建完整家庭自动化生态

**技术实现**
```rust
// crates/mcp-tools/src/smart_home.rs
pub struct SmartHomeMCP {
    // 设备管理器
    device_manager: DeviceManager,

    // 场景编排器
    scene_orchestrator: SceneOrchestrator,

    // 自动化引擎
    automation_engine: AutomationEngine,
}

pub struct SmartHomeDevice {
    pub device_id: String,
    pub device_type: DeviceType,
    pub capabilities: Vec<DeviceCapability>,
    pub status: DeviceStatus,
}

// 设备类型定义
#[derive(Debug, Clone)]
pub enum DeviceType {
    Light(LightDevice),
    Thermostat(ThermostatDevice),
    Lock(LockDevice),
    Curtain(CurtainDevice),
    Camera(CameraDevice),
    Sensor(SensorDevice),
}

impl MCPTool for SmartHomeDevice {
    async fn invoke(&self, params: ToolParams) -> Result<ToolResult> {
        let action = params.get_string("action")?;

        match action.as_str() {
            "turn_on" => {
                let device = params.get_string("device")?;
                self.turn_on(&device).await?;
                Ok(ToolResult::Success("设备已开启".to_string()))
            }
            "set_brightness" => {
                let brightness = params.get_u32("brightness")?;
                self.set_brightness(brightness).await?;
                Ok(ToolResult::Success("亮度已调节".to_string()))
            }
            "set_temperature" => {
                let temperature = params.get_f32("temperature")?;
                self.set_temperature(temperature).await?;
                Ok(ToolResult::Success(format!("温度设置为 {}°C", temperature)))
            }
            "create_scene" => {
                let scene_name = params.get_string("scene")?;
                let actions = params.get_array("actions")?;
                self.create_scene(&scene_name, actions).await?;
                Ok(ToolResult::Success(format!("场景已创建: {}", scene_name)))
            }
            _ => Err(ToolError::UnsupportedAction(action)),
        }
    }
}

// 场景编排器
pub struct SceneOrchestrator {
    scenes: Arc<RwLock<HashMap<String, Scene>>>,
    active_scene: Arc<RwLock<Option<String>>>,
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub name: String,
    pub actions: Vec<SceneAction>,
    pub triggers: Vec<SceneTrigger>,
}

#[derive(Debug, Clone)]
pub enum SceneAction {
    DeviceControl {
        device_id: String,
        action: String,
        params: HashMap<String, Value>,
    },
    Wait { duration: Duration },
    Conditional {
        condition: Condition,
        then_actions: Vec<SceneAction>,
        else_actions: Vec<SceneAction>,
    },
}

impl SceneOrchestrator {
    pub async fn execute_scene(&self, scene_name: &str) -> Result<()> {
        let scenes = self.scenes.read().await;
        let scene = scenes.get(scene_name)
            .ok_or(MCPError::SceneNotFound(scene_name.to_string()))?;

        for action in &scene.actions {
            self.execute_action(action).await?;
        }

        Ok(())
    }
}

// 自动化引擎
pub struct AutomationEngine {
    rules: Arc<RwLock<Vec<AutomationRule>>>,
    event_bus: EventBus,
}

#[derive(Debug, Clone)]
pub struct AutomationRule {
    pub id: String,
    pub name: String,
    pub trigger: TriggerCondition,
    pub conditions: Vec<Condition>,
    pub actions: Vec<SceneAction>,
    pub enabled: bool,
}

impl AutomationEngine {
    pub async fn evaluate_rules(&self, event: &DeviceEvent) -> Result<()> {
        let rules = self.rules.read().await;

        for rule in rules {
            if rule.enabled && self.matches_trigger(&rule.trigger, event) {
                self.execute_rule(&rule).await?;
            }
        }

        Ok(())
    }
}
```

**智能家居工具能力**

| 设备类型 | 支持操作 | 技术实现 |
|----------|----------|----------|
| **灯光** | 开关、亮度、色温、场景 | Zigbee/Z-Wave协议 |
| **空调** | 开关、温度、风速、模式 | 红外+WiFi双模 |
| **门锁** | 开锁、关锁、密码管理 | 蓝牙+指纹识别 |
| **窗帘** | 开合、百分比控制 | 电机控制+传感器 |
| **摄像头** | 拍摄、录像、云台控制 | RTSP/HTTP协议 |
| **传感器** | 温度、湿度、光照、运动 | IoT网关聚合 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 设备类型支持 | > 20种 | 功能测试 |
| 设备兼容率 | > 90% | 200台设备测试 |
| 场景执行成功率 | > 98% | 500次场景测试 |
| 响应延迟 | < 500ms | 性能测试 |
| 自动化准确率 | > 95% | 真实场景测试 |

---

### P1-T2: IoT设备集成 (2周)

**任务描述**
统一接入各类IoT设备，构建完整的物联网生态系统

**技术实现**
```rust
// crates/mcp-tools/src/iot_integration.rs
pub struct IoTIntegrationMCP {
    // 设备发现
    device_discovery: DeviceDiscovery,

    // 协议适配器
    protocol_adapters: HashMap<ProtocolType, Box<dyn ProtocolAdapter>>,

    // 设备管理器
    device_registry: DeviceRegistry,

    // 数据聚合
    data_aggregator: DataAggregator,
}

pub enum ProtocolType {
    Zigbee,
    ZWave,
    WiFi,
    Bluetooth,
    MQTT,
    CoAP,
}

// 协议适配器接口
pub trait ProtocolAdapter: Send + Sync {
    async fn discover_devices(&self) -> Result<Vec<IoTDevice>>;
    async fn connect_device(&self, device_id: &str) -> Result<()>;
    async fn send_command(&self, device_id: &str, command: &DeviceCommand) -> Result<()>;
    async fn read_sensor(&self, device_id: &str, sensor_type: &str) -> Result<SensorData>;
    fn get_protocol_type(&self) -> ProtocolType;
}

// 设备发现
pub struct DeviceDiscovery {
    scanners: Vec<Box<dyn DeviceScanner>>,
    discovery_cache: Arc<RwLock<HashMap<String, DiscoveredDevice>>>,
}

impl DeviceDiscovery {
    pub async fn discover_all(&self) -> Result<Vec<IoTDevice>> {
        let mut all_devices = Vec::new();

        for scanner in &self.scanners {
            let devices = scanner.scan().await?;
            all_devices.extend(devices);
        }

        // 去重和过滤
        let unique_devices = self.deduplicate_devices(all_devices);
        Ok(unique_devices)
    }
}

// IoT设备定义
pub struct IoTDevice {
    pub device_id: String,
    pub device_type: IoTDeviceType,
    pub protocol: ProtocolType,
    pub capabilities: Vec<DeviceCapability>,
    pub status: DeviceStatus,
    pub location: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IoTDeviceType {
    Sensor(SensorType),
    Actuator(ActuatorType),
    Appliance(ApplianceType),
    SecurityDevice(SecurityDeviceType),
}

// 传感器类型
#[derive(Debug, Clone)]
pub enum SensorType {
    Temperature,
    Humidity,
    Light,
    Motion,
    AirQuality,
    Noise,
    Power,
    WaterLeak,
}

// 执行器类型
#[derive(Debug, Clone)]
pub enum ActuatorType {
    Switch,
    Dimmer,
    Valve,
    Motor,
    Speaker,
    Display,
}

impl MCPTool for IoTDevice {
    async fn invoke(&self, params: ToolParams) -> Result<ToolResult> {
        let action = params.get_string("action")?;

        match action.as_str() {
            "read_sensor" => {
                let sensor_type = params.get_string("sensor_type")?;
                let data = self.read_sensor_data(&sensor_type).await?;
                Ok(ToolResult::SensorData(data))
            }
            "send_command" => {
                let command = params.get_string("command")?;
                let value = params.get("value")?;
                self.send_control_command(&command, value).await?;
                Ok(ToolResult::Success("命令已发送".to_string()))
            }
            "get_status" => {
                let status = self.get_device_status().await?;
                Ok(ToolResult::Status(status))
            }
            "configure" => {
                let config = params.get_object("config")?;
                self.configure_device(config).await?;
                Ok(ToolResult::Success("配置已更新".to_string()))
            }
            _ => Err(ToolError::UnsupportedAction(action)),
        }
    }
}
```

**IoT设备能力**

| 设备类别 | 协议支持 | 典型设备 | 数据频率 |
|----------|----------|----------|----------|
| **传感器** | Zigbee/Z-Wave/WiFi | 温湿度、光照、运动 | 1s-60s |
| **执行器** | Zigbee/Z-Wave/BLE | 开关、阀门、电机 | 实时 |
| **家电** | WiFi/MQTT | 洗衣机、冰箱、电视 | 事件驱动 |
| **安防** | WiFi/BLE/Zigbee | 门磁、摄像头、报警器 | 实时 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 协议支持 | > 6种 | 协议测试套件 |
| 设备发现率 | > 95% | 200台设备测试 |
| 数据准确性 | > 99% | 对照测试 |
| 连接稳定性 | > 98% | 7x24测试 |
| 自动化场景 | > 100种 | 场景库测试 |

---

### P1-T3: 企业系统工具开发 (2周)

**任务描述**
接入企业级系统（CRM、ERP、OA），实现企业流程自动化

**技术实现**
```rust
// crates/mcp-tools/src/enterprise_systems.rs
pub struct EnterpriseSystemsMCP {
    // 系统连接器
    crm_connector: CRMConnector,
    erp_connector: ERPConnector,
    oa_connector: OAConnector,

    // 数据转换器
    data_transformer: DataTransformer,

    // 业务引擎
    business_engine: BusinessEngine,
}

// CRM系统连接器
pub struct CRMConnector {
    api_endpoint: String,
    auth_token: String,
    rate_limiter: RateLimiter,
}

impl CRMConnector {
    pub async fn query_customer(&self, query: &CustomerQuery) -> Result<Vec<Customer>> {
        // 1. 查询参数转换
        let api_query = self.transform_to_api_query(query)?;

        // 2. API调用
        let response = self.call_crm_api(&api_query).await?;

        // 3. 数据转换
        let customers = self.transform_from_api_response(&response)?;

        Ok(customers)
    }

    pub async fn create_lead(&self, lead_data: &LeadData) -> Result<String> {
        let payload = serde_json::to_string(lead_data)?;
        let response = self.post("/api/leads", &payload).await?;
        Ok(response.lead_id)
    }
}

// ERP系统连接器
pub struct ERPConnector {
    connection_pool: ConnectionPool,
    query_cache: Arc<RwLock<LruCache<String, QueryResult>>>,
}

impl ERPConnector {
    pub async fn get_inventory(&self, product_id: &str) -> Result<InventoryStatus> {
        let cache_key = format!("inventory:{}", product_id);

        // 尝试缓存
        if let Some(cached) = self.query_cache.read().get(&cache_key) {
            return Ok(cached.clone());
        }

        // 查询ERP系统
        let inventory = self.query_inventory_from_erp(product_id).await?;

        // 更新缓存
        self.query_cache.write().insert(cache_key, inventory.clone());

        Ok(inventory)
    }

    pub async fn create_sales_order(&self, order: &SalesOrder) -> Result<String> {
        let transaction_id = self.begin_transaction().await?;

        try {
            let order_id = self.insert_sales_order(order).await?;
            self.commit_transaction(transaction_id).await?;
            Ok(order_id)
        } catch (e) {
            self.rollback_transaction(transaction_id).await?;
            Err(e)
        }
    }
}

impl MCPTool for EnterpriseSystemsMCP {
    async fn invoke(&self, params: ToolParams) -> Result<ToolResult> {
        let system_type = params.get_string("system")?;
        let action = params.get_string("action")?;

        match system_type.as_str() {
            "crm" => {
                match action.as_str() {
                    "query_customer" => {
                        let query = params.get_object("query")?;
                        let customers = self.crm_connector.query_customer(&query).await?;
                        Ok(ToolResult::Data(customers))
                    }
                    "create_lead" => {
                        let lead_data = params.get_object("lead_data")?;
                        let lead_id = self.crm_connector.create_lead(&lead_data).await?;
                        Ok(ToolResult::Id(lead_id))
                    }
                    _ => Err(ToolError::UnsupportedAction(action)),
                }
            }
            "erp" => {
                match action.as_str() {
                    "get_inventory" => {
                        let product_id = params.get_string("product_id")?;
                        let inventory = self.erp_connector.get_inventory(&product_id).await?;
                        Ok(ToolResult::Data(inventory))
                    }
                    "create_order" => {
                        let order = params.get_object("order")?;
                        let order_id = self.erp_connector.create_sales_order(&order).await?;
                        Ok(ToolResult::Id(order_id))
                    }
                    _ => Err(ToolError::UnsupportedAction(action)),
                }
            }
            "oa" => {
                match action.as_str() {
                    "submit_approval" => {
                        let request = params.get_object("request")?;
                        let result = self.oa_connector.submit_approval(&request).await?;
                        Ok(ToolResult::Data(result))
                    }
                    "get_todo_list" => {
                        let user_id = params.get_string("user_id")?;
                        let todos = self.oa_connector.get_todo_list(&user_id).await?;
                        Ok(ToolResult::Data(todos))
                    }
                    _ => Err(ToolError::UnsupportedAction(action)),
                }
            }
            _ => Err(ToolError::UnsupportedSystem(system_type)),
        }
    }
}
```

**企业系统能力**

| 系统类型 | 主要功能 | 数据同步 | API限制 |
|----------|----------|----------|---------|
| **CRM** | 客户管理、销售漏斗、线索跟进 | 实时/准实时 | 1000次/分钟 |
| **ERP** | 库存管理、订单处理、财务 | 定时同步 | 500次/分钟 |
| **OA** | 审批流程、考勤、文档 | 实时 | 2000次/分钟 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 系统接入数 | > 5种 | 集成测试 |
| API调用成功率 | > 99% | 压测测试 |
| 数据同步延迟 | < 5s | 性能测试 |
| 并发处理能力 | > 100 TPS | 压力测试 |
| 业务场景覆盖 | > 20种 | 场景测试 |

---

### P1-T4: MCP工具协调器优化 (2周)

**任务描述**
优化MCP工具的并发执行、错误恢复、性能监控等能力

**技术实现**
```rust
// crates/mcp-protocol/src/coordinator.rs
pub struct MCPToolCoordinator {
    // 工具池
    tool_pool: Arc<RwLock<ToolPool>>,

    // 执行调度器
    scheduler: TaskScheduler,

    // 错误恢复
    error_recovery: ErrorRecovery,

    // 性能监控
    metrics_collector: MetricsCollector,
}

pub struct ToolPool {
    tools: HashMap<String, PooledTool>,
    available_tools: Arc<RwLock<HashMap<String, Vec<PooledTool>>>>,
    max_pool_size: usize,
}

// 任务调度器
pub struct TaskScheduler {
    task_queue: Arc<crossbeam::queue::SegQueue<Task>>,
    worker_threads: Vec<JoinHandle<()>>,
    concurrency_limit: Arc<Semaphore>,
}

impl TaskScheduler {
    pub async fn schedule(&self, tasks: Vec<Task>) -> Vec<TaskHandle> {
        let mut handles = Vec::new();

        for task in tasks {
            // 并发控制
            self.concurrency_limit.acquire().await;

            let handle = self.spawn_worker(task);
            handles.push(handle);
        }

        handles
    }

    fn spawn_worker(&self, task: Task) -> TaskHandle {
        let semaphore = self.concurrency_limit.clone();
        let task_queue = self.task_queue.clone();

        tokio::spawn(async move {
            defer {
                semaphore.add_permits(1);
            }

            // 执行任务
            let result = task.execute().await;

            // 记录指标
            metrics::record_task_completion(&task, &result);
        })
    }
}

// 错误恢复
pub struct ErrorRecovery {
    retry_policies: HashMap<String, RetryPolicy>,
    fallback_strategies: HashMap<String, FallbackStrategy>,
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_retries: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f32,
}

impl ErrorRecovery {
    pub async fn execute_with_retry<T>(
        &self,
        task_id: &str,
        operation: impl Fn() -> Result<T>,
    ) -> Result<T> {
        let policy = self.retry_policies.get(task_id)
            .unwrap_or(&DefaultRetryPolicy::default());

        let mut attempt = 0;
        let mut delay = policy.base_delay;

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) if attempt < policy.max_retries => {
                    attempt += 1;
                    tokio::time::sleep(delay).await;
                    delay = min(delay * policy.backoff_multiplier, policy.max_delay);
                }
                Err(e) => {
                    // 尝试fallback
                    if let Some(strategy) = self.fallback_strategies.get(task_id) {
                        return strategy.execute(&e);
                    }
                    return Err(e);
                }
            }
        }
    }
}

// 性能监控
pub struct MetricsCollector {
    metrics: Arc<Mutex<HashMap<String, TaskMetrics>>>,
    export_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub total_invocations: u64,
    pub successful_invocations: u64,
    pub failed_invocations: u64,
    pub average_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
}

impl MetricsCollector {
    pub fn record_invocation(&self, tool_id: &str, latency: Duration, success: bool) {
        let mut metrics = self.metrics.lock().unwrap();

        if let Some(m) = metrics.get_mut(tool_id) {
            m.total_invocations += 1;
            if success {
                m.successful_invocations += 1;
            } else {
                m.failed_invocations += 1;
            }
            m.update_latency(latency);
        } else {
            metrics.insert(tool_id.to_string(), TaskMetrics::new(latency, success));
        }
    }

    pub fn get_metrics(&self, tool_id: &str) -> Option<TaskMetrics> {
        self.metrics.lock().unwrap().get(tool_id).cloned()
    }
}
```

**协调器能力**

| 能力 | 说明 | 技术实现 |
|------|------|----------|
| **并发控制** | 限制同时执行的工具数量 | Semaphore + 线程池 |
| **负载均衡** | 根据负载分配任务 | 加权轮询算法 |
| **错误恢复** | 自动重试 + 降级 | 指数退避 + Fallback |
| **性能监控** | 实时指标收集 | 内存缓存 + 定期导出 |
| **熔断器** | 防止故障扩散 | 失败率阈值 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 并发工具数 | > 20个 | 压力测试 |
| 错误恢复率 | > 95% | 故障注入测试 |
| 性能提升 | > 3x | 对比测试 |
| 监控覆盖 | 100% | 指标验证 |
| 稳定性 | > 99.9% | 长期运行测试 |

---

## 📦 模块结构

```
crates/mcp-tools/
├── src/
│   ├── lib.rs
│   ├── smart_home/              # 智能家居工具
│   │   ├── lighting.rs          # 灯光控制
│   │   ├── climate.rs           # 空调控制
│   │   ├── security.rs          # 安防设备
│   │   └── scenes.rs            # 场景编排
│   ├── iot_integration/         # IoT设备集成
│   │   ├── protocols.rs         # 协议适配器
│   │   ├── device_discovery.rs  # 设备发现
│   │   └── data_aggregation.rs  # 数据聚合
│   ├── enterprise_systems/      # 企业系统
│   │   ├── crm.rs               # CRM系统
│   │   ├── erp.rs               # ERP系统
│   │   └── oa.rs                # OA系统
│   └── coordinator.rs           # 工具协调器
└── Cargo.toml
```

## 🎬 Demo场景

### Demo-1: 智能家居控制
```
场景：用户说"准备睡觉"

处理流程：
1. 用户输入："准备睡觉"
2. AI理解意图 → 睡眠场景
3. 制定计划
   → 关闭客厅灯光
   → 调节卧室温度到24°C
   → 关闭电视和音响
   → 启动安防模式
4. MCP执行
   → smart-home工具 → 设备联动
5. 浏览器展示
   🛏️ 晚安场景已启动
   ✅ 灯光已关闭
   ✅ 温度已调节
   ✅ 安防已启动

验收：多设备联动成功，场景执行流畅
```

### Demo-2: IoT设备监控
```
场景：用户说"检查工厂状态"

处理流程：
1. 用户输入："检查工厂状态"
2. AI理解意图 → 设备巡检
3. 制定计划
   → 读取所有传感器数据
   → 分析设备状态
   → 生成巡检报告
4. MCP执行
   → iot-integration工具 → 批量读取
   → AI分析 → 状态评估
5. 浏览器展示
   📊 工厂状态报告
   ✅ 温度：25°C (正常)
   ✅ 湿度：60% (正常)
   ⚠️ 3号机器振动异常

验收：多设备数据聚合，智能分析准确
```

### Demo-3: 企业系统自动化
```
场景：用户说"创建销售订单"

处理流程：
1. 用户输入："创建销售订单"
2. AI理解意图 → 订单创建
3. 制定计划
   → 查询客户信息（CRM）
   → 检查库存（ERP）
   → 创建订单（ERP）
   → 发送审批（OA）
4. MCP执行
   → crm工具 → 查询客户
   → erp工具 → 检查库存
   → erp工具 → 创建订单
   → oa工具 → 提交审批
5. 浏览器展示
   📝 订单创建成功
   ✅ 客户信息已获取
   ✅ 库存检查完成
   ✅ 订单#12345已创建
   📋 审批流程已启动

验收：跨系统数据流转，流程自动化
```

## ⚡ 性能指标

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| **工具注册成功率** | 100% | 工具注册测试 |
| **工具调用成功率** | > 98% | 10万次调用测试 |
| **平均响应延迟** | < 500ms | 各工具基准测试 |
| **并发处理能力** | > 20个工具 | 并发压力测试 |
| **设备兼容性** | > 90%主流设备 | 设备库测试 |

## 🎯 成功定义

### 必须达到
- ✅ 所有4个任务验收标准达标
- ✅ 3个Demo场景可正常运行
- ✅ 工具注册成功率100%
- ✅ MCP工具调用成功率>98%
- ✅ 支持>20种IoT设备

### 期望达到
- 🎯 支持5个企业系统接入
- 🎯 设备兼容性>95%
- 🎯 场景编排能力>100种
- 🎯 性能提升>3x

### 超预期
- 🚀 MCP工具>100个
- 🚀 IoT设备类型>50种
- 🚀 跨设备场景编排
- 🚀 企业系统深度集成
            text: summary,
            key_points,
            confidence: self.calculate_confidence(&key_points),
            reading_time: self.estimate_reading_time(&summary),
        })
    }

    pub async fn explain_term(&self, term: &str, context: &PageContext) -> Explanation {
        // 1. 识别术语类型
        let term_type = self.classify_term(term, &context);

        // 2. 检索相关信息
        let related_info = context.find_related_info(term);

        // 3. 生成解释
        let explanation = match term_type {
            TermType::Technical => self.generate_technical_explanation(term, &related_info),
            TermType::Concept => self.generate_conceptual_explanation(term, &related_info),
            TermType::Code => self.generate_code_explanation(term, &related_info),
        };

        // 4. 生成示例
        let examples = self.generate_examples(term, term_type);

        // 5. 相关推荐
        let related_terms = self.recommend_related_terms(term);

        Explanation {
            term: term.to_string(),
            explanation,
            examples,
            related_terms,
            difficulty_level: self.assess_difficulty(term, &context),
        }
    }

    pub async fn generate_code_suggestion(&self, code_context: &CodeContext) -> CodeSuggestion {
        // 代码上下文理解
        let language = self.detect_language(&code_context.text);
        let api_usage = self.analyze_api_usage(&code_context.text);

        // 生成建议
        let suggestions = match code_context.request_type {
            RequestType::Completion => {
                self.generate_completions(&code_context)
            }
            RequestType::Explanation => {
                self.explain_code(&code_context)
            }
            RequestType::Optimization => {
                self.optimize_code(&code_context)
            }
        };

        CodeSuggestion {
            original_code: code_context.text.clone(),
            suggestions,
            confidence: self.calculate_suggestion_confidence(&suggestions),
        }
    }
}

// 摘要生成器
pub struct SummaryOptions {
    pub max_length: usize,
    pub style: SummaryStyle,  // Concise, Detailed, BulletPoints
    pub include_key_points: bool,
    pub target_audience: AudienceLevel,  // Beginner, Intermediate, Expert
}

#[derive(Debug)]
pub struct Summary {
    pub text: String,
    pub key_points: Vec<String>,
    pub confidence: f32,
    pub reading_time: Duration,
}

// 术语解释
#[derive(Debug)]
pub enum TermType {
    Technical,  // API、框架等
    Concept,    // 算法、设计模式
    Code,       // 代码片段
}

pub struct Explanation {
    pub term: String,
    pub explanation: String,
    pub examples: Vec<CodeExample>,
    pub related_terms: Vec<String>,
    pub difficulty_level: DifficultyLevel,
}

pub struct CodeExample {
    pub language: String,
    pub code: String,
    pub description: String,
}
```

**生成内容类型**

| 内容类型 | 示例输入 | 生成输出 |
|----------|----------|----------|
| **摘要** | 整篇文章 | 200字摘要+5个要点 |
| **术语解释** | "双向绑定" | 定义+原理+代码示例 |
| **代码建议** | Vue.js代码片段 | 优化建议+最佳实践 |
| **数据可视化** | HTML表格 | Chart.js图表代码 |
| **思维导图** | 文档内容 | 结构化大纲 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 摘要生成延迟 | < 200ms | 性能测试 |
| 摘要覆盖率 | > 90% | 信息覆盖率 |
| 术语解释准确率 | > 92% | 专家评审 |
| 代码建议质量 | > 4.0/5 | 开发者评分 |
| 中英文支持 | 100% | 多语言测试 |

---

### P1-T3: DOM智能操作 (2周)

**任务描述**
AI精准定位和操作DOM元素

**技术实现**
```rust
// crates/ai-dom/src/lib.rs
pub struct DomIntelligence {
    element_locator: ElementLocator,
    state_tracker: StateTracker,
    interaction_predictor: InteractionPredictor,
}

pub struct ElementLocator {
    semantic_analyzer: SemanticAnalyzer,
    visual_analyzer: VisualAnalyzer,
    structure_analyzer: StructureAnalyzer,
}

impl ElementLocator {
    async fn find_element(&self, query: &str, context: &PageContext) -> Result<DomNodeId> {
        // 1. 查询解析
        let parsed_query = self.semantic_analyzer.parse(query);

        // 2. 语义匹配
        let semantic_candidates = self.semantic_analyzer.find_by_meaning(&parsed_query, &context);

        // 3. 视觉辅助定位（可选）
        if semantic_candidates.len() == 1 {
            return Ok(semantic_candidates[0]);
        }

        // 4. 视觉特征匹配
        let visual_candidates = self.visual_analyzer.match_features(&parsed_query);

        // 5. 上下文融合
        let best_match = self.fuse_candidates(&semantic_candidates, &visual_candidates, &context);

        match best_match {
            Some(node_id) => Ok(node_id),
            None => Err(DomError::ElementNotFound(query.to_string())),
        }
    }
}

// 智能选择器生成
pub struct SmartSelector {
    text_analyzer: TextAnalyzer,
    attribute_extractor: AttributeExtractor,
}

impl SmartSelector {
    fn generate_selector(&self, target: &DomNode) -> Selector {
        // 1. 文本特征
        if let Some(text) = target.get_text() {
            let text_score = self.calculate_text_uniqueness(&text);
            if text_score > 0.8 {
                return Selector::Text(text);
            }
        }

        // 2. ARIA属性
        if let Some(aria_label) = target.get_attribute("aria-label") {
            return Selector::Attribute("aria-label".to_string(), aria_label);
        }

        // 3. 数据属性
        if let Some(data_id) = target.get_attribute("data-testid") {
            return Selector::Attribute("data-testid".to_string(), data_id);
        }

        // 4. 结构位置
        let parent_info = target.get_parent_info();
        if let Some(position) = self.calculate_relative_position(target) {
            return Selector::Position {
                tag: target.tag_name.clone(),
                position,
            };
        }

        // 5. 组合选择器
        Selector::Combined(vec![
            Selector::Tag(target.tag_name.clone()),
            Selector::Attribute("class".to_string(), target.get_attribute("class").unwrap_or_default()),
        ])
    }
}

// 状态跟踪
pub struct StateTracker {
    snapshots: Vec<DomSnapshot>,
    change_detector: ChangeDetector,
}

impl StateTracker {
    fn take_snapshot(&mut self, dom: &DomTree) -> SnapshotId {
        let snapshot = DomSnapshot {
            id: SnapshotId::new(),
            timestamp: Instant::now(),
            url: dom.current_url.clone(),
            elements: self.extract_element_signatures(dom),
            hash: self.calculate_hash(dom),
        };

        let id = snapshot.id;
        self.snapshots.push(snapshot);

        // 保持最近10个快照
        if self.snapshots.len() > 10 {
            self.snapshots.remove(0);
        }

        id
    }

    fn detect_changes(&self, current: &DomTree) -> Vec<DomChange> {
        let current_hash = self.calculate_hash(current);

        if let Some(last_snapshot) = self.snapshots.last() {
            if last_snapshot.hash != current_hash {
                return self.change_detector.diff(&last_snapshot.elements, &current);
            }
        }

        vec![]
    }
}
```

**定位策略**

| 策略类型 | 优先级 | 适用场景 | 示例 |
|----------|--------|----------|------|
| **语义匹配** | 高 | 有明确文本/标签 | "登录按钮" |
| **属性匹配** | 高 | ARIA/data属性 | `[aria-label="Search"]` |
| **视觉特征** | 中 | 图标、位置 | "搜索框旁边的按钮" |
| **结构位置** | 中 | 导航、表单 | "第二个输入框" |
| **组合选择器** | 低 | 复杂场景 | `form input:nth-child(3)` |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 定位准确率 | > 95% | 1000个测试用例 |
| 定位延迟 | < 50ms | 性能测试 |
| 复杂表单支持 | 100% | 电商/注册表单测试 |
| 动态页面支持 | 100% | SPA页面测试 |
| 误操作率 | < 0.1% | 安全测试 |

---

### P1-T4: ai-gpu加速 (1.5周)

**任务描述**
GPU/CUDA加速推理（可选）

**技术实现**
```rust
// crates/ai-gpu/src/lib.rs
pub struct GPUAccelerator {
    device: GPUDevice,
    memory_pool: GPUMemoryPool,
    compute_ctx: ComputeContext,
}

pub enum GPUDevice {
    NvidiaCuda { device_id: usize },
    AMDROCm { device_id: usize },
    IntelOneAPI { device_id: usize },
    AppleMetal { device_id: usize },
}

impl GPUAccelerator {
    pub fn auto_detect() -> Result<Self> {
        // 1. 检测GPU
        if let Some(nvidia) = Self::detect_nvidia() {
            info!("Detected NVIDIA GPU: {}", nvidia.name);
            return Ok(GPUAccelerator::new(GPUDevice::NvidiaCuda { device_id: nvidia.id }));
        }

        if let Some(amd) = Self::detect_amd() {
            info!("Detected AMD GPU: {}", amd.name);
            return Ok(GPUAccelerator::new(GPUDevice::AMDROCm { device_id: amd.id }));
        }

        if let Some(apple) = Self::detect_apple() {
            info!("Detected Apple Silicon GPU");
            return Ok(GPUAccelerator::new(GPUDevice::AppleMetal { device_id: 0 }));
        }

        // 无GPU，使用CPU fallback
        warn!("No compatible GPU found, using CPU fallback");
        Err(GPUError::NoCompatibleDevice)
    }

    pub async fn accelerate_inference(&self, model: &Model, input: &Tensor) -> Result<Tensor> {
        // 1. 检查GPU内存
        let required_memory = self.calculate_memory_requirement(model, input);
        if self.memory_pool.available() < required_memory {
            info!("GPU memory low, triggering cleanup");
            self.memory_pool.cleanup();
        }

        // 2. 上传数据到GPU
        let gpu_input = self.upload_to_gpu(input).await?;

        // 3. 执行推理
        let gpu_output = self.compute_ctx.run(model, &gpu_input).await?;

        // 4. 下载结果
        let output = self.download_from_gpu(&gpu_output).await?;

        Ok(output)
    }
}

// 显存管理
pub struct GPUMemoryPool {
    device: GPUDevice,
    allocated: HashMap<AllocationId, AllocationInfo>,
    total_memory: usize,
    used_memory: usize,
}

impl GPUMemoryPool {
    fn allocate(&mut self, size: usize) -> Result<AllocationId> {
        if self.used_memory + size > self.total_memory {
            // 尝试清理缓存
            self.cleanup();

            if self.used_memory + size > self.total_memory {
                return Err(GPUError::OutOfMemory);
            }
        }

        let id = AllocationId::new();
        self.allocated.insert(id, AllocationInfo { size, last_used: Instant::now() });
        self.used_memory += size;

        Ok(id)
    }

    fn cleanup(&mut self) {
        // LRU清理策略
        let mut allocations: Vec<_> = self.allocated.iter().collect();
        allocations.sort_by(|a, b| a.1.last_used.cmp(&b.1.last_used));

        for (id, info) in allocations {
            if self.used_memory < self.total_memory * 0.8 {
                break;
            }

            self.free(*id);
        }

        info!("GPU memory cleaned, used: {}/{} MB", self.used_memory / 1024 / 1024, self.total_memory / 1024 / 1024);
    }
}

// CPU fallback
pub struct CPUFallback {
    onnx_runtime: Arc<ONNXRuntime>,
}

impl CPUFallback {
    pub fn new() -> Self {
        CPUFallback {
            onnx_runtime: Arc::new(ONNXRuntime::new()),
        }
    }

    pub async fn infer(&self, model: &Model, input: &Tensor) -> Result<Tensor> {
        // 使用ONNX Runtime CPU执行
        let output = self.onnx_runtime.run(model, input).await?;
        Ok(output)
    }
}
```

**加速效果对比**

| 模型 | CPU延迟 | GPU延迟 | 加速比 | 显存占用 |
|------|---------|---------|--------|----------|
| Qwen-0.5B | 120ms | 45ms | 2.7x | 2GB |
| Llama-1B | 250ms | 80ms | 3.1x | 4GB |
| CodeLlama-1.3B | 320ms | 110ms | 2.9x | 5GB |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| GPU检测 | 100%主流GPU | 硬件测试 |
| 加速比 | > 3x | CPU vs GPU对比 |
| 显存管理 | 自动回收 | 压力测试 |
| Fallback | 无GPU时正常工作 | 测试环境验证 |
| 功耗监控 | 笔记本降频 | 实际使用测试 |

---

### P1-T5: 安全与权限 (1周)

**任务描述**
AI操作权限控制，安全审计

**技术实现**
```rust
// crates/ai-security/src/lib.rs
pub struct SecurityManager {
    permission_db: PermissionDB,
    audit_logger: AuditLogger,
    safety_filter: SafetyFilter,
}

pub struct PermissionDB {
    permissions: HashMap<String, PermissionConfig>,
    trusted_domains: HashSet<String>,
    sensitive_patterns: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub struct PermissionConfig {
    pub allowed_actions: HashSet<ActionType>,
    pub confirmation_required: bool,
    pub confidence_threshold: f32,
    pub user_approval_required: bool,
}

// 权限级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionLevel {
    None,              // 无权限
    ReadOnly,          // 只读
    ReadWrite,         // 读写
    FullControl,       // 完全控制
}

// 敏感操作检测
pub struct SafetyFilter {
    sensitive_keywords: HashSet<String>,
    dangerous_domains: HashSet<String>,
    financial_patterns: Vec<Pattern>,
}

impl SafetyFilter {
    fn classify_sensitivity(&self, action: &ActionStep) -> SensitivityLevel {
        match action {
            ActionStep::FillField { value, .. } => {
                if self.contains_sensitive_data(value) {
                    SensitivityLevel::Critical
                } else {
                    SensitivityLevel::Normal
                }
            }
            ActionStep::SubmitForm { .. } => SensitivityLevel::High,
            ActionStep::Navigate { url } => {
                if self.is_financial_domain(url) {
                    SensitivityLevel::Critical
                } else {
                    SensitivityLevel::Normal
                }
            }
            _ => SensitivityLevel::Low,
        }
    }
}

// 审计日志
pub struct AuditLogger {
    log_file: PathBuf,
    encryption_key: [u8; 32],
}

impl AuditLogger {
    fn log_action(&self, action: &ActionStep, user_id: &str, result: &ActionResult) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            user_id: user_id.to_string(),
            action_type: self.classify_action(action),
            target_url: self.extract_url(action),
            result: if result.success { "success" } else { "failure" },
            confidence: result.confidence,
            user_confirmed: result.user_confirmed,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let encrypted = self.encrypt(&json);

        // 写入日志文件
        write!(self.log_file, "{}\n", encrypted);
    }
}
```

**安全策略**

| 场景 | 权限要求 | 确认机制 | 审计 |
|------|----------|----------|------|
| **普通浏览** | ReadOnly | 无需确认 | 记录操作 |
| **表单填写** | ReadWrite | 高置信度自动执行 | 详细记录 |
| **支付操作** | FullControl | 强制用户确认 | 强制记录+加密 |
| **密码输入** | FullControl | 每次都需要确认 | 加密存储 |
| **未知域名** | None | 默认拒绝 | 记录拒绝 |

**权限配置示例**
```rust
// 用户配置文件
{
  "user_id": "user123",
  "permissions": {
    "taobao.com": {
      "allowed_actions": ["read", "fill_form", "click"],
      "confirmation_required": false,
      "confidence_threshold": 0.8
    },
    "github.com": {
      "allowed_actions": ["read", "fill_form"],
      "confirmation_required": false,
      "confidence_threshold": 0.9
    },
    "bank.com": {
      "allowed_actions": ["read"],
      "confirmation_required": true,
      "confidence_threshold": 0.99
    }
  }
}
```

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 敏感操作拦截 | 100% | 安全测试 |
| 误报率 | < 1% | 正常操作测试 |
| 审计完整性 | 100% | 日志检查 |
| 加密强度 | AES-256 | 代码审计 |
| 权限错误率 | < 0.01% | 集成测试 |

## 📦 新增crate结构

```
crates/
├── ai-action/
│   ├── src/
│   │   ├── lib.rs           # 执行引擎
│   │   ├── planner.rs       # 行动计划
│   │   ├── executor.rs      # 行动执行
│   │   └── safety.rs        # 安全控制
│   └── Cargo.toml
├── ai-generator/
│   ├── src/
│   │   ├── lib.rs           # 内容生成
│   │   ├── summarizer.rs    # 摘要生成
│   │   ├── explainer.rs     # 术语解释
│   │   └── code_assistant.rs # 代码建议
│   └── Cargo.toml
├── ai-dom/
│   ├── src/
│   │   ├── lib.rs           # DOM操作
│   │   ├── locator.rs       # 元素定位
│   │   ├── state_tracker.rs # 状态跟踪
│   │   └── selector.rs      # 智能选择器
│   └── Cargo.toml
├── ai-gpu/
│   ├── src/
│   │   ├── lib.rs           # GPU加速
│   │   ├── cuda.rs          # CUDA实现
│   │   ├── metal.rs         # Metal实现
│   │   └── fallback.rs      # CPU fallback
│   └── Cargo.toml
└── ai-security/
    ├── src/
    │   ├── lib.rs           # 安全控制
    │   ├── permission.rs    # 权限管理
    │   ├── audit.rs         # 审计日志
    │   └── filter.rs        # 安全过滤
    └── Cargo.toml
```

## 🎬 Demo场景

### Demo-4: 自动填表
```
场景：用户访问注册页面
用户说："用我的信息填这个表单"

1. AI识别表单结构
   → 检测到字段：姓名、邮箱、电话、地址

2. 从用户档案匹配数据
   → 用户档案包含：姓名、邮箱、电话
   → 地址需要用户补充

3. 自动填写已知字段
   → 姓名：张三
   → 邮箱：zhangsan@example.com
   → 电话：13800138000

4. 询问未知字段
   → "请输入您的地址"

5. 用户确认后提交
   → 弹窗确认："确认提交注册信息吗？"
```

### Demo-5: 智能对比助手
```
场景：用户在电商页面选择两个手机
用户说："比较这两个商品"

1. AI识别选中元素
   → iPhone 15 Pro (¥8999)
   → 华为Mate 60 (¥6999)

2. 提取详细信息
   → iPhone: A17芯片, 48MP摄像头, 6.1英寸
   → 华为: 麒麟9000S, 50MP摄像头, 6.69英寸

3. 生成对比表
   ┌──────────────┬──────────────┬──────────────┐
   │    指标      │  iPhone 15   │  华为Mate 60 │
   ├──────────────┼──────────────┼──────────────┤
   │ 价格         │ ¥8999        │ ¥6999        │
   │ 芯片         │ A17          │ 麒麟9000S    │
   │ 摄像头       │ 48MP         │ 50MP         │
   │ 屏幕         │ 6.1英寸      │ 6.69英寸     │
   └──────────────┴──────────────┴──────────────┘

4. AI分析建议
   → "iPhone性能更强但价格高2000元"
   → "华为性价比更高，屏幕更大"
   → "建议：如果追求性能选iPhone，预算有限选华为"
```

### Demo-6: 智能研究助手
```
场景：用户在技术文档页选择"双向绑定"
用户操作：双击选中"双向绑定"

1. AI理解上下文
   → 当前页面：Vue.js文档
   → 选中术语：双向绑定
   → 页面位置：指南-核心概念

2. 生成详细解释
   → 定义：数据和视图之间的自动同步
   → 原理：v-model + 数据劫持 + 发布订阅
   → 示例代码：
     ```vue
     <input v-model="message">
     <p>{{ message }}</p>

     <script>
     export default {
       data() {
         return {
           message: 'Hello'
         }
       }
     }
     </script>
     ```

3. 相关概念推荐
   → 数据劫持
   → 响应式原理
   → v-model指令

4. 相关文档链接
   → Vue.js官方文档-数据绑定
   → 深入响应式原理
```

## 🎯 成功指标

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| **任务完成率** | ≥ 85% | 100个自动化任务 |
| **执行准确率** | ≥ 95% | DOM操作测试 |
| **响应延迟** | < 200ms | 端到端延迟 |
| **GPU加速比** | ≥ 3x | CPU vs GPU |
| **安全事件** | 0 | 审计日志 |

---

**Phase 1总结：建立AI执行能力，让浏览器"能行动"！** ✅
