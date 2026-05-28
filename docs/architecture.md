# Architecture Decisions

## 为什么分三层

```
Flutter UI  ← 一套代码 iOS+Android
    ↕ dart:ffi (C ABI)
Rust Engine ← 零GC, 实时音频安全
    ↕ C ABI
FluidSynth  ← 成熟的 SoundFont 渲染器
```

## 为什么 Rust 而不是 Dart 做引擎

1. **实时 BPM** 要求可预测延迟 — Dart GC 可能导致音频卡顿
2. **Yamaha Style 解析** 是二进制逆向工程 — Rust 的 pattern matching 天然适合
3. **FFI 直连 FluidSynth C API** — Rust 的 C ABI 兼容性最好
4. **Lock-free MIDI Scheduler** — Rust 的所有权模型天然杜绝数据竞争

## 为什么 Phase 1 用 Java 而不是直接写 Rust

1. 原版代码是 Java，逐行对照翻译最不容易引入错误
2. Golden Test 体系：Java 输出 = 正确答案，Rust 输出必须逐位一致
3. 42 个测试现在就能跑，不需要等 Rust 编译链就绪

## Golden Test 策略

```
原版 JJazzLab  →  MIDI 文件  →  canonical JSON
                                    ↓
Phase 1 Java   →  JSON 输出   →  diff == 0 ✅
Phase 2 Rust   →  JSON 输出   →  diff == 0 ✅
Phase 3 Flutter → JSON 输出   →  diff == 0 ✅
```

## 模块依赖方向

```
harmony  ← 零依赖（仅 Guava）
  ↑
phrase   ← 依赖 harmony
  ↑
humanizer ← 依赖 phrase + harmony
quantizer ← 依赖 harmony
musicgen  ← 依赖 harmony + phrase
```

## 已知简化点（vs 原版 JJazzLab）

| 简化项 | 影响 | 计划 |
|--------|------|------|
| fitDegreeAdvanced 缺少音阶匹配 | 复杂和弦适配可能不准 | Phase 2 Rust 补全 |
| DOM11/MIN11 同时有 3rd 和 11th | 声部排列可能有 b9 冲突 | 需音乐理论审查 |
| ChordSymbol 别名表只覆盖 25 种 | 部分稀有和弦符号解析失败 | 按需补充 |
| ChordSequence 是占位实现 | 不支持小节内和弦切换 | Phase 2 完整重写 |
| Humanizer 随机因子改为内联 new Random() | 不可复现，但不影响正确性 | Phase 2 修复 |
