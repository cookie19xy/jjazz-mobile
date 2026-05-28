# JJazz Mobile

移动版 JJazzLab — 自动伴奏生成器

## 项目定位

将桌面端 JJazzLab（Java + NetBeans + Swing）移植到 iOS / Android，
架构拆分为三层：**Rust 引擎 + Flutter UI + FluidSynth 音频**。

## 当前进度

| 阶段 | 状态 | 说明 |
|------|------|------|
| Phase 1: Pure Java Core | ✅ 完成 | 19 源文件, 42 测试 |
| Phase 2: Rust 引擎 | ⬜ 待开始 | 算法翻译 + Golden Test |
| Phase 3: Flutter UI | ⬜ 待开始 | 跨平台界面 |

## 技术栈

- **验证层**: Java 21 + Maven + Guava + JUnit 5
- **引擎层**: Rust (待开始)
- **UI 层**: Flutter (待开始)
- **音频层**: FluidSynth C 库 (待开始)
- **参考源**: [JJazzLab](https://github.com/jjazzboss/JJazzLab) (LGPL v2.1, 只读)

## 项目结构

```
jjazz-mobile/
├── jjazz-pure-core/     # Phase 1: Java 核心抽离验证
│   ├── src/main/java/org/jjazz/purecore/
│   │   ├── harmony/     # 音乐理论模型 (Note, ChordType, Scale…)
│   │   ├── phrase/      # 乐句模型 (NoteEvent, Phrase)
│   │   ├── humanizer/   # 人性化引擎
│   │   ├── quantizer/   # 量化引擎
│   │   ├── musicgen/    # 音乐生成基础
│   │   └── util/        # 工具类
│   └── src/test/        # 42 个单元测试
├── jjazz-engine/        # Phase 2: Rust 引擎 (待开始)
├── jjazz-flutter/       # Phase 3: Flutter UI (待开始)
└── docs/                # 文档
    ├── architecture.md
    └── phase1-report.md
```

## 构建 & 测试

```bash
cd jjazz-pure-core
mvn test    # 42 tests, all passing
```

## 许可证

LGPL v2.1 — 继承自原版 [JJazzLab](https://github.com/jjazzboss/JJazzLab)

## 原项目

https://github.com/jjazzboss/JJazzLab — 作者 Jerome Lelasseux, 35,000+ 用户
