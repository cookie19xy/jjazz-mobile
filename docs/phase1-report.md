# Phase 1 Completion Report

## 目标

从原版 JJazzLab (Java + NetBeans + Swing) 中剥离出纯算法核心，
去除所有 UI/平台依赖，建立可验证的 Golden Test 基准。

## 交付物

### 19 个源文件 (2,406 行)

| 包 | 文件 | 行数 | 来源 |
|----|------|------|------|
| harmony | Note.java | 378 | model/Harmony |
| harmony | ChordType.java | 387 | model/Harmony |
| harmony | Chord.java | 205 | model/Harmony |
| harmony | ChordSymbol.java | 200 | model/Harmony |
| harmony | Degree.java | 123 | model/Harmony |
| harmony | Scale.java | 109 | model/Harmony |
| harmony | TimeSignature.java | 95 | model/Harmony |
| harmony | Position.java | 94 | model/Harmony |
| harmony | SymbolicDuration.java | 82 | model/Harmony |
| harmony | ChordTypes.java | 78 | model/Harmony (SPI 替代) |
| harmony | StandardScales.java | 65 | model/Harmony |
| phrase | Phrase.java | 94 | model/Phrase |
| phrase | NoteEvent.java | 84 | model/Phrase |
| phrase | Phrases.java | 65 | model/Phrase |
| humanizer | Humanizer.java | 116 | core/Humanizer |
| quantizer | Quantizer.java | 105 | core/Quantizer |
| musicgen | ChordSequence.java | 41 | core/RhythmMusicGeneration |
| util | FloatRange.java | 46 | core/Utilities |
| util | IntRange.java | 39 | core/Utilities |

### 3 个测试文件, 42 个用例

| 测试类 | 用例数 | 覆盖范围 |
|--------|--------|----------|
| HarmonyModelTest | 25 | Note, Degree, ChordType, ChordSymbol, Scale, Position, TimeSignature, Chord |
| PhraseModelTest | 14 | NoteEvent, Phrase, FloatRange, IntRange, Humanizer, Quantizer |
| EndToEndTest | 3 | 和弦解析 → 贝斯线生成 → 人性化 → 量化 全管线 |

## 剥离的依赖

| 移除项 | 替代方案 |
|--------|----------|
| NetBeans Platform (Lookup, ServiceProvider, TopComponent) | 完全删除 |
| Swing / AWT | 完全删除 |
| javax.sound.midi Sequencer | 暂不需要（Phase 1 不播放） |
| XStream | Gson (JSON 输出) |
| ResUtil (i18n) | 硬编码英文 |
| ChordTypeDatabase SPI | 内联 ChordTypes 注册表 |
| StandardScaleInstance | fitDegreeAdvanced 接受 null |
| ObservableProperties | 直接字段 |
| UndoManager | 删除 |

## 保留的依赖

| 依赖 | 用途 |
|------|------|
| Google Guava | Preconditions.checkArgument() |
| Gson | JSON 序列化 |
| JUnit 5 | 测试 |

## 测试结果

```
Tests run: 42, Failures: 0, Errors: 0, Skipped: 0
BUILD SUCCESS
```

## 已知问题（诚实清单）

1. `ChordSymbol("Cmaj")` 错误映射到 CM7 而非 C 三和弦
2. fitDegreeAdvanced 缺少原版的 Scale-based 音阶匹配
3. DOM11/MIN11 和弦定义中同时存在 3rd 和 11th（原版禁止）
4. Humanizer 随机种子不可复现
5. ChordSequence 为占位实现，不支持小节内和弦变化
