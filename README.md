# JJazz Mobile

移动版 JJazzLab — 自动伴奏生成器

## 项目定位

将桌面端 JJazzLab（Java + NetBeans + Swing）移植到 iOS / Android。
架构：**Rust 引擎 + Flutter UI + FluidSynth 音频**。

## 当前进度

| 阶段 | 状态 | 说明 |
|------|------|------|
| 音乐理论模型 | ✅ | Note, Degree, ChordType(50+), ChordSymbol, Scale, TimeSignature, Position |
| 乐句模型 | ✅ | Phrase, NoteEvent, SourcePhrase |
| 和弦适配 | ✅ | 6 种 Retrigger 规则, fitMelody/Bass/Chord pipeline |
| 人性化 + 量化 | ✅ | Humanizer, Quantizer |
| 风格系统 | ✅ | AccType, Style, StylePart, ChannelSettings |
| **SMF 风格解析器** | ✅ | 解析 .prs/.sty/.yjz 为标准 MIDI，提取真实乐句 |
| **真实风格播放** | ✅ | 8247 音符/3和弦，词曲分离播放 |
| 音频合成 | ✅ | rustysynth (纯 Rust SoundFont 合成器, 44100Hz stereo) |
| 模式库 | ⚠️ | 手写 Bossa/Swing/Rock patterns (正在被风格文件替代) |
| AccentProcessor | ❌ | Hold/Shot/Extended 和弦处理 |
| CASM 解析 | ❌ | 从风格文件读取 Ctb2ChannelSettings |
| 流式引擎 | ❌ | 按小节实时生成, BPM 可变 |
| Flutter UI | ❌ | 跨平台界面 |
| FFI 桥接 | ❌ | Rust ↔ Flutter C-ABI |

**测试**: 22 tests, all passing (14 unit + 8 integration)

## 技术栈

- **引擎**: Rust (edition 2021)
- **依赖**: serde, serde_json, rand, rustysynth, midly
- **UI**: Flutter (待开始)
- **音频**: rustysynth → iOS/Android (零 C 依赖)
- **参考源**: [JJazzLab](https://github.com/jjazzboss/JJazzLab) (LGPL v2.1)

## 项目结构

```
jjazz-mobile/
├── jjazz-engine/          # Rust 引擎 (33 源文件)
│   ├── src/
│   │   ├── harmony/       # 音乐理论 (note, degree, chord_type, chord_symbol, scale, time_signature, position)
│   │   ├── phrase/        # 乐句模型 (phrase, note_event)
│   │   ├── humanizer.rs   # 人性化
│   │   ├── quantizer.rs   # 量化
│   │   ├── retrigger.rs   # 6 种和弦适配规则
│   │   ├── source_phrase.rs  # SourcePhrase + fit_*_to_chord pipeline
│   │   ├── style.rs       # AccType, Style, StylePart, ChannelSettings
│   │   ├── style_parser.rs   # SMF 格式解析器 (.prs/.sty/.yjz)
│   │   ├── style_player.rs   # 真实风格文件 → 和弦适配播放
│   │   ├── patterns.rs    # 手写 Bossa/Swing 模式库 (即将废弃)
│   │   ├── synth.rs       # rustysynth 音频渲染
│   │   ├── musicgen/      # 生成器 (generate_with_style, generate_clean)
│   │   └── bin/
│   │       ├── jjazz-demo.rs   # demo: 和弦 → WAV
│   │       ├── jjazz-export.rs # 导出 golden baseline JSON
│   │       ├── parse-style.rs  # 解析风格文件 → JSON
│   │       └── play-style.rs   # 用真实风格文件播放和弦
│   ├── tests/             # 8 集成测试
│   ├── golden/            # 5 条基准 JSON (deterministic)
│   └── output/            # 生成的 WAV 文件
├── docs/
│   ├── architecture.md
│   └── phase1-report.md
└── README.md
```

## 快速开始

```bash
# 1. 下载一个 GM SoundFont (如 TimGM6mb.sf2, FluidR3_GM.sf2) 放到 jjazz-engine/ 目录
# 2. 构建运行

# 方式 1: 用 Yamaha 风格文件 (推荐, 8247 真实音符)
cargo run --bin play-style -- path/to/style.yjz Dm7 G7 Cmaj7

# 方式 2: 用手写 pattern (内置, 无需外部文件)
cargo run --bin jjazz-demo -- Dm7 G7 Cmaj7

# 解析任意风格文件为 JSON
cargo run --bin parse-style -- path/to/style.prs

# 导出 golden baseline
cargo run --bin jjazz-export -- Dm7 G7 Cmaj7

# 运行测试
cargo test
```

## 和弦输入格式

支持 JJazzLab 同款和弦语法:
- 三和弦: `C`, `Dm`, `Em`, `F`, `G`, `Am`, `Bdim`
- 七和弦: `Cmaj7`, `Dm7`, `G7`, `Bm7b5`
- 扩展: `Cmaj9`, `Dm11`, `G13`, `C7b9`
- 转位: `C/E`, `Dm7/A`
- `m` = minor, `M` = major, `dim` = diminished, `sus` = suspended

空格分隔: `Dm7 G7 Cmaj7`

## 许可证

LGPL v2.1 — 继承自原版 [JJazzLab](https://github.com/jjazzboss/JJazzLab)

## 原项目

https://github.com/jjazzboss/JJazzLab — 作者 Jerome Lelasseux, 35,000+ 用户
