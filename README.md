# ask

ask 是一个基于AI的命令行工具，对于简单的问题，使用它比使用浏览器更方便。

## 安装

从Releases页面下载最新的二进制文件，解压后将其放置在PATH中。或者使用 Cargo 安装：

```bash
cargo install --git https://github.com/xiyaowong/ask
```

## 使用

帮助

- 查看帮助 `ask` `ask help` `ask -h` `ask --help`

设置

- 帮助 `ask config --help`
- 请求超时时间 `ask config timeout {10 seconds}`
- 模型供应商 `ask config provider {deepseek}`
- 模型 `ask config model {deepseek-chat}`
- 显示配置 `ask config show`

使用环境变量提供 API Key

- DeepSeek - `export ASK_DEEPSEEK_KEY={your key}`
- Grok - `export ASK_GROK_KEY={your key}`

预设

- 帮助 `ask preset --help`
- 添加 `ask preset set {name} {prompt}`
- 移除 `ask preset remove {name}`
- 列出 `ask preset list`

使用

- 直接问 `ask {question}`
- 使用预设 `ask {preset} {question}`

---

# ask

ask is a command-line tool powered by AI, designed for quick and convenient answers to simple questions, making it easier than using a web browser.

## Installation

Download the latest binary from the Releases page, extract it, and place it in your PATH. Alternatively, you can install it using Cargo:

```bash
cargo install --git https://github.com/xiyaowong/ask
```
## Usage

Help

- View help `ask` `ask help` `ask -h` `ask --help`

Configuration

- Help `ask config --help`
- Request timeout `ask config timeout {10 seconds}`
- Model provider `ask config provider {deepseek}`
- Model `ask config model {deepseek-chat}`
- Show configuration `ask config show`

Environment Variables for API Keys

- DeepSeek - `export ASK_DEEPSEEK_KEY={your key}`
- Grok3 - `export ASK_GROK_KEY={your key}`

Presets

- Help `ask preset --help`
- Set/Add preset `ask preset set {name} {prompt}`
- Remove preset `ask preset remove {name}`
- List presets `ask preset list`

Usage

- Ask directly `ask {question}`
- Use preset `ask {preset} {question}`

---

# License

MIT
