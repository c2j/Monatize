# Phase 3: 多模态交互 (8周)

## 📋 阶段目标

**核心目标**：语音、图像、手势等自然交互方式，提升用户体验

- ✅ 语音交互（ASR、TTS、语音命令）
- ✅ 视觉理解（图像识别、场景分析、物体检测）
- ✅ 手势控制（摄像头手势识别、动作映射）
- ✅ 多模态融合（语音+视觉+文本综合理解）
- ✅ 自然交互优化（响应速度、准确率、用户体验）

**用户可感知价值**：
- 用户说话"把灯调暗点"，AI理解并立即调节灯光亮度
- 用户手势指向摄像头，AI识别并拍照或录像
- 用户说"有点热"同时扇扇子，AI融合语音+动作自动开空调
- 眼神看向设备并点头，AI理解并控制对应设备

## 🎯 详细任务列表

### P3-T1: 语音交互 (2.5周)

**任务描述**
构建完整的语音交互系统：ASR、TTS、语音命令理解

**技术实现**
```rust
// crates/multimodal/src/voice_interaction.rs
pub struct VoiceInteractionMCP {
    // 自动语音识别
    asr_engine: ASREngine,

    // 文本转语音
    tts_engine: TTSEngine,

    // 语音命令理解
    voice_command_understanding: VoiceCommandUnderstanding,

    // 语音活动检测
    voice_activity_detector: VoiceActivityDetector,

    // 噪音抑制
    noise_suppressor: NoiseSuppressor,
}

// ASR引擎
pub struct ASREngine {
    // 流式识别
    streaming_recognizer: StreamingRecognizer,

    // 离线模型
    offline_model: OfflineASRModel,

    // 云端API（备用）
    cloud_api: Option<CloudASRAPI>,

    // 语言模型
    language_model: LanguageModel,
}

impl ASREngine {
    pub async fn recognize_stream(
        &self,
        audio_stream: impl Stream<Item = AudioChunk>,
    ) -> Result<RecognitionResult> {
        // 1. 语音活动检测
        let vad_result = self.voice_activity_detector.detect(&audio_stream).await?;

        if !vad_result.has_voice {
            return Ok(RecognitionResult::Silence);
        }

        // 2. 噪音抑制
        let clean_audio = self.noise_suppressor.suppress(&vad_result.audio).await?;

        // 3. 流式识别
        let mut interim_results = Vec::new();
        let mut final_result = String::new();

        for chunk in clean_audio {
            let result = self.streaming_recognizer.recognize_chunk(&chunk).await?;

            if result.is_final {
                final_result.push_str(&result.text);
                interim_results.clear();
            } else {
                interim_results.push(result);
            }
        }

        // 4. 后处理
        let processed_text = self.postprocess(&final_result)?;

        Ok(RecognitionResult {
            text: processed_text,
            confidence: self.calculate_confidence(&interim_results),
            is_final: true,
        })
    }
}

// TTS引擎
pub struct TTSEngine {
    // 语音合成模型
    synthesis_model: TTSModel,

    // 语音克隆
    voice_cloning: VoiceCloning,

    // 情感控制
    emotion_control: EmotionControl,
}

impl TTSEngine {
    pub async fn synthesize_speech(
        &self,
        text: &str,
        voice_id: Option<String>,
        emotion: Option<Emotion>,
    ) -> Result<AudioData> {
        // 1. 文本预处理
        let processed_text = self.preprocess_text(text)?;

        // 2. 选择语音
        let voice = if let Some(id) = voice_id {
            self.voice_cloning.get_voice(&id)?
        } else {
            self.get_default_voice()
        };

        // 3. 情感调节
        let configured_emotion = emotion.unwrap_or_default();

        // 4. 语音合成
        let audio = self.synthesis_model.synthesize(&processed_text, &voice, &configured_emotion).await?;

        // 5. 后处理
        let enhanced_audio = self.postprocess_audio(&audio)?;

        Ok(enhanced_audio)
    }
}

// 语音命令理解
pub struct VoiceCommandUnderstanding {
    // 意图识别器
    intent_recognizer: IntentRecognizer,

    // 参数提取器
    parameter_extractor: ParameterExtractor,

    // 上下文理解
    context_understanding: ContextUnderstanding,
}

impl VoiceCommandUnderstanding {
    pub async fn understand_command(
        &self,
        speech_text: &str,
        context: &InteractionContext,
    ) -> Result<VoiceCommand> {
        // 1. 意图识别
        let intent = self.intent_recognizer.recognize(speech_text, context)?;

        // 2. 参数提取
        let parameters = self.parameter_extractor.extract(&speech_text, &intent)?;

        // 3. 上下文理解
        let resolved_context = self.context_understanding.resolve(&intent, &parameters, context)?;

        Ok(VoiceCommand {
            intent,
            parameters,
            context: resolved_context,
            confidence: self.calculate_confidence(&intent, &parameters),
        })
    }
}
```

**语音交互能力**

| 能力 | 技术指标 | 应用场景 |
|------|----------|----------|
| **ASR准确率** | > 95% | 语音命令输入 |
| **TTS自然度** | MOS > 4.0 | 语音反馈 |
| **延迟** | < 300ms | 实时对话 |
| **噪音抑制** | > 20dB | 嘈杂环境 |
| **多语言** | > 10种 | 国际化支持 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 语音识别准确率 | > 95% | 1000小时语音测试 |
| TTS自然度 | > 4.0 MOS | 主观评测 |
| 响应延迟 | < 300ms | 端到端测试 |
| 噪音环境适应 | 噪音<50dB | 噪音测试 |
| 并发处理 | > 10路 | 压力测试 |

---

### P3-T2: 视觉理解 (2.5周)

**任务描述**
构建视觉理解系统：图像识别、场景分析、物体检测

**技术实现**
```rust
// crates/multimodal/src/vision_understanding.rs
pub struct VisionUnderstandingMCP {
    // 图像识别引擎
    image_recognizer: ImageRecognizer,

    // 场景分析器
    scene_analyzer: SceneAnalyzer,

    // 物体检测器
    object_detector: ObjectDetector,

    // 人脸识别
    face_recognizer: FaceRecognizer,

    // OCR引擎
    ocr_engine: OCREngine,
}

// 图像识别引擎
pub struct ImageRecognizer {
    // 分类模型
    classification_model: ClassificationModel,

    // 特征提取器
    feature_extractor: FeatureExtractor,

    // 置信度校准
    confidence_calibrator: ConfidenceCalibrator,
}

impl ImageRecognizer {
    pub async fn recognize_image(&self, image: &ImageData) -> Result<ImageRecognitionResult> {
        // 1. 图像预处理
        let preprocessed = self.preprocess_image(image)?;

        // 2. 特征提取
        let features = self.feature_extractor.extract(&preprocessed)?;

        // 3. 分类预测
        let predictions = self.classification_model.predict(&features)?;

        // 4. 置信度校准
        let calibrated_confidence = self.confidence_calibrator.calibrate(&predictions)?;

        // 5. 结果生成
        let result = self.generate_result(predictions, calibrated_confidence)?;

        Ok(result)
    }
}

// 场景分析器
pub struct SceneAnalyzer {
    // 场景分类器
    scene_classifier: SceneClassifier,

    // 深度估计
    depth_estimator: DepthEstimator,

    // 语义分割
    semantic_segmenter: SemanticSegmenter,
}

impl SceneAnalyzer {
    pub async fn analyze_scene(&self, image: &ImageData) -> Result<SceneAnalysisResult> {
        // 1. 场景分类
        let scene_type = self.scene_classifier.classify(image)?;

        // 2. 深度估计
        let depth_map = self.depth_estimator.estimate(image)?;

        // 3. 语义分割
        let segments = self.semantic_segmenter.segment(image)?;

        // 4. 场景理解
        let understanding = self.understand_scene(&scene_type, &segments)?;

        Ok(SceneAnalysisResult {
            scene_type,
            depth_map,
            segments,
            understanding,
        })
    }
}

// 物体检测器
pub struct ObjectDetector {
    // 检测模型
    detection_model: DetectionModel,

    // 目标跟踪
    object_tracker: ObjectTracker,

    // 姿态估计
    pose_estimator: PoseEstimator,
}

impl ObjectDetector {
    pub async fn detect_objects(&self, image: &ImageData) -> Result<Vec<DetectedObject>> {
        // 1. 目标检测
        let detections = self.detection_model.detect(image)?;

        // 2. 非极大值抑制
        let filtered = self.apply_nms(&detections)?;

        // 3. 分类置信度过滤
        let confident = self.filter_by_confidence(&filtered, 0.5)?;

        // 4. 姿态估计（可选）
        let objects = if self.pose_estimator.is_available() {
            let mut objects = Vec::new();
            for det in confident {
                let pose = self.pose_estimator.estimate(&det.bounding_box)?;
                objects.push(DetectedObject {
                    class: det.class,
                    confidence: det.confidence,
                    bounding_box: det.bounding_box,
                    pose: Some(pose),
                });
            }
            objects
        } else {
            confident.into_iter().map(|d| DetectedObject {
                class: d.class,
                confidence: d.confidence,
                bounding_box: d.bounding_box,
                pose: None,
            }).collect()
        };

        Ok(objects)
    }
}

impl MCPTool for VisionUnderstandingMCP {
    async fn invoke(&self, params: ToolParams) -> Result<ToolResult> {
        let action = params.get_string("action")?;

        match action.as_str() {
            "recognize_image" => {
                let image_data = params.get_image("image")?;
                let result = self.image_recognizer.recognize_image(&image_data).await?;
                Ok(ToolResult::VisionResult(result))
            }
            "analyze_scene" => {
                let image_data = params.get_image("image")?;
                let result = self.scene_analyzer.analyze_scene(&image_data).await?;
                Ok(ToolResult::SceneAnalysis(result))
            }
            "detect_objects" => {
                let image_data = params.get_image("image")?;
                let objects = self.object_detector.detect_objects(&image_data).await?;
                Ok(ToolResult::Objects(objects))
            }
            "recognize_face" => {
                let image_data = params.get_image("image")?;
                let faces = self.face_recognizer.recognize_faces(&image_data).await?;
                Ok(ToolResult::Faces(faces))
            }
            "extract_text" => {
                let image_data = params.get_image("image")?;
                let text = self.ocr_engine.extract_text(&image_data).await?;
                Ok(ToolResult::Text(text))
            }
            _ => Err(ToolError::UnsupportedAction(action)),
        }
    }
}
```

**视觉理解能力**

| 能力 | 技术指标 | 应用场景 |
|------|----------|----------|
| **图像识别准确率** | > 90% | 物体识别 |
| **场景理解** | > 20种场景 | 环境分析 |
| **物体检测** | mAP > 0.8 | 目标检测 |
| **人脸识别** | 准确率 > 99% | 身份识别 |
| **OCR识别** | 准确率 > 95% | 文字提取 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 图像识别准确率 | > 90% | ImageNet测试 |
| 场景分析准确率 | > 85% | ADE20K测试 |
| 物体检测mAP | > 0.8 | COCO测试 |
| 人脸识别准确率 | > 99% | LFW测试 |
| OCR准确率 | > 95% | 测试集验证 |

---

### P3-T3: 手势控制 (2周)

**任务描述**
构建手势识别和控制映射系统

**技术实现**
```rust
// crates/multimodal/src/gesture_control.rs
pub struct GestureControlMCP {
    // 手势识别引擎
    gesture_recognizer: GestureRecognizer,

    // 动作跟踪
    action_tracker: ActionTracker,

    // 手势映射
    gesture_mapper: GestureMapper,

    // 实时处理
    realtime_processor: RealtimeProcessor,
}

// 手势识别引擎
pub struct GestureRecognizer {
    // 手势分类模型
    gesture_classifier: GestureClassifier,

    // 手部关键点检测
    hand_keypoint_detector: HandKeypointDetector,

    // 手势序列分析
    gesture_sequence_analyzer: GestureSequenceAnalyzer,
}

impl GestureRecognizer {
    pub async fn recognize_gesture(
        &self,
        video_frame: &VideoFrame,
    ) -> Result<GestureRecognitionResult> {
        // 1. 手部关键点检测
        let keypoints = self.hand_keypoint_detector.detect(&video_frame)?;

        // 2. 手势分类
        let gesture_type = self.gesture_classifier.classify(&keypoints)?;

        // 3. 手势序列分析
        let sequence_result = self.gesture_sequence_analyzer.analyze(&gesture_type, &keypoints)?;

        Ok(GestureRecognitionResult {
            gesture_type,
            confidence: sequence_result.confidence,
            keypoints,
            is_complete: sequence_result.is_complete,
        })
    }
}

// 动作跟踪器
pub struct ActionTracker {
    // 2D/3D姿态估计
    pose_estimator: PoseEstimator,

    // 动作分类器
    action_classifier: ActionClassifier,

    // 时间序列分析
    temporal_analyzer: TemporalAnalyzer,
}

impl ActionTracker {
    pub async fn track_action(
        &self,
        video_sequence: &[VideoFrame],
    ) -> Result<ActionTrackingResult> {
        // 1. 姿态估计
        let poses = self.estimate_poses(video_sequence)?;

        // 2. 动作分类
        let action_type = self.action_classifier.classify(&poses)?;

        // 3. 时间分析
        let temporal_features = self.temporal_analyzer.analyze(&poses)?;

        Ok(ActionTrackingResult {
            action_type,
            poses,
            temporal_features,
            duration: self.calculate_duration(video_sequence),
        })
    }
}

// 手势映射器
pub struct GestureMapper {
    // 映射规则库
    mapping_rules: Arc<RwLock<HashMap<String, GestureMapping>>>,

    // 上下文适配
    context_adapter: ContextAdapter,
}

impl GestureMapper {
    pub async fn map_gesture_to_action(
        &self,
        gesture: &GestureRecognitionResult,
        context: &InteractionContext,
    ) -> Result<MappedAction> {
        // 1. 查找映射规则
        let rules = self.mapping_rules.read().await;
        let mapping = rules.get(&gesture.gesture_type.to_string())
            .ok_or(GestureError::NoMappingFound)?;

        // 2. 上下文适配
        let adapted_mapping = self.context_adapter.adapt(mapping, context)?;

        // 3. 生成动作
        let action = MappedAction {
            device_id: adapted_mapping.device_id,
            action: adapted_mapping.action,
            parameters: self.extract_parameters(gesture, &adapted_mapping)?,
            confidence: gesture.confidence,
        };

        Ok(action)
    }
}

impl MCPTool for GestureControlMCP {
    async fn invoke(&self, params: ToolParams) -> Result<ToolResult> {
        let action = params.get_string("action")?;

        match action.as_str() {
            "recognize_gesture" => {
                let video_frame = params.get_video_frame("frame")?;
                let result = self.gesture_recognizer.recognize_gesture(&video_frame).await?;
                Ok(ToolResult::Gesture(result))
            }
            "track_action" => {
                let video_sequence = params.get_video_sequence("sequence")?;
                let result = self.action_tracker.track_action(&video_sequence).await?;
                Ok(ToolResult::Action(result))
            }
            "map_to_action" => {
                let gesture = params.get_gesture("gesture")?;
                let context = params.get_context("context")?;
                let mapped = self.gesture_mapper.map_gesture_to_action(&gesture, &context).await?;
                Ok(ToolResult::MappedAction(mapped))
            }
            _ => Err(ToolError::UnsupportedAction(action)),
        }
    }
}
```

**手势控制能力**

| 手势类型 | 识别准确率 | 映射动作 |
|----------|------------|----------|
| **手指手势** | > 90% | 设备开关、亮度调节 |
| **手掌手势** | > 95% | 场景切换、音量控制 |
| **手势轨迹** | > 85% | 自定义操作 |
| **身体姿态** | > 88% | 设备定位、控制 |
| **组合手势** | > 80% | 复杂场景控制 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 手势识别准确率 | > 90% | 1000次手势测试 |
| 实时延迟 | < 100ms | 实时性测试 |
| 误触率 | < 5% | 误触测试 |
| 支持手势类型 | > 20种 | 功能测试 |
| 并发识别 | > 5人 | 多人测试 |

---

### P3-T4: 多模态融合 (1周)

**任务描述**
融合语音、视觉、手势等多种模态，实现综合理解

**技术实现**
```rust
// crates/multimodal/src/multimodal_fusion.rs
pub struct MultimodalFusionEngine {
    // 特征提取器
    feature_extractors: HashMap<Modality, Box<dyn FeatureExtractor>>,

    // 特征对齐
    feature_aligner: FeatureAligner,

    // 融合网络
    fusion_network: FusionNetwork,

    // 决策融合
    decision_fusion: DecisionFusion,
}

// 融合网络
pub struct FusionNetwork {
    // 早期融合
    early_fusion: EarlyFusion,

    // 晚期融合
    late_fusion: LateFusion,

    // 混合融合
    hybrid_fusion: HybridFusion,
}

impl MultimodalFusionEngine {
    pub async fn fuse_modalities(
        &self,
        inputs: &MultimodalInputs,
    ) -> Result<FusedUnderstanding> {
        // 1. 特征提取
        let features = self.extract_features(inputs)?;

        // 2. 特征对齐
        let aligned_features = self.feature_aligner.align(&features)?;

        // 3. 选择融合策略
        let fusion_strategy = self.select_fusion_strategy(&inputs.modalities)?;

        // 4. 执行融合
        let fused_representation = match fusion_strategy {
            FusionStrategy::Early => {
                self.fusion_network.early_fusion.fuse(&aligned_features)?
            }
            FusionStrategy::Late => {
                let intermediate_results = self.compute_intermediate(&features)?;
                self.fusion_network.late_fusion.fuse(&intermediate_results)?
            }
            FusionStrategy::Hybrid => {
                self.fusion_network.hybrid_fusion.fuse(&aligned_features, &features)?
            }
        };

        // 5. 生成理解结果
        let understanding = self.generate_understanding(&fused_representation)?;

        Ok(understanding)
    }
}

// 决策融合
pub struct DecisionFusion {
    // 投票机制
    voting_system: VotingSystem,

    // 置信度加权
    confidence_weighting: ConfidenceWeighting,

    // 冲突解决
    conflict_resolver: ConflictResolver,
}

impl DecisionFusion {
    pub async fn fuse_decisions(
        &self,
        decisions: &[ModalityDecision],
    ) -> Result<FusedDecision> {
        // 1. 置信度加权
        let weighted_decisions = self.confidence_weighting.weight(decisions)?;

        // 2. 冲突检测
        let conflicts = self.detect_conflicts(&weighted_decisions)?;

        // 3. 冲突解决
        let resolved_decisions = if !conflicts.is_empty() {
            self.conflict_resolver.resolve(&conflicts, &weighted_decisions)?
        } else {
            weighted_decisions
        };

        // 4. 最终投票
        let final_decision = self.voting_system.vote(&resolved_decisions)?;

        Ok(final_decision)
    }
}
```

**多模态融合能力**

| 融合类型 | 技术方案 | 应用场景 |
|----------|----------|----------|
| **早期融合** | 特征级融合 | 简单任务、快速响应 |
| **晚期融合** | 决策级融合 | 复杂任务、高准确率 |
| **混合融合** | 分层融合 | 平衡性能和准确率 |
| **注意力融合** | 动态权重 | 多模态注意力机制 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 融合准确率 | > 85% | 多模态测试集 |
| 响应延迟 | < 500ms | 端到端测试 |
| 鲁棒性 | 缺失1模态仍可用 | 鲁棒性测试 |
| 计算效率 | < 2x单模态 | 性能测试 |
| 用户满意度 | > 4.2/5 | 用户测试 |

        // 3. 用户意图融合
        let intent_enhanced = self.merge_intent(&classifications, intent);

        // 4. 生成标注
        let annotations = self.annotation_generator.generate(&intent_enhanced);

        SemanticMap {
            annotations,
            element_map: self.build_element_map(&classifications),
            interaction_hints: self.generate_hints(&annotations),
            confidence: self.calculate_confidence(&annotations),
        }
    }
}

// 语义标注类型
#[derive(Debug, Clone)]
pub struct SemanticAnnotation {
    pub element_id: DomNodeId,
    pub annotation_type: AnnotationType,
    pub label: String,
    pub importance: ImportanceLevel,  // High/Medium/Low
    pub position: AnnotationPosition,
    pub style: AnnotationStyle,
    pub interactive: bool,
    pub action: Option<SemanticAction>,
}

#[derive(Debug, Clone)]
pub enum AnnotationType {
    // 电商
    Price, Discount, Rating, Stock, Shipping,

    // 新闻
    Headline, Author, PublishTime, Source, Tags,

    // 文档
    Title, Subtitle, Paragraph, Code, Link, API,

    // 表单
    Field, Label, Validation, Required,

    // 通用
    Button, Link, Image, Video, Navigation, Search,
}

pub struct SemanticMap {
    pub annotations: Vec<SemanticAnnotation>,
    pub element_map: HashMap<DomNodeId, ElementSemantic>,
    pub interaction_hints: Vec<InteractionHint>,
    pub confidence: f32,
}

// 智能高亮
pub struct AttentionGuidance {
    heatmap_generator: HeatmapGenerator,
    highlight_renderer: HighlightRenderer,
}

impl AttentionGuidance {
    fn generate_heatmap(&self, page: &Page, user_attention: &UserAttentionModel) -> Heatmap {
        let mut heatmap = Heatmap::new(page.size);

        for gaze_point in &user_attention.gaze_points {
            let intensity = self.calculate_intensity(gaze_point, &user_attention.dwell_time);
            heatmap.add_point(gaze_point.x, gaze_point.y, intensity);
        }

        heatmap.apply_blur();
        heatmap
    }

    fn render_highlights(&self, base_pixels: &[u8], heatmap: &Heatmap) -> Vec<u8> {
        // 1. 将热力图叠加到渲染结果
        // 2. 根据注意力分配透明度
        // 3. 高兴趣区域轻微高亮
        // 4. 低兴趣区域略微淡化
        // 5. 保持可读性
    }
}
```

**语义标注策略**

| 页面类型 | 关键标注 | 高亮策略 | 交互提示 |
|----------|----------|----------|----------|
| **电商页面** | 价格、促销、评分、发货 | 红色高亮价格，绿色显示折扣 | 悬停显示历史价格 |
| **新闻文章** | 标题、作者、时间、关键事实 | 标题加粗，时间淡化 | 点击展开详细评价 |
| **技术文档** | API、代码示例、注意事项 | 代码块边框高亮 | 点击复制代码 |
| **学术论文** | 摘要、结论、方法、引用 | 关键结论黄色标记 | 点击查看图表详情 |
| **表单页面** | 必填项、验证规则 | 红色标记必填项 | 实时验证提示 |

**性能优化**

| 优化策略 | 实现方法 | 性能提升 |
|----------|----------|----------|
| **异步语义分析** | 不阻塞渲染管线 | 保持60FPS |
| **增量更新** | 仅重绘变化区域 | 减少30%渲染时间 |
| **纹理复用** | 缓存静态元素 | 节省50%GPU内存 |
| **LOD渲染** | 远距离降低质量 | 提升20%帧率 |
| **批处理** | 合并标注绘制调用 | 减少40%draw calls |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 渲染帧率 | 55-60 FPS | GPU Profiler |
| 语义标注准确率 | > 92% | 人工对比 |
| 页面类型支持 | 5种 | 功能测试 |
| 可点击区域识别 | > 95% | 点击测试 |
| 性能影响 | < 5%帧率损失 | 对比测试 |

---

### P3-T2: 动态UI生成 (2周)

**任务描述**
根据用户意图生成交互式UI（对比表、图表、思维导图）

**技术实现**
```rust
// crates/ai-renderer/src/dynamic_ui.rs
pub struct DynamicUIGenerator {
    data_extractor: DataExtractor,
    visualization_engine: VisualizationEngine,
    layout_engine: LayoutEngine,
}

impl DynamicUIGenerator {
    fn generate_ui(&self, intent: &UserIntent, page: &Page) -> Result<DynamicUI> {
        match intent {
            UserIntent::Compare { item_a, item_b, .. } => {
                self.generate_comparison_ui(item_a, item_b)
            }
            UserIntent::Analyze { data_type, .. } => {
                self.generate_visualization_ui(data_type, page)
            }
            UserIntent::Extract { pattern, format } => {
                self.generate_extraction_ui(pattern, format, page)
            }
            UserIntent::Learn { topic, .. } => {
                self.generate_study_ui(topic, page)
            }
            _ => Err(UIError::UnsupportedIntent),
        }
    }

    fn generate_comparison_ui(&self, item_a: &String, item_b: &String) -> Result<DynamicUI> {
        // 1. 提取商品数据
        let product_a = self.extract_product_data(item_a)?;
        let product_b = self.extract_product_data(item_b)?;

        // 2. 对齐属性
        let comparison_table = self.align_properties(&product_a, &product_b);

        // 3. 生成交互式表格
        let ui = DynamicUI::ComparisonTable {
            columns: vec![
                "属性".to_string(),
                item_a.clone(),
                item_b.clone(),
                "差异".to_string(),
            ],
            rows: comparison_table.rows,
            sortable: true,
            filterable: true,
            highlight_differences: true,
        };

        Ok(ui)
    }
}

pub enum DynamicUI {
    ComparisonTable {
        columns: Vec<String>,
        rows: Vec<ComparisonRow>,
        sortable: bool,
        filterable: bool,
        highlight_differences: bool,
    },
    DataVisualization {
        chart_type: ChartType,
        data: Vec<DataPoint>,
        options: ChartOptions,
        interactive: bool,
    },
    StudyNotes {
        summary: String,
        key_concepts: Vec<Concept>,
        related_topics: Vec<String>,
        quiz_questions: Vec<QuizQuestion>,
    },
    MindMap {
        center_topic: String,
        branches: Vec<MindMapBranch>,
        interactive: bool,
        exportable: bool,
    },
}

// 图表生成
pub struct VisualizationEngine {
    chartjs_adapter: ChartJSAdapter,
    d3_adapter: D3Adapter,
}

impl VisualizationEngine {
    fn create_chart(&self, data: &TableData) -> Chart {
        // 1. 数据类型分析
        let data_type = self.analyze_data_type(data);

        // 2. 选择最佳图表类型
        let chart_type = match data_type {
            DataType::TimeSeries => ChartType::Line,
            DataType::Categorical => ChartType::Bar,
            DataType::Numerical => ChartType::Scatter,
            DataType::Proportion => ChartType::Pie,
            _ => ChartType::Table,
        };

        // 3. 生成配置
        let config = ChartConfig {
            chart_type,
            data: data.clone(),
            options: ChartOptions {
                responsive: true,
                interactive: true,
                animation: true,
                export_format: vec!["PNG", "SVG", "PDF"],
            },
        };

        Chart { config }
    }
}

// 思维导图生成
pub struct MindMapGenerator {
    text_analyzer: TextAnalyzer,
    concept_extractor: ConceptExtractor,
    relation_inferrer: RelationInferrer,
}

impl MindMapGenerator {
    fn generate_mindmap(&self, content: &str) -> MindMap {
        // 1. 提取关键概念
        let concepts = self.concept_extractor.extract(content);

        // 2. 推理概念关系
        let relations = self.relation_inferrer.infer(&concepts);

        // 3. 构建树形结构
        let mindmap = self.build_tree_structure(&concepts, &relations);

        MindMap {
            center: mindmap.center,
            branches: mindmap.branches,
            interactive: true,
        }
    }
}
```

**动态UI类型**

| UI类型 | 生成场景 | 核心技术 | 交互能力 |
|--------|----------|----------|----------|
| **对比表格** | 比较商品/方案 | 表格对齐算法 | 排序/筛选/高亮 |
| **数据图表** | 数据可视化 | Chart.js/D3 | 缩放/筛选/导出 |
| **思维导图** | 知识结构 | 树形布局算法 | 展开/折叠/编辑 |
| **学习笔记** | 文档理解 | 摘要提取 | 重点标记/测验 |
| **时间线** | 事件分析 | 时间排序 | 过滤/缩放 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| UI生成延迟 | < 300ms | 性能测试 |
| 图表响应性 | > 60FPS | 交互测试 |
| 导出格式支持 | PNG/SVG/PDF | 功能测试 |
| 数据准确性 | 100% | 数据对比 |
| 交互流畅度 | < 16ms延迟 | 用户体验测试 |

---

### P3-T3: 智能交互提示 (1.5周)

**任务描述**
基于用户意图的实时提示，不遮挡内容

**技术实现**
```rust
// crates/ai-renderer/src/interaction_hints.rs
pub struct InteractionHintSystem {
    intent_predictor: IntentPredictor,
    hint_generator: HintGenerator,
    placement_optimizer: PlacementOptimizer,
}

impl InteractionHintSystem {
    fn generate_hints(&self, context: &InteractionContext) -> Vec<InteractionHint> {
        // 1. 预测用户意图
        let predicted_intents = self.intent_predictor.predict(&context);

        // 2. 生成提示候选
        let hint_candidates = self.generate_candidates(&predicted_intents);

        // 3. 选择最佳提示
        let selected_hints = self.select_best_hints(&hint_candidates);

        // 4. 优化位置
        let optimized_hints = self.placement_optimizer.optimize(&selected_hints);

        optimized_hints
    }

    fn predict_intent(&self, context: &InteractionContext) -> Vec<PredictedIntent> {
        let mut intents = Vec::new();

        // 基于当前页面
        if let Some(page_type) = context.page_type {
            let page_intents = self.predict_from_page(page_type);
            intents.extend(page_intents);
        }

        // 基于用户行为
        let behavior_intents = self.predict_from_behavior(&context.user_behavior);
        intents.extend(behavior_intents);

        // 基于时间
        let time_intents = self.predict_from_time(context.timestamp);
        intents.extend(time_intents);

        // 基于上下文
        let context_intents = self.predict_from_context(context);
        intents.extend(context_intents);

        // 按置信度排序
        intents.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        intents.into_iter().take(3).collect()
    }
}

// 提示类型
#[derive(Debug)]
pub enum InteractionHint {
    Tooltip {
        content: String,
        position: HintPosition,
        trigger: HintTrigger,
        dismissible: bool,
    },
    InlineHint {
        text: String,
        position: HintPosition,
        style: HintStyle,
    },
    FloatingButton {
        icon: String,
        action: String,
        position: HintPosition,
        priority: HintPriority,
    },
    ContextMenu {
        items: Vec<MenuItem>,
        position: HintPosition,
    },
    InlineNotification {
        text: String,
        type_: NotificationType,
        duration: Duration,
    },
}

pub struct HintPosition {
    pub x: f32,
    pub y: f32,
    pub alignment: Alignment,  // Top/Bottom/Left/Right/Auto
    pub offset: (f32, f32),
}

pub enum HintTrigger {
    Hover,
    Click,
    Selection,
    Focus,
    Timed(Duration),
    Contextual,
}
```

**提示策略**

| 场景 | 提示类型 | 显示时机 | 隐藏条件 |
|------|----------|----------|----------|
| **初次使用** | 引导提示 | 首次访问页面 | 用户关闭或3秒后 |
| **选择文本** | 工具提示 | 文本选中时 | 取消选择 |
| **异常操作** | 警告提示 | 操作失败时 | 操作成功或15秒 |
| **智能建议** | 悬浮按钮 | AI识别需求时 | 用户忽略3次 |
| **系统通知** | 内联提示 | 后台任务完成 | 用户查看后 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 提示准确率 | > 90% | 用户反馈 |
| 提示延迟 | < 100ms | 性能测试 |
| 可自定义性 | 100% | 功能测试 |
| 无干扰性 | 100%不遮挡内容 | 用户调查 |
| 关闭率 | < 20% | 行为分析 |

---

### P3-T4: ai-multimodal多模态支持 (1.5周)

**任务描述**
图像理解、语音识别、TTS语音播报

**技术实现**
```rust
// crates/ai-multimodal/src/lib.rs
pub struct MultimodalProcessor {
    // 视觉理解
    vision_model: VisionModel,

    // 语音识别
    asr_model: ASRModel,

    // 语音合成
    tts_model: TTSModel,

    // 多模态融合
    fusion_engine: FusionEngine,
}

impl MultimodalProcessor {
    async fn process_image(&self, image_data: &ImageData) -> VisionResult {
        // 1. 预处理图像
        let preprocessed = self.preprocess_image(image_data);

        // 2. 视觉模型推理
        let features = self.vision_model.extract_features(&preprocessed).await?;

        // 3. 场景理解
        let scene_understanding = self.scene_understand(&features);

        // 4. 元素检测
        let elements = self.detect_elements(&features);

        // 5. 生成描述
        let description = self.generate_description(&scene_understanding, &elements);

        VisionResult {
            scene_type: scene_understanding.scene_type,
            confidence: scene_understanding.confidence,
            elements,
            description,
            actionable_items: self.extract_actionable_items(&elements),
        }
    }

    async fn process_audio(&self, audio_data: &AudioData) -> ASRResult {
        // 1. 音频预处理
        let features = self.extract_audio_features(audio_data);

        // 2. 语音识别
        let transcription = self.asr_model.transcribe(&features).await?;

        // 3. 意图理解
        let intent = self.intent_recognizer.recognize(&transcription);

        ASRResult {
            text: transcription,
            intent,
            confidence: self.asr_model.confidence(),
            language: self.detect_language(&features),
        }
    }

    async fn synthesize_speech(&self, text: &str, voice: &VoiceConfig) -> AudioData {
        // 1. 文本预处理
        let processed_text = self.preprocess_text(text);

        // 2. 语音合成
        let audio_buffer = self.tts_model.synthesize(&processed_text, voice).await?;

        // 3. 后处理
        let enhanced_audio = self.post_process(&audio_buffer);

        enhanced_audio
    }
}

// 视觉理解模型
pub struct VisionModel {
    // 使用LLaVA-1.5进行图像理解
    llava_model: LlavaModel,

    // 表格识别
    table_detector: TableDetector,

    // 图表识别
    chart_analyzer: ChartAnalyzer,

    // OCR引擎
    ocr_engine: OCREngine,
}

impl VisionModel {
    async fn understand_webpage(&self, screenshot: &Screenshot) -> WebpageUnderstanding {
        // 1. 布局分析
        let layout = self.analyze_layout(&screenshot);

        // 2. 文本提取
        let text_elements = self.extract_text(&screenshot);

        // 3. 图像分析
        let image_elements = self.analyze_images(&screenshot);

        // 4. 交互元素检测
        let interactive_elements = self.detect_interactive(&screenshot);

        // 5. 多模态融合
        let understanding = self.fuse_modalities(&layout, &text_elements, &image_elements, &interactive_elements);

        WebpageUnderstanding {
            layout,
            text: text_elements,
            images: image_elements,
            interactive: interactive_elements,
            semantic_structure: understanding.semantic_structure,
        }
    }
}

// 语音交互
pub struct VoiceInterface {
    processor: MultimodalProcessor,
    conversation_manager: ConversationManager,
    wake_word_detector: WakeWordDetector,
}

impl VoiceInterface {
    async fn handle_voice_command(&mut self) -> Result<VoiceResponse> {
        // 1. 唤醒词检测
        if !self.wake_word_detector.detect().await? {
            return Ok(VoiceResponse::None);
        }

        // 2. 语音识别
        let audio = self.capture_audio().await?;
        let asr_result = self.processor.process_audio(&audio).await?;

        // 3. 意图理解
        let intent = self.parse_intent(&asr_result.text);

        // 4. 执行操作
        let action_result = self.execute_intent(&intent).await?;

        // 5. 生成回复
        let response = self.generate_response(&action_result);

        // 6. 语音合成
        let audio_response = self.processor.synthesize_speech(&response, &self.voice_config).await?;

        // 7. 播放回复
        self.play_audio(&audio_response).await?;

        Ok(VoiceResponse::Success(response))
    }
}
```

**多模态能力**

| 能力 | 技术实现 | 应用场景 |
|------|----------|----------|
| **图像理解** | LLaVA-1.5-7B | 截图分析、图表理解 |
| **语音识别** | Whisper-large | 语音控制、输入 |
| **语音合成** | Piper-TTS | 播报内容、反馈 |
| **视频分析** | CLIP + 时间注意力 | 视频摘要、关键帧 |
| **OCR** | PaddleOCR + ViT | 文本提取、翻译 |

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 图像理解准确率 | > 88% | 标准测试集 |
| 语音识别准确率 | > 95%(中文) | 语音测试集 |
| 语音响应延迟 | < 500ms | 性能测试 |
| TTS自然度 | > 4.0/5.0 | 用户评分 |
| 实时性 | < 100ms延迟 | 端到端测试 |

---

### P3-T5: 完整整合测试 (0.5周)

**任务描述**
10个场景端到端验证

**技术实现**
```rust
// ci/smoke-3.py - Phase 3 E2E测试
def test_end_to_end_scenarios():
    scenarios = [
        # 场景1: 智能购物助手
        "智能购物助手",
        # 用户浏览电商页面
        # AI自动标注价格、评分、促销
        # 用户说"比较这两个商品"
        # AI生成对比表，突出差异
        # 验证对比表准确性

        # 场景2: 学术阅读助手
        "学术阅读助手",
        # 打开PDF论文
        # AI自动生成摘要、关键结论
        # 用户问"这个方法的创新点是什么？"
        # AI基于论文内容回答
        # 验证答案准确性

        # 场景3: 语音导航
        "语音导航",
        # 语音说"帮我找附近的川菜餐厅"
        # ASR识别 → 地图搜索 → 筛选川菜
        # 显示结果 → 播报结果数量
        # 验证导航成功

        # 场景4: 智能填表
        "智能填表",
        # 打开注册表单
        # AI自动识别字段
        # 从用户档案匹配数据
        # 自动填写并标记需确认的字段
        # 验证填写准确性

        # 场景5: 数据可视化
        "数据可视化",
        # 打开包含表格的页面
        # 用户说"显示价格趋势"
        # AI提取数据 → 生成图表 → 高亮趋势
        # 验证图表正确性

        # 场景6: 上下文理解
        "上下文理解",
        # 之前浏览过相机页面
        # 用户说"上次的相机降价了吗？"
        # AI回忆并查询价格历史
        # 验证回指消解准确性

        # 场景7: 智能学习
        "智能学习",
        # 打开技术文档
        # AI标注API、示例、注意事项
        # 用户选择术语查看解释
        # AI生成代码示例和最佳实践
        # 验证解释质量

        # 场景8: 个性化推荐
        "个性化推荐",
        # 打开新标签页
        # AI基于历史推荐内容
        # 显示推荐原因
        # 验证推荐相关性

        # 场景9: 跨模态搜索
        "跨模态搜索",
        # 上传图片
        # AI理解图片内容
        # 基于图片搜索相关信息
        # 验证搜索结果相关性

        # 场景10: 智能工作流
        "智能工作流",
        # "帮我做市场调研"
        # AI理解意图 → 搜索报告 → 提取数据 → 生成图表 → 保存结果
        # 验证工作流完整性
    ]

    for scenario in scenarios:
        run_scenario(scenario)
        assert_scenario_passed(scenario)
```

**验收标准**
| 标准 | 目标值 | 验证方法 |
|------|--------|----------|
| 场景通过率 | 100% (10/10) | 端到端测试 |
| 平均任务时间 | < 30s | 场景计时 |
| 满意度 | > 4.5/5 | 用户评分 |
| 崩溃率 | 0% | 稳定性测试 |
| 阻塞Bug | 0 | 代码审查 |

## 📦 修改crate结构

```
crates/
├── gpu-compositor/            # 升级为 ai-renderer
│   ├── src/
│   │   ├── lib.rs           # 主渲染器
│   │   ├── semantic.rs      # 语义标注
│   │   ├── dynamic_ui.rs    # 动态UI
│   │   ├── hints.rs         # 交互提示
│   │   └── overlay.rs       # 标注叠加
│   └── Cargo.toml
└── ai-multimodal/
    ├── src/
    │   ├── lib.rs           # 多模态处理
    │   ├── vision.rs        # 视觉理解
    │   ├── asr.rs           # 语音识别
    │   ├── tts.rs           # 语音合成
    │   └── fusion.rs        # 多模态融合
    └── Cargo.toml
```

## 🎬 完整Demo场景

### Demo-10: 智能阅读模式
```
场景：用户打开学术论文PDF

1. AI自动解析论文结构
   → 提取：标题、作者、摘要、正文、图表、参考文献

2. 生成语义标注
   → 标题：绿色粗体
   → 摘要：蓝色背景
   → 方法：黄色高亮
   → 结论：橙色边框
   → 引用：灰色淡化

3. 侧边栏智能笔记
   ┌─────────────────────┐
   │ 📝 智能笔记         │
   ├─────────────────────┤
   │ 📊 研究方法         │
   │  - 实验设计：A/B测试 │
   │  - 样本量：1000     │
   │  - 置信区间：95%    │
   │                     │
   │ 🎯 关键结论         │
   │  - 方法X比方法Y高效40%│
   │  - 统计显著性 p<0.01│
   │                     │
   │ 🔗 相关链接         │
   │  → 查看完整数据     │
   │  → 下载补充材料     │
   └─────────────────────┘

4. 用户交互
   → 点击图表 → 查看高清大图
   → 点击引用 → 跳转到参考文献
   → 选择术语 → AI解释含义
   → 问AI问题 → 基于论文内容回答
```

### Demo-11: 语音浏览器
```
场景：用户通过语音控制浏览器

用户："帮我找附近的川菜餐厅，人均100左右"

处理流程：
1. ASR识别
   → 文本："帮我找附近的川菜餐厅，人均100左右"

2. 意图理解
   → 任务：搜索餐厅
   → 菜系：川菜
   → 价格：人均100元
   → 范围：附近

3. 执行搜索
   → 调用地图API → 搜索川菜餐厅
   → 筛选条件：人均消费 ≤ 120元
   → 距离排序

4. 结果处理
   → 提取：店名、评分、价格、距离
   → 生成摘要：找到3家川菜餐厅

5. 语音播报
   → TTS："找到3家川菜餐厅。排名第1的麻辣诱惑，距离您1.2公里，人均消费95元，评分4.5星。排名第2的..."

6. 用户选择
   → 用户："给第一家打电话"
   → AI识别 → 提取电话 → 调起拨号应用
```

### Demo-12: 智能数据可视化
```
场景：用户上传Excel表格

1. AI分析表格结构
   → 检测列：日期、收入、支出、利润
   → 数据类型：时间序列、数值
   → 数据质量：完整无缺失

2. 智能图表推荐
   → 数据类型：时间序列 → 推荐折线图
   → 多变量：收入/支出 → 推荐双轴图
   → 趋势：推荐趋势线

3. 生成交互式图表
   ┌─────────────────────────────────────┐
   │ 📈 收入支出趋势分析                  │
   ├─────────────────────────────────────┤
   │  Y轴：金额(万元)                     │
   │  ┌─────────────────────────────────┐ │
   │  │    /\     /\                    │ │
   │  │   /  \   /  \   /\              │ │
   │  │  /    \ /    \ /  \   /\        │ │
   │  └─────────────────────────────────┘ │
   │  X轴：时间(月份)                      │
   │                                     │
   │ 操作：                               │
   │ [缩放] [筛选] [导出PNG] [导出CSV]      │
   └─────────────────────────────────────┘

4. 智能洞察
   → "3月份收入突增32%，主要来自新客户"
   → "支出与收入相关性0.85，成本控制良好"
   → "建议：5-7月为淡季，可优化营销策略"

5. 用户调整
   → 用户："显示同比增长"
   → AI计算同比增长率 → 添加同比曲线
   → 用户："高亮异常点"
   → AI检测异常值 → 红点标记
```

## 🎯 成功指标

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| **渲染帧率** | 55-60 FPS | GPU Profiler |
| **语义标注准确率** | ≥ 92% | 人工对比 |
| **多模态识别准确率** | ≥ 88% | 标准测试集 |
| **端到端任务时间** | < 30s | 10个场景 |
| **用户满意度** | > 4.5/5 | 用户调研 |

---

**Phase 3总结：完成AI渲染整合，实现智能展示！** ✅
