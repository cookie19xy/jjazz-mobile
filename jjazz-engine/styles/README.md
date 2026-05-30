# Style Files

将 `.prs`, `.sty`, `.yjz` 风格文件放此目录。

## 内置风格

- `psBase.yjz` — JJSwing 贝斯/吉他基础风格 (149KB, 来自 JJazzLab)
  - P1: 密集乐句 (32拍, 14通道)
  - P2: 中等密度
  - P3: 稀疏
  - P4: 尾奏

## 如何获取更多风格

JJazzLab 支持 150+ Yamaha 键盘风格文件 (`.prs`, `.sty`)。这些文件是标准 MIDI 格式，可直接被本引擎读取。

免费 Yamaha 风格可从以下获取：
- PSR Tutorial (https://psrtutorial.com)
- JJazzLab 自带的 rhythm database

将下载的 `.prs` 或 `.sty` 文件放入此目录即可使用。

## 用法

```bash
# 播放基础风格
cargo run --bin play-style -- styles/psBase.yjz -b 2 Dm7 G7 Cmaj7

# 选择密度 (-p 0/1/2)
cargo run --bin play-style -- styles/psBase.yjz -b 2 -p 1 Dm7 G7

# 使用自己的风格文件
cargo run --bin play-style -- styles/BossaNova2.S469.prs -b 2 Cmaj7 Am7 Dm7 G7
```
