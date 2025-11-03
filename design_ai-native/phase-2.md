# Phase 2: AI学习与记忆 (10周)

## 📋 阶段目标

**核心目标**：AI从交互中学习，优化执行策略，提供个性化服务

- ✅ 学习引擎（偏好学习、模式提取、策略优化）
- ✅ 世界模型完善（设备状态跟踪、环境感知）
- ✅ 个性化适配（用户习惯学习、主动服务）
- ✅ 安全与隐私（本地AI优先、权限管理、审计）
- ✅ 持续学习（3次交互掌握偏好，越用越聪明）

**用户可感知价值**：
- 用户说"有点热"，AI自动学习并记住用户偏好，下次直接调空调
- AI学习用户的作息习惯，主动在合适时间提醒和调整设备
- AI从错误中学习，用户纠正后下次不再犯同样错误
- AI理解用户家庭环境，提供更精准的自动化建议

## 🎯 详细任务列表

### P2-T1: 学习引擎 (3.5周)

**任务描述**
构建AI学习引擎，从用户交互中持续学习和优化

**技术实现**
```rust
// crates/ai-core-brain/src/learning_engine.rs
pub struct LearningEngine {
    // 偏好学习器
    preference_learner: PreferenceLearner,

    // 模式提取器
    pattern_extractor: PatternExtractor,

    // 策略优化器
    strategy_optimizer: StrategyOptimizer,

    // 学习反馈处理器
    feedback_processor: FeedbackProcessor,

    // 学习历史
    learning_history: Arc<RwLock<Vec<LearningRecord>>>,
}

// 偏好学习器
pub struct PreferenceLearner {
    user_profiles: Arc<RwLock<HashMap<String, UserProfile>>>,
    preference_models: HashMap<String, PreferenceModel>,
    interaction_tracker: InteractionTracker,
}

impl PreferenceLearner {
    pub async fn learn_preferences(
        &self,
        user_id: &str,
        interactions: &[UserInteraction],
    ) -> Result<UserPreferences> {
        // 1. 分析交互模式
        let patterns = self.analyze_interaction_patterns(interactions)?;

        // 2. 提取偏好信号
        let signals = self.extract_preference_signals(&patterns)?;

        // 3. 更新用户画像
        let preferences = self.update_user_profile(user_id, &signals)?;

        // 4. 验证学习结果
        self.validate_preferences(&preferences)?;

        Ok(preferences)
    }

    fn analyze_interaction_patterns(
        &self,
        interactions: &[UserInteraction],
    ) -> Result<Vec<InteractionPattern>> {
        // 时间模式分析
        let time_patterns = self.analyze_time_patterns(interactions);

        // 设备偏好分析
        let device_patterns = self.analyze_device_preferences(interactions);

        // 行为模式分析
        let behavior_patterns = self.analyze_behavior_patterns(interactions);

        Ok(vec![
            InteractionPattern::Time(time_patterns),
            InteractionPattern::Device(device_patterns),
            InteractionPattern::Behavior(behavior_patterns),
        ])
    }
}

// 模式提取器
pub struct PatternExtractor {
    sequence_miner: SequenceMiner,
    clustering_engine: ClusteringEngine,
    anomaly_detector: AnomalyDetector,
}

impl PatternExtractor {
    pub async fn extract_patterns(
        &self,
        data: &[LearningData],
    ) -> Result<Vec<LearningPattern>> {
        // 1. 序列模式挖掘
        let sequence_patterns = self.sequence_miner.mine_frequent_sequences(data)?;

        // 2. 聚类分析
        let clusters = self.clustering_engine.cluster(data)?;

        // 3. 异常检测
        let anomalies = self.anomaly_detector.detect_anomalies(data)?;

        // 4. 模式整合
        let patterns = self.integrate_patterns(sequence_patterns, clusters, anomalies)?;

        Ok(patterns)
    }
}

// 策略优化器
pub struct StrategyOptimizer {
    // 策略库
    strategy_library: Arc<RwLock<HashMap<String, Strategy>>>,

    // 性能评估器
    performance_evaluator: PerformanceEvaluator,

    // 优化算法
    optimizer: Box<dyn OptimizationAlgorithm>,
}

impl StrategyOptimizer {
    pub async fn optimize_strategy(
        &self,
        strategy_id: &str,
        feedback: &PerformanceFeedback,
    ) -> Result<StrategyUpdate> {
        // 1. 评估当前策略
        let current_performance = self.performance_evaluator.evaluate(strategy_id)?;

        // 2. 生成优化建议
        let optimizations = self.optimizer.optimize(&current_performance, feedback)?;

        // 3. A/B测试（可选）
        let test_result = if optimizations.requires_testing {
            self.run_ab_test(&optimizations).await?
        } else {
            None
        };

        // 4. 更新策略
        let update = StrategyUpdate {
            strategy_id: strategy_id.to_string(),
            changes: optimizations.changes,
            confidence: optimizations.confidence,
            test_result,
        };

        Ok(update)
    }
}
```

**学习能力**

| 学习类型 | 技术方法 | 应用场景 |
|----------|----------|----------|
| **偏好学习** | 协同过滤+深度学习 | 设备偏好、温度习惯 |
| **模式识别** | 序列挖掘+聚类 | 行为模式、日程规律 |
| **策略优化** | 强化学习+进化算法 | 执行策略优化 |
| **异常检测** | 统计模型+ML | 设备故障预警 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 学习速度 | 3次交互掌握偏好 | 用户测试 |
| 预测准确率 | > 85% | 1000次预测测试 |
| 优化效果 | 策略成功率提升>20% | A/B测试 |
| 异常检测率 | > 90% | 故障注入测试 |
| 内存占用 | < 500MB | 性能测试 |

---

### P2-T2: 世界模型完善 (2.5周)

**任务描述**
完善AI世界模型，实现设备状态实时跟踪和环境感知

**技术实现**
```rust
// crates/ai-core-brain/src/world_model.rs
pub struct EnhancedWorldModel {
    // 设备状态跟踪
    device_tracker: DeviceStateTracker,

    // 环境感知
    environment_sensor: EnvironmentSensor,

    // 状态预测
    state_predictor: StatePredictor,

    // 环境分析
    environment_analyzer: EnvironmentAnalyzer,
}

pub struct DeviceStateTracker {
    // 设备状态映射
    device_states: Arc<RwLock<HashMap<DeviceId, DeviceState>>>,

    // 状态历史
    state_history: Arc<RwLock<LruCache<DeviceId, Vec<StateSnapshot>>>>,

    // 状态同步器
    state_synchronizer: StateSynchronizer,
}

impl DeviceStateTracker {
    pub async fn update_device_state(&self, device_id: &str, new_state: DeviceState) -> Result<()> {
        let mut states = self.device_states.write().await;

        // 1. 获取旧状态
        let old_state = states.get(device_id).cloned();

        // 2. 更新状态
        states.insert(device_id.to_string(), new_state.clone());

        // 3. 记录历史
        self.record_state_history(device_id, &new_state).await?;

        // 4. 检测状态变化
        if let Some(old) = old_state {
            self.detect_state_changes(device_id, &old, &new_state)?;
        }

        // 5. 触发相关事件
        self.trigger_state_events(device_id, &new_state).await?;

        Ok(())
    }

    async fn record_state_history(&self, device_id: &str, state: &DeviceState) -> Result<()> {
        let mut history = self.state_history.write().await;

        if let Some(states) = history.get_mut(&device_id.to_string()) {
            states.push(StateSnapshot {
                timestamp: Utc::now(),
                state: state.clone(),
            });

            // 保持最近1000条记录
            if states.len() > 1000 {
                states.remove(0);
            }
        } else {
            history.insert(
                device_id.to_string(),
                vec![StateSnapshot {
                    timestamp: Utc::now(),
                    state: state.clone(),
                }],
            );
        }

        Ok(())
    }
}

// 环境感知
pub struct EnvironmentSensor {
    // 传感器融合
    sensor_fusion: SensorFusion,

    // 环境模型
    environment_model: EnvironmentModel,

    // 上下文推理
    context_reasoner: ContextReasoner,
}

impl EnvironmentSensor {
    pub async fn感知_environment(&self) -> Result<EnvironmentState> {
        // 1. 聚合传感器数据
        let sensor_data = self.sensor_fusion.aggregate().await?;

        // 2. 构建环境模型
        let environment = self.environment_model.build_model(&sensor_data)?;

        // 3. 推理上下文
        let context = self.context_reasoner.infer(&environment)?;

        Ok(EnvironmentState {
            physical: environment,
            contextual: context,
            timestamp: Utc::now(),
        })
    }
}

// 状态预测
pub struct StatePredictor {
    // 预测模型
    prediction_models: HashMap<DeviceType, Box<dyn PredictionModel>>,

    // 预测历史
    prediction_history: Arc<RwLock<Vec<PredictionRecord>>>,
}

impl StatePredictor {
    pub async fn predict_device_state(
        &self,
        device_id: &str,
        time_horizon: Duration,
    ) -> Result<StatePrediction> {
        // 1. 选择预测模型
        let device_type = self.get_device_type(device_id)?;
        let model = self.prediction_models
            .get(&device_type)
            .ok_or(LearningError::NoPredictionModel)?;

        // 2. 获取历史数据
        let history = self.get_device_history(device_id, Duration::days(7)).await?;

        // 3. 执行预测
        let prediction = model.predict(&history, time_horizon)?;

        // 4. 计算置信度
        let confidence = self.calculate_prediction_confidence(&history, &prediction)?;

        // 5. 记录预测
        self.record_prediction(device_id, &prediction, confidence).await?;

        Ok(StatePrediction {
            device_id: device_id.to_string(),
            predicted_state: prediction,
            confidence,
            time_horizon,
        })
    }
}
```

**世界模型能力**

| 能力 | 说明 | 技术实现 |
|------|------|----------|
| **设备跟踪** | 实时监控所有设备状态 | 事件驱动+状态机 |
| **环境感知** | 融合多源传感器数据 | 卡尔曼滤波+贝叶斯 |
| **状态预测** | 基于历史预测未来状态 | LSTM+马尔可夫模型 |
| **上下文推理** | 理解环境语义 | 知识图谱+规则引擎 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 状态同步延迟 | < 100ms | 实时监控测试 |
| 预测准确率 | > 85% | 7天预测验证 |
| 设备跟踪率 | 100%已注册设备 | 设备库测试 |
| 环境感知精度 | > 90% | 传感器对照测试 |
| 预测延迟 | < 200ms | 性能基准测试 |

---

### P2-T3: 个性化适配 (2.5周)

**任务描述**
AI学习用户习惯，提供个性化服务和主动建议

**技术实现**
```rust
// crates/ai-core-brain/src/personalization.rs
pub struct PersonalizationEngine {
    // 用户画像
    user_profiler: UserProfiler,

    // 习惯学习器
    habit_learner: HabitLearner,

    // 主动服务引擎
    proactive_service: ProactiveServiceEngine,

    // 个性化推荐器
    recommendation_engine: PersonalizedRecommender,
}

// 用户画像
pub struct UserProfiler {
    // 静态特征
    static_features: Arc<RwLock<UserStaticProfile>>,

    // 动态特征
    dynamic_features: Arc<RwLock<UserDynamicProfile>>,

    // 特征提取器
    feature_extractor: FeatureExtractor,
}

impl UserProfiler {
    pub async fn update_profile(
        &self,
        user_id: &str,
        interactions: &[UserInteraction],
    ) -> Result<UserProfile> {
        // 1. 提取交互特征
        let features = self.feature_extractor.extract(interactions)?;

        // 2. 更新静态特征（很少变化）
        let static_profile = self.update_static_features(&features)?;

        // 3. 更新动态特征（经常变化）
        let dynamic_profile = self.update_dynamic_features(&features)?;

        // 4. 生成完整画像
        let profile = UserProfile {
            user_id: user_id.to_string(),
            static_features: static_profile,
            dynamic_features: dynamic_profile,
            last_updated: Utc::now(),
        };

        // 5. 保存画像
        self.save_profile(&profile)?;

        Ok(profile)
    }
}

// 习惯学习器
pub struct HabitLearner {
    // 习惯检测器
    habit_detector: HabitDetector,

    // 习惯跟踪器
    habit_tracker: HabitTracker,

    // 习惯优化器
    habit_optimizer: HabitOptimizer,
}

impl HabitLearner {
    pub async fn learn_habits(&self, user_id: &str) -> Result<Vec<Habit>> {
        // 1. 收集用户行为数据
        let behavior_data = self.collect_behavior_data(user_id).await?;

        // 2. 检测重复模式
        let patterns = self.habit_detector.detect_patterns(&behavior_data)?;

        // 3. 验证习惯有效性
        let valid_habits = self.validate_habits(&patterns)?;

        // 4. 跟踪习惯强度
        let tracked_habits = self.habit_tracker.track(&valid_habits)?;

        // 5. 优化习惯建议
        let optimized_habits = self.habit_optimizer.optimize(&tracked_habits)?;

        Ok(optimized_habits)
    }
}

// 主动服务引擎
pub struct ProactiveServiceEngine {
    // 机会检测器
    opportunity_detector: OpportunityDetector,

    // 服务推荐器
    service_recommender: ServiceRecommender,

    // 通知管理器
    notification_manager: NotificationManager,
}

impl ProactiveServiceEngine {
    pub async fn generate_proactive_suggestions(
        &self,
        user_id: &str,
        current_context: &Context,
    ) -> Result<Vec<ProactiveSuggestion>> {
        // 1. 检测服务机会
        let opportunities = self.opportunity_detector.detect(
            user_id,
            current_context,
        )?;

        // 2. 生成服务建议
        let suggestions = self.service_recommender.recommend(
            &opportunities,
            current_context,
        )?;

        // 3. 排序和过滤
        let ranked_suggestions = self.rank_and_filter_suggestions(&suggestions)?;

        // 4. 生成通知
        let notifications = self.notification_manager.prepare_notifications(
            &ranked_suggestions,
        )?;

        Ok(ranked_suggestions
            .into_iter()
            .map(|s| ProactiveSuggestion {
                suggestion: s,
                notification: notifications
                    .get(&s.id)
                    .cloned()
                    .unwrap_or_default(),
            })
            .collect())
    }
}
```

**个性化能力**

| 能力类型 | 学习方法 | 应用场景 |
|----------|----------|----------|
| **习惯学习** | 时间序列分析 | 作息规律、使用习惯 |
| **偏好适配** | 协同过滤 | 设备设置、内容推荐 |
| **主动服务** | 机会挖掘 | 智能提醒、自动化建议 |
| **上下文适应** | 上下文推理 | 环境感知、场景识别 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 习惯识别准确率 | > 80% | 30天行为分析 |
| 个性化推荐CTR | > 25% | A/B测试 |
| 主动服务采纳率 | > 60% | 用户反馈测试 |
| 响应时间 | < 300ms | 性能测试 |
| 用户满意度 | > 4.0/5 | 用户调研 |

---

### P2-T4: 安全与隐私 (1.5周)

**任务描述**
强化本地AI优先、权限管理、审计日志等安全机制

**技术实现**
```rust
// crates/security-privacy/src/lib.rs
pub struct SecurityPrivacyManager {
    // 本地AI优先
    local_ai_orchestrator: LocalAIOrchestrator,

    // 权限管理
    permission_manager: PermissionManager,

    // 审计日志
    audit_logger: AuditLogger,

    // 加密管理
    encryption_manager: EncryptionManager,
}

// 本地AI优先
pub struct LocalAIOrchestrator {
    // 本地模型库
    local_models: Arc<RwLock<HashMap<String, LocalModel>>>,

    // 模型选择器
    model_selector: ModelSelector,

    // 推理引擎
    inference_engine: LocalInferenceEngine,
}

impl LocalAIOrchestrator {
    pub async fn prefer_local_inference(
        &self,
        task: &InferenceTask,
    ) -> Result<InferenceResult> {
        // 1. 检查本地模型可用性
        let available_models = self.get_available_local_models(task)?;

        if !available_models.is_empty() {
            // 2. 选择最佳本地模型
            let best_model = self.model_selector.select_best(&available_models, task)?;

            // 3. 本地推理
            let result = self.inference_engine.run(best_model, task).await?;

            return Ok(result);
        }

        // 4. 如果没有本地模型，提示用户
        Err(SecurityError::LocalModelUnavailable)
    }
}

// 权限管理
pub struct PermissionManager {
    // 权限配置
    permission_configs: Arc<RwLock<HashMap<String, PermissionConfig>>>,

    // 用户授权记录
    user_grants: Arc<RwLock<HashMap<String, Vec<GrantRecord>>>>,

    // 风险评估
    risk_assessor: RiskAssessor,
}

impl PermissionManager {
    pub async fn check_permission(
        &self,
        user_id: &str,
        tool_id: &str,
        action: &str,
    ) -> Result<PermissionCheck> {
        let config = self.permission_configs
            .read()
            .await
            .get(tool_id)
            .ok_or(SecurityError::NoPermissionConfig)?;

        // 1. 风险评估
        let risk = self.risk_assessor.assess(tool_id, action)?;

        // 2. 检查自动授权阈值
        if risk.level <= config.auto_grant_threshold {
            return Ok(PermissionCheck::Granted);
        }

        // 3. 检查历史授权
        let has_grant = self.check_historical_grant(user_id, tool_id, action)?;

        if has_grant {
            return Ok(PermissionCheck::Granted);
        }

        // 4. 需要用户确认
        Ok(PermissionCheck::NeedsConfirmation {
            message: self.generate_confirmation_message(tool_id, action, &risk)?,
        })
    }
}

// 审计日志
pub struct AuditLogger {
    // 日志存储
    log_storage: Arc<dyn LogStorage>,

    // 实时监控
    realtime_monitor: RealtimeMonitor,

    // 异常检测
    anomaly_detector: SecurityAnomalyDetector,
}

impl AuditLogger {
    pub async fn log_event(&self, event: &SecurityEvent) -> Result<()> {
        // 1. 记录日志
        self.log_storage.store(event).await?;

        // 2. 实时监控
        self.realtime_monitor.process(event).await?;

        // 3. 异常检测
        if let Some(anomaly) = self.anomaly_detector.detect(event)? {
            self.handle_security_anomaly(&anomaly).await?;
        }

        Ok(())
    }
}
```

**安全与隐私能力**

| 能力 | 说明 | 技术实现 |
|------|------|----------|
| **本地AI优先** | 敏感数据不出本机 | 本地模型+智能降级 |
| **权限管理** | 最小权限原则 | RBAC+风险评估 |
| **审计日志** | 所有操作可追溯 | 不可篡改日志 |
| **端到端加密** | 数据传输加密 | AES-256+TLS 1.3 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 本地AI使用率 | > 80%敏感任务 | 任务分类统计 |
| 权限检查正确率 | 100% | 安全测试 |
| 审计覆盖率 | 100%关键操作 | 日志检查 |
| 加密强度 | AES-256 | 安全审计 |
| 隐私合规性 | 100%GDPR | 合规检查 |

    // 操作习惯
    behavior_patterns: Vec<BehaviorPattern>,

    // 存储引擎
    storage: Arc<RocksDB>,
}

impl LongTermMemory {
    fn update_from_interaction(&mut self, interaction: &Interaction) {
        // 1. 更新用户偏好
        self.update_preferences(interaction);

        // 2. 记录浏览历史
        self.browsing_history.push(PageVisit {
            url: interaction.url.clone(),
            timestamp: interaction.timestamp,
            duration: interaction.duration,
            actions: interaction.actions.clone(),
        });

        // 3. 提取行为模式
        let pattern = self.extract_behavior_pattern(interaction);
        self.behavior_patterns.push(pattern);

        // 4. 持久化存储
        self.persist();
    }

    fn get_user_preference(&self, key: &str) -> Option<&PreferenceValue> {
        self.preferences.get(key)
    }
}

pub struct SemanticMemory {
    // 概念网络
    concept_graph: ConceptGraph,

    // 实体关系
    entity_relations: HashMap<EntityId, Vec<Relation>>,

    // 语义向量索引
    vector_index: FaissIndex,

    // 知识库
    knowledge_base: KnowledgeBase,
}

impl SemanticMemory {
    fn build_concept_links(&mut self, interaction: &Interaction) {
        // 1. 提取实体
        let entities = self.extract_entities(&interaction.content);

        // 2. 建立实体关系
        for entity in &entities {
            // 查找已有实体
            if let Some(existing) = self.find_similar_entity(entity) {
                // 建立关联
                self.entity_relations
                    .entry(entity.id)
                    .or_insert_with(Vec::new)
                    .push(Relation {
                        target: existing.id,
                        relation_type: self.infer_relation_type(entity, existing),
                        confidence: self.calculate_similarity(entity, existing),
                        last_updated: Utc::now(),
                    });
            } else {
                // 新实体，添加到概念图
                self.concept_graph.add_entity(entity);
            }
        }

        // 3. 更新向量索引
        self.update_vector_index(&entities);
    }

    fn semantic_search(&self, query: &str) -> Vec<SemanticMatch> {
        // 1. 生成查询向量
        let query_vector = self.encode_text(query);

        // 2. 相似度搜索
        let (indices, distances) = self.vector_index.search(&query_vector, k=10);

        // 3. 构建结果
        indices.into_iter().zip(distances).map(|(idx, dist)| {
            let entity = self.concept_graph.get_entity(idx);
            SemanticMatch {
                entity,
                similarity: 1.0 - dist, // 转换为相似度
            }
        }).collect()
    }
}

pub struct ProceduralMemory {
    // 自动化模式
    automation_patterns: HashMap<PatternId, AutomationPattern>,

    // 操作序列
    action_sequences: Vec<ActionSequence>,

    // 快捷操作
    shortcuts: HashMap<String, ShortcutAction>,
}

impl ProceduralMemory {
    fn extract_pattern(&mut self, behavior: &UserBehavior) -> Option<AutomationPattern> {
        // 1. 行为序列分析
        let sequence = self.analyze_action_sequence(&behavior.actions);

        // 2. 检查是否重复
        if sequence.repeat_count >= 3 {
            // 3. 提取模式
            let pattern = AutomationPattern {
                id: PatternId::new(),
                trigger: sequence.trigger.clone(),
                actions: sequence.actions.clone(),
                confidence: self.calculate_pattern_confidence(&sequence),
                last_seen: Utc::now(),
            };

            // 4. 存储模式
            self.automation_patterns.insert(pattern.id, pattern.clone());

            // 5. 建议自动化
            if pattern.confidence > 0.8 {
                self.suggest_automation(pattern);
            }

            Some(pattern)
        } else {
            None
        }
    }
}
```

**记忆层次结构**

| 记忆类型 | 容量 | 持久时间 | 作用 | 示例 |
|----------|------|----------|------|------|
| **工作记忆** | 10个交互 | 会话内 | 当前任务 | 当前浏览的页面 |
| **语义记忆** | 无限制 | 永久 | 概念关联 | "iPhone"和"苹果"的关系 |
| **程序记忆** | 50个模式 | 永久 | 操作自动化 | "购物流程"模式 |
| **情节记忆** | 1000条 | 永久 | 事件回忆 | "上次的购物经历" |

**验收标准**
| 标准 | 目标值 | 测试方法 |
|------|--------|----------|
| 记忆容量 | 10万+条目 | 压力测试 |
| 检索延迟 | < 20ms | 性能测试 |
| 存储可靠性 | 100% | 崩溃恢复测试 |
| 模式识别准确率 | > 85% | 行为分析测试 |
| 内存使用 | < 100MB | 内存监控 |

---

### P2-T2: ai-knowledge-graph知识图谱构建 (2.5周)

**任务描述**
从浏览历史构建个人知识图谱

**技术实现**
```rust
// crates/ai-knowledge-graph/src/lib.rs
pub struct PersonalKnowledgeGraph {
    // 实体存储
    entities: HashMap<EntityId, UserEntity>,

    // 关系存储
    relationships: Vec<Relation>,

    // 图结构
    graph: petgraph::Graph<EntityId, RelationType, petgraph::Undirected, u32>,

    // 索引加速
    text_index: HashMap<String, Vec<EntityId>>,
    type_index: HashMap<EntityType, Vec<EntityId>>,
    time_index: BTreeMap<Timestamp, Vec<EntityId>>,
}

impl PersonalKnowledgeGraph {
    pub fn build_from_browsing(&mut self, history: Vec<PageVisit>) {
        // 1. 遍历浏览历史
        for visit in history {
            // 2. 页面类型识别
            let page_type = self.classify_page(&visit.url, &visit.content);

            // 3. 实体提取
            let entities = self.extract_entities(&visit, page_type);

            // 4. 关系推理
            let relations = self.infer_relations(&entities, &visit);

            // 5. 添加到图
            self.add_entities(&entities);
            self.add_relations(&relations);

            // 6. 更新索引
            self.update_indices(&entities);
        }

        // 7. 图优化
        self.optimize_graph();
    }

    fn extract_entities(&self, visit: &PageVisit, page_type: PageType) -> Vec<UserEntity> {
        match page_type {
            PageType::Ecommerce => self.extract_product_entities(visit),
            PageType::News => self.extract_news_entities(visit),
            PageType::TechDoc => self.extract_tech_entities(visit),
            PageType::Blog => self.extract_blog_entities(visit),
            PageType::Social => self.extract_social_entities(visit),
        }
    }

    fn extract_product_entities(&self, visit: &PageVisit) -> Vec<UserEntity> {
        let mut entities = Vec::new();

        // 产品名称
        if let Some(product_name) = self.extract_product_name(&visit.content) {
            entities.push(UserEntity {
                id: EntityId::new(),
                entity_type: EntityType::Product,
                name: product_name,
                properties: self.extract_product_properties(&visit.content),
                first_seen: visit.timestamp,
                last_seen: visit.timestamp,
                visit_count: 1,
            });
        }

        // 品牌
        if let Some(brand) = self.extract_brand(&visit.content) {
            entities.push(UserEntity {
                id: EntityId::new(),
                entity_type: EntityType::Brand,
                name: brand,
                properties: HashMap::new(),
                first_seen: visit.timestamp,
                last_seen: visit.timestamp,
                visit_count: 1,
            });
        }

        // 价格
        if let Some(price) = self.extract_price(&visit.content) {
            let entity = UserEntity {
                id: EntityId::new(),
                entity_type: EntityType::Price,
                name: price.to_string(),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("value".to_string(), json!(price));
                    props.insert("currency".to_string(), json!("CNY"));
                    props
                },
                first_seen: visit.timestamp,
                last_seen: visit.timestamp,
                visit_count: 1,
            };
            entities.push(entity);
        }

        entities
    }

    fn infer_relations(&self, entities: &[UserEntity], visit: &PageVisit) -> Vec<Relation> {
        let mut relations = Vec::new();

        // 产品-品牌关系
        if let (Some(product), Some(brand)) = (
            entities.iter().find(|e| e.entity_type == EntityType::Product),
            entities.iter().find(|e| e.entity_type == EntityType::Brand),
        ) {
            relations.push(Relation {
                source: product.id,
                target: brand.id,
                relation_type: RelationType::BelongsTo,
                confidence: 0.95,
                evidence: vec![visit.url.clone()],
                created_at: visit.timestamp,
            });
        }

        // 价格-产品关系
        if let (Some(product), Some(price)) = (
            entities.iter().find(|e| e.entity_type == EntityType::Product),
            entities.iter().find(|e| e.entity_type == EntityType::Price),
        ) {
            relations.push(Relation {
                source: product.id,
                target: price.id,
                relation_type: RelationType::PricedAt,
                confidence: 0.90,
                evidence: vec![visit.url.clone()],
                created_at: visit.timestamp,
            });
        }

        // 用户兴趣关系
        for entity in entities {
            if entity.visit_count > 5 {
                relations.push(Relation {
                    source: USER_ROOT_ID,
                    target: entity.id,
                    relation_type: RelationType::InterestedIn,
                    confidence: self.calculate_interest_confidence(entity),
                    evidence: vec![],
                    created_at: entity.last_seen,
                });
            }
        }

        relations
    }

    fn query(&self, query: &str) -> Vec<EntityMatch> {
        // 1. 解析查询
        let parsed_query = self.parse_query(query);

        // 2. 实体匹配
        let mut candidate_entities = Vec::new();

        // 文本匹配
        if let Some(entity_ids) = self.text_index.get(query) {
            for id in entity_ids {
                if let Some(entity) = self.entities.get(id) {
                    candidate_entities.push(entity);
                }
            }
        }

        // 3. 关系遍历
        if parsed_query.has_relation() {
            let related = self.traverse_relations(&parsed_query);
            candidate_entities.extend(related);
        }

        // 4. 排序和去重
        let mut unique_entities: HashSet<_> = candidate_entities.into_iter().collect();
        let mut results: Vec<_> = unique_entities.into_iter().collect();

        results.sort_by(|a, b| b.visit_count.cmp(&a.visit_count));

        results.into_iter().take(10).map(|entity| {
            EntityMatch {
                entity,
                relevance_score: self.calculate_relevance(entity, query),
                matched_properties: self.find_matched_properties(entity, query),
            }
        }).collect()
    }
}

// 查询解析
struct Query {
    entity_types: Vec<EntityType>,
    relations: Vec<RelationQuery>,
    constraints: Vec<Constraint>,
}

impl Query {
    fn parse(input: &str) -> Self {
        // 简单的查询解析器
        // 例如："我之前看过的相机" -> EntityType: Product, Constraint: category=camera
        let mut query = Query {
            entity_types: Vec::new(),
            relations: Vec::new(),
            constraints: Vec::new(),
        };

        if input.contains("相机") || input.contains("摄影") {
            query.entity_types.push(EntityType::Product);
            query.constraints.push(Constraint {
                key: "category".to_string(),
                operator: "==".to_string(),
                value: json!("camera"),
            });
        }

        query
    }
}
```

**知识图谱规模**

| 实体类型 | 预期数量 | 示例 |
|----------|----------|------|
| **产品** | 1-5万 | iPhone 15, MacBook Pro |
| **品牌** | 500-2000 | 苹果, 华为, 索尼 |
| **人物** | 1000-5000 | 作者, 朋友, 同事 |
| **地点** | 500-2000 | 北京, 公司, 家 |
| **概念** | 5000-2万 | 技术, 兴趣, 话题 |
| **关系** | 10-50万 | 购买过, 感兴趣, 关注 |

**验收标准**
| 标准 | 目标值 | 测试方法 |
|------|--------|----------|
| 实体数量 | ≥ 10万 | 图统计 |
| 关系数量 | ≥ 50万 | 图统计 |
| 实体提取F1 | > 0.88 | 标准评测集 |
| 关系推理准确率 | > 85% | 人工标注对比 |
| 查询延迟 | < 100ms | 性能测试 |

---

### P2-T3: ai-context-resolver上下文理解 (2周)

**任务描述**
跨页面上下文理解，"上次的相机"类回指消解

**技术实现**
```rust
// crates/ai-context-resolver/src/lib.rs
pub struct ContextResolver {
    // 回指解析器
    anaphora_resolver: AnaphoraResolver,

    // 上下文管理器
    context_manager: ContextManager,

    // 指代词词典
    pronoun_dict: PronounDict,
}

impl ContextResolver {
    pub fn resolve_reference(&self, text: &str, current_page: &PageContext) -> ResolutionResult {
        // 1. 检测回指示象
        let references = self.anaphora_resolver.detect(text);

        // 2. 搜索候选实体
        let candidates = self.search_candidate_entities(&references, current_page);

        // 3. 上下文融合
        let resolved = self.resolve_with_context(&candidates, current_page);

        ResolutionResult {
            original_text: text.to_string(),
            resolved_entities: resolved.entities,
            confidence: resolved.confidence,
            resolution_method: resolved.method,
        }
    }

    fn resolve_with_context(&self, candidates: &[EntityCandidate], page: &PageContext) -> ResolvedEntity {
        // 评分维度
        let mut scores = Vec::new();

        for candidate in candidates {
            let mut score = 0.0;

            // 1. 语义相似度 (40%)
            let semantic_score = self.calculate_similarity(&candidate.entity, &page.current_focus);
            score += semantic_score * 0.4;

            // 2. 时间距离 (20%)
            let time_score = self.calculate_time_score(candidate.entity.last_seen, page.timestamp);
            score += time_score * 0.2;

            // 3. 频率权重 (20%)
            let freq_score = (candidate.entity.visit_count as f32 / 100.0).min(1.0);
            score += freq_score * 0.2;

            // 4. 页面类型匹配 (10%)
            let type_score = if Self::is_type_match(&candidate.entity, &page.page_type) { 1.0 } else { 0.0 };
            score += type_score * 0.1;

            // 5. 话题连续性 (10%)
            let topic_score = self.calculate_topic_continuity(candidate.entity.topic, page.topic);
            score += topic_score * 0.1;

            scores.push((candidate.entity.clone(), score));
        }

        // 选择最高分
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let (best_entity, best_score) = &scores[0];

        ResolvedEntity {
            entity: best_entity.clone(),
            confidence: *best_score,
            method: self.determine_resolution_method(best_entity),
        }
    }
}

pub struct AnaphoraResolver {
    // 指代词类型识别
    pronoun_types: HashMap<String, PronounType>,
    // 语境模式
    context_patterns: Vec<ContextPattern>,
}

impl AnaphoraResolver {
    fn detect(&self, text: &str) -> Vec<Reference> {
        let mut references = Vec::new();

        // 1. 检测直接指代
        // "上次的", "这个", "那个"
        if text.contains("上次的") {
            let entity_type = self.extract_type_from_context(text);
            references.push(Reference {
                pronoun: "上次的".to_string(),
                position: text.find("上次的").unwrap(),
                entity_type,
                antecedent_type: AntecedentType::LastMentioned,
            });
        }

        // 2. 检测类指代
        // "它", "这个产品", "那个公司"
        if text.contains("它") || text.contains("这个") || text.contains("那个") {
            let antecedent = self.find_antecedent(text);
            if let Some(ant) = antecedent {
                references.push(Reference {
                    pronoun: "它".to_string(),
                    position: text.find("它").unwrap(),
                    entity_type: ant.entity_type,
                    antecedent_type: AntecedentType::Explicit(ant.id),
                });
            }
        }

        // 3. 检测省略指代
        // "我之前看过的那个" (省略具体名词)
        if text.contains("我之前看过的") {
            let implicit_type = self.infer_type_from_verb(text);
            references.push(Reference {
                pronoun: "我之前看过的".to_string(),
                position: text.find("我之前看过的").unwrap(),
                entity_type: implicit_type,
                antecedent_type: AntecedentType::Implicit,
            });
        }

        references
    }
}

// 指代类型
#[derive(Debug)]
enum AntecedentType {
    Explicit(EntityId),        // 明确指代：那个iPhone
    Implicit,                   // 隐式指代：我之前看过的
    LastMentioned,             // 最近提及：那个产品
    Generic,                   // 泛指：它（之前讨论的所有）
}
```

**回指类型详解**

| 类型 | 示例 | 解析方法 | 候选搜索范围 |
|------|------|----------|--------------|
| **时间回指** | "上次的相机" | 按时间倒序搜索最近实体 | 最近10次相关页面 |
| **语义回指** | "那个便宜的手机" | 按属性匹配（便宜+手机） | 所有实体+属性筛选 |
| **话题回指** | "我刚才看的" | 按话题连续性 | 同话题历史 |
| **省略回指** | "那个" | 根据上下文推断 | 最近N句对话/页面 |
| **泛指** | "它很好用" | 指代整个主题 | 当前话题所有实体 |

**验收标准**
| 标准 | 目标值 | 测试方法 |
|------|--------|----------|
| 回指消解准确率 | > 90% | 1000个测试用例 |
| 支持回指类型 | 5种 | 功能测试 |
| 上下文窗口 | 10个页面 | 集成测试 |
| 解析延迟 | < 50ms | 性能测试 |
| 置信度评估 | AUC > 0.85 | ROC曲线 |

---

### P2-T4: ai-recommender个性化推荐 (1.5周)

**任务描述**
基于历史行为的智能推荐

**技术实现**
```rust
// crates/ai-recommender/src/lib.rs
pub struct PersonalizedRecommender {
    // 用户兴趣模型
    user_interest_model: UserInterestModel,

    // 协同过滤
    collaborative_filter: CollaborativeFilter,

    // 内容推荐
    content_recommender: ContentRecommender,

    // 实时特征提取
    feature_extractor: FeatureExtractor,
}

impl PersonalizedRecommender {
    pub fn recommend(&self, context: &RecommendationContext) -> Vec<Recommendation> {
        // 1. 特征提取
        let features = self.feature_extractor.extract(context);

        // 2. 多策略推荐
        let mut recommendations = Vec::new();

        // 策略1: 基于兴趣
        if let Some(interest_recs) = self.user_interest_model.recommend(&features) {
            recommendations.extend(interest_recs);
        }

        // 策略2: 协同过滤
        if let Some(collab_recs) = self.collaborative_filter.recommend(context.user_id, &features) {
            recommendations.extend(collab_recs);
        }

        // 策略3: 内容相似
        if let Some(content_recs) = self.content_recommender.recommend(&context.current_page, &features) {
            recommendations.extend(content_recs);
        }

        // 策略4: 热门内容
        if let Some(trending_recs) = self.trending_recommender.get_trending(&features.user_profile) {
            recommendations.extend(trending_recs);
        }

        // 5. 混合排序
        let reranked = self.rerank_recommendations(recommendations, context);

        // 6. 多样性优化
        let diversified = self.ensure_diversity(reranked);

        // 7. 限制数量
        diversified.into_iter().take(10).collect()
    }

    fn rerank_recommendations(&self, recs: Vec<Recommendation>, context: &RecommendationContext) -> Vec<Recommendation> {
        // 使用机器学习模型重排序
        let mut scored_recs: Vec<_> = recs.into_iter().map(|rec| {
            let score = self.calculate_combined_score(&rec, context);
            (rec, score)
        }).collect();

        // 按分数排序
        scored_recs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        scored_recs.into_iter().map(|(rec, _)| rec).collect()
    }
}

pub struct UserInterestModel {
    // 兴趣向量
    interest_vector: HashMap<InterestCategory, f32>,

    // 行为序列
    behavior_sequence: Vec<UserAction>,

    // 衰减因子
    decay_factor: f32,
}

impl UserInterestModel {
    fn update_from_behavior(&mut self, action: &UserAction) {
        // 1. 更新兴趣向量
        let category = self.categorize_action(action);
        let weight = self.calculate_action_weight(action);

        // 2. 时间衰减
        let decay = self.calculate_decay(action.timestamp);

        // 3. 更新向量
        let current_score = self.interest_vector.get(&category).unwrap_or(&0.0);
        let new_score = current_score * decay + weight;
        self.interest_vector.insert(category, new_score);

        // 4. 添加到序列
        self.behavior_sequence.push(action.clone());

        // 5. 保持最近1000条
        if self.behavior_sequence.len() > 1000 {
            self.behavior_sequence.remove(0);
        }
    }

    fn recommend(&self, features: &FeatureVector) -> Option<Vec<Recommendation>> {
        // 基于兴趣相似度推荐
        let top_interests: Vec<_> = self.interest_vector
            .iter()
            .filter(|(_, &score)| score > 0.1)
            .collect();

        if top_interests.is_empty() {
            return None;
        }

        let mut recommendations = Vec::new();

        for (category, score) in top_interests {
            let recs = self.query_by_interest(category, *score);
            recommendations.extend(recs);
        }

        Some(recommendations)
    }
}

// 推荐类型
pub enum RecommendationType {
    News,          // 新闻
    Product,       // 商品
    Article,       // 文章
    Website,       // 网站
    Video,         // 视频
    Topic,         // 话题
}

pub struct Recommendation {
    pub id: String,
    pub item: RecommendationItem,
    pub score: f32,
    pub reason: String,  // 推荐原因
    pub recommendation_type: RecommendationType,
}

impl Recommendation {
    pub fn explain(&self) -> String {
        // 生成可解释的推荐原因
        format!(
            "推荐 {} 因为：{}",
            self.item.title,
            self.reason
        )
    }
}
```

**推荐策略**

| 策略 | 数据来源 | 适用场景 | 优点 | 缺点 |
|------|----------|----------|------|------|
| **兴趣向量** | 用户行为历史 | 长期偏好 | 准确反映兴趣 | 冷启动问题 |
| **协同过滤** | 相似用户行为 | 新内容发现 | 发现惊喜 | 需要用户量 |
| **内容推荐** | 当前页面内容 | 上下文相关 | 高相关性 | 缺乏多样性 |
| **热门推荐** | 全局趋势 | 新用户 | 无需历史 | 缺乏个性 |

**验收标准**
| 标准 | 目标值 | 测试方法 |
|------|--------|----------|
| 推荐准确率 | > 75% | A/B测试 |
| 点击率(CTR) | > 15% | 在线测试 |
| 多样性 | > 0.6 | 熵值计算 |
| 冷启动 | 3次操作后生效 | 新用户测试 |
| 解释性 | 100%有理由 | 审计检查 |

---

### P2-T5: ai-privacy隐私保护 (1周)

**任务描述**
本地存储、加密、可导出的隐私保护机制

**技术实现**
```rust
// crates/ai-privacy/src/lib.rs
pub struct PrivacyManager {
    // 加密引擎
    encryption_engine: EncryptionEngine,

    // 本地存储
    local_storage: LocalStorage,

    // 隐私策略
    privacy_policy: PrivacyPolicy,

    // 数据治理
    data_governance: DataGovernance,
}

impl PrivacyManager {
    pub fn store_sensitive_data(&self, key: &str, data: &SensitiveData) -> Result<()> {
        // 1. 加密数据
        let encrypted = self.encryption_engine.encrypt(data)?;

        // 2. 添加元数据
        let stored_data = StoredData {
            encrypted_payload: encrypted,
            metadata: DataMetadata {
                key: key.to_string(),
                created_at: Utc::now(),
                sensitivity_level: data.sensitivity_level,
                retention_policy: data.retention_policy,
                access_count: 0,
            },
        };

        // 3. 存储到本地
        self.local_storage.store(&key, &stored_data)?;

        // 4. 更新审计日志
        self.log_access(key, AccessType::Store)?;

        Ok(())
    }

    pub fn retrieve_data(&self, key: &str, auth: &AuthToken) -> Result<SensitiveData> {
        // 1. 验证权限
        self.validate_access(key, auth, Permission::Read)?;

        // 2. 获取加密数据
        let stored_data = self.local_storage.retrieve(key)?;

        // 3. 解密
        let decrypted = self.encryption_engine.decrypt(&stored_data.encrypted_payload)?;

        // 4. 更新访问统计
        self.update_access_count(key)?;

        // 5. 记录审计
        self.log_access(key, AccessType::Retrieve)?;

        Ok(decrypted)
    }
}

// 加密引擎
pub struct EncryptionEngine {
    cipher: Aes256Gcm,
    key_manager: KeyManager,
}

impl EncryptionEngine {
    pub fn encrypt(&self, data: &SensitiveData) -> Result<EncryptedPayload> {
        // 1. 生成随机IV
        let iv = random::<[u8; 12]>();

        // 2. 序列化数据
        let serialized = bincode::serialize(data)?;

        // 3. 加密
        let cipher = Aes256Gcm::new(Key::from_slice(&self.key_manager.get_data_key()));
        let encrypted = cipher.encrypt(Nonce::from_slice(&iv), serialized.as_ref())
            .map_err(|_| PrivacyError::EncryptionFailed)?;

        Ok(EncryptedPayload {
            iv,
            ciphertext: encrypted,
        })
    }
}

// 数据治理
pub struct DataGovernance {
    // 数据分类
    data_classifier: DataClassifier,

    // 保留策略
    retention_policies: HashMap<DataType, RetentionPolicy>,

    // 审计日志
    audit_log: AuditLog,
}

impl DataGovernance {
    pub fn enforce_retention_policy(&self) -> Result<Vec<String>> {
        let mut deleted_keys = Vec::new();

        // 1. 扫描过期数据
        for (key, metadata) in self.local_storage.get_all_metadata()? {
            if self.is_expired(&metadata) {
                // 2. 删除数据
                self.local_storage.delete(&key)?;

                // 3. 记录审计
                self.audit_log.log_event(AuditEvent {
                    event_type: EventType::DataDeleted,
                    key: key.clone(),
                    timestamp: Utc::now(),
                    reason: "Retention policy expired".to_string(),
                });

                deleted_keys.push(key);
            }
        }

        Ok(deleted_keys)
    }

    fn is_expired(&self, metadata: &DataMetadata) -> bool {
        let retention = &self.retention_policies[&metadata.data_type];
        let age = Utc::now().signed_duration_since(metadata.created_at);

        age > retention.max_age
    }
}

// 隐私设置
#[derive(Debug, Serialize, Deserialize)]
pub struct PrivacySettings {
    // 数据本地化
    pub local_only: bool,

    // 加密设置
    pub encryption_enabled: bool,
    pub encryption_level: EncryptionLevel,

    // 数据共享
    pub share_analytics: bool,
    pub share_usage_data: bool,

    // 自动清理
    pub auto_cleanup: bool,
    pub retention_period_days: u32,

    // 数据导出
    pub export_format: ExportFormat,
}
```

**隐私保护策略**

| 数据类型 | 存储方式 | 加密级别 | 保留期 | 共享策略 |
|----------|----------|----------|--------|----------|
| **浏览历史** | 本地SQLite | AES-256 | 永久 | 不共享 |
| **用户偏好** | 本地JSON | AES-256 | 永久 | 不共享 |
| **表单数据** | 内存缓存 | AES-256 | 会话结束删除 | 不共享 |
| **分析数据** | 本地聚合 | 无敏信息 | 30天 | 可选匿名共享 |
| **崩溃日志** | 本地文件 | 脱敏处理 | 7天 | 需用户同意 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 本地化率 | 100%（敏感数据） | 代码审计 |
| 加密强度 | AES-256 | 安全审计 |
| 数据导出 | 100%可导出 | 功能测试 |
| 自动清理 | 100%执行 | 定时任务测试 |
| 审计完整性 | 100%记录 | 日志审查 |

## 📦 新增crate结构

```
crates/
├── ai-memory/
│   ├── src/
│   │   ├── lib.rs           # 记忆系统
│   │   ├── working.rs       # 工作记忆
│   │   ├── long_term.rs     # 长期记忆
│   │   ├── semantic.rs      # 语义记忆
│   │   └── procedural.rs    # 程序记忆
│   └── Cargo.toml
├── ai-knowledge-graph/
│   ├── src/
│   │   ├── lib.rs           # 知识图谱
│   │   ├── graph.rs         # 图结构
│   │   ├── entity.rs        # 实体管理
│   │   └── relation.rs      # 关系推理
│   └── Cargo.toml
├── ai-context-resolver/
│   ├── src/
│   │   ├── lib.rs           # 上下文理解
│   │   ├── anaphora.rs      # 回指解析
│   │   └── resolution.rs    # 指代消解
│   └── Cargo.toml
├── ai-recommender/
│   ├── src/
│   │   ├── lib.rs           # 推荐系统
│   │   ├── interest.rs      # 兴趣模型
│   │   ├── collaborative.rs # 协同过滤
│   │   └── content.rs       # 内容推荐
│   └── Cargo.toml
└── ai-privacy/
    ├── src/
    │   ├── lib.rs           # 隐私管理
    │   ├── encryption.rs    # 加密引擎
    │   ├── governance.rs    # 数据治理
    │   └── audit.rs         # 审计日志
    └── Cargo.toml
```

## 🎬 Demo场景

### Demo-7: 上下文理解
```
场景：用户之前浏览过相机产品
时间线：
  T1: 用户浏览iPhone 15页面
  T2: 用户浏览华为Mate 60页面
  T3: 用户浏览索尼A7M4相机页面
  T4: 用户打开新页面

用户输入："上次的那个相机怎么样？"

AI理解过程：
1. 识别回指示象："上次的" + "相机"
2. 搜索候选实体：
   - iPhone 15 (排除，非相机)
   - 华为Mate 60 (排除，非相机)
   - 索尼A7M4 (匹配，相机)
3. 应用评分：
   - 时间距离：0.9 (最近浏览)
   - 类型匹配：1.0 (都是相机)
   - 访问频率：0.8 (浏览了3分钟)
4. 决策：索尼A7M4 (置信度0.92)
5. 回答：
   "索尼A7M4是全画幅微单，主要特点：
   - 3300万像素，画质出色
   - 7级防抖，手持拍摄稳定
   - 实时追踪对焦，捕捉瞬间
   您想了解它的价格还是详细参数？"
```

### Demo-8: 个性化首页
```
场景：用户打开新标签页

AI推荐过程：
1. 分析用户画像：
   - 技术爱好者 (浏览过大量技术文档)
   - 摄影爱好者 (浏览过相机评测)
   - 科技新闻读者 (经常看科技资讯)

2. 生成个性化推荐：
   ┌──────────────────────────────────────────────┐
   │ 📰 为您推荐                                   │
   ├──────────────────────────────────────────────┤
   │ 🔥 热点新闻                                  │
   │  "苹果发布M3芯片，性能提升40%"                │
   │  → 推荐原因：您关注苹果产品                    │
   │                                              │
   │ 📸 摄影技巧                                  │
   │  "微单相机选购指南"                           │
   │  → 推荐原因：您浏览过相机评测                  │
   │                                              │
   │ 💻 技术文档                                  │
   │  "Rust 1.70新特性详解"                        │
   │  → 推荐原因：您经常查看技术文档                │
   │                                              │
   │ ⚡ 快速访问                                  │
   │  GitHub StackOverflow 知乎                    │
   │  → 基于您的访问频率                           │
   └──────────────────────────────────────────────┘

每个推荐都显示推荐原因，增强可解释性。
```

### Demo-9: 智能表单
```
场景：用户在酒店预订页面
用户说："帮我订上次那种酒店"

AI记忆回溯：
1. 搜索历史：
   - 3个月前住过：如家酒店 (北京)
   - 1个月前查看过：7天连锁 (上海)
   - 偏好：中档连锁酒店、评分>4.0

2. 提取偏好：
   - 品牌：如家 > 7天 > 汉庭
   - 价格区间：150-250元
   - 位置：市中心优先
   - 评分要求：≥ 4.0

3. 自动应用筛选：
   ✅ 自动勾选：如家、7天、汉庭
   ✅ 自动填入价格：150-250元
   ✅ 自动设置评分：≥4.0
   ✅ 显示已应用筛选："基于您的历史偏好"

4. 用户仅需选择入住日期，AI完成其他设置。
```

## 🎯 成功指标

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| **回指消解准确率** | ≥ 90% | 1000个用例 |
| **知识图谱规模** | ≥ 10万节点 | 图数据库统计 |
| **推荐点击率** | ≥ 15% | A/B测试 |
| **记忆检索延迟** | < 20ms | 性能测试 |
| **隐私评分** | 100%本地 | 安全审计 |

---

**Phase 2总结：构建智能记忆系统，让浏览器"有记忆"！** ✅
