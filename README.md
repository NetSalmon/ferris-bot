# 🦀 Ferris Bot

一个基于Rust编写的AI Agent Bot，支持工具调用和流式对话，提供Web API接口。

<img src="./resource/ferris-hello.gif" alt="this is Ferris" style="text-align: center">

## ✨ 功能特性

- 🤖 **AI对话** - 支持OpenAI兼容API的流式对话
- 🛠️ **工具调用** - 内置多种系统工具，支持自动工具调用和人工审批
- 🌐 **Web API** - 提供HTTP REST API接口
- ⚡ **异步架构** - 基于Tokio的高性能异步运行时
- 🔒 **安全控制** - 危险工具操作需要人工审批

## 🔧 内置工具

| 工具 | 描述 |
|------|------|
| `bash` | 执行Bash命令 |
| `ls` | 列出目录内容 |
| `cat` | 查看文件内容 |
| `tail` | 查看文件尾部内容 |
| `grep` | 文本搜索匹配 |
| `sed` | 流编辑器，文本替换/处理 |
| `get_weather` | 获取天气信息 |

## 📦 技术栈

- **语言**: Rust 2024 Edition
- **Web框架**: Axum
- **异步运行时**: Tokio
- **HTTP客户端**: Reqwest
- **序列化**: Serde + Serde JSON
- **CORS**: Tower HTTP

## 🚀 快速开始

### 环境要求

- Rust 1.80+
- Cargo

### 安装运行

1. 克隆仓库
```bash
git clone <your-repo-url>
cd ferris-bot
```

2. 创建环境变量文件
```bash
cp .env.example .env
```

3. 编辑 `.env` 文件，配置API信息：
```env
FERRIS_BOT_BASE_URL=https://api.openai.com/v1
FERRIS_BOT_API_KEY=your-api-key-here
FERRIS_BOT_MODEL=gpt-4
```

4. 运行项目
```bash
cargo run
```

服务将启动在 `http://0.0.0.0:11451`

## 📡 API接口

### 对话接口

#### 发送消息
```http
POST /agent
Content-Type: application/json

{
  "message": "你好，请帮我查看当前目录的文件"
}
```

#### 获取AI回复（SSE流）
```http
GET /agent
```

### 工具控制接口

#### 获取工具输出
```http
GET /agent/tool
```

#### 审批工具执行
```http
POST /agent/tool
Content-Type: application/json

{
  "approve": true
}
```

## 🏗️ 项目结构

```
ferris-bot/
├── src/
│   ├── main.rs          # 程序入口
│   ├── agent.rs         # AI Agent核心逻辑
│   ├── client/          # HTTP客户端（流式/批量）
│   ├── service/         # Web服务和路由处理
│   ├── service.rs       # 服务启动逻辑
│   ├── tools/           # 工具实现
│   │   ├── bash.rs      # Bash命令工具
│   │   ├── cat.rs       # 文件查看工具
│   │   ├── control.rs   # 工具控制管理
│   │   ├── get_weather.rs # 天气查询工具
│   │   ├── grep.rs      # 文本搜索工具
│   │   ├── ls.rs        # 目录列表工具
│   │   ├── sed.rs       # 流编辑器工具
│   │   └── tail.rs      # 文件尾部查看工具
│   ├── entities/        # 数据实体定义
│   └── error.rs         # 错误处理
├── Cargo.toml           # Rust依赖配置
└── .env                 # 环境变量（不提交到Git）
```

## ⚙️ 配置说明

| 环境变量 | 说明 | 必填 |
|----------|------|------|
| `FERRIS_BOT_BASE_URL` | AI API基础URL | ✅ |
| `FERRIS_BOT_API_KEY` | API密钥 | ✅ |
| `FERRIS_BOT_MODEL` | 模型名称 | ✅ |

## 🛡️ 安全说明

- 危险工具（如bash）默认需要人工审批
- 支持CORS跨域配置
- 建议使用HTTPS部署生产环境
- 请妥善保管API密钥，不要提交到代码仓库

## 📋 TODO

- [x] 自动添加工具
- [x] 从环境变量读取配置
- [x] 更换输出方式
- [x] 封装Agent结构
- [x] 添加Web服务器
- [x] 工具审批机制（通过Web界面）
- [ ] 添加AgentManager控制Agent生命周期
- [ ] 添加channel管理对话上下文

## 🤝 贡献

欢迎提交Issue和Pull Request！

## 📄 许可证

MIT License

---

🦀 Powered by Rust
