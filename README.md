# 🦀 Ferris Bot

一个基于Rust编写的AI Agent Bot，支持工具调用和流式对话，提供Web API接口。

<img src="./resource/ferris-hello.gif" alt="this is Ferris" style="text-align: center">

## ✨ 功能特性

- 🤖 **AI对话** - 支持OpenAI兼容API的流式对话
- 🛠️ **工具调用** - 内置多种系统工具，支持自动工具调用和人工审批
- 🌐 **Web API** - 提供HTTP REST API接口
- ⚡ **异步架构** - 基于Tokio的高性能异步运行时
- 🔒 **安全控制** - 危险工具操作需要人工审批
- 🎛️ **多Agent管理** - AgentManager支持创建、销毁和管理多个Agent实例
- 💭 **思维链** - 支持AI思考过程（reasoning）的流式输出

## 🔧 内置工具

| 工具 | 描述           |
|------|--------------|
| `bash` | 执行Bash命令     |
| `ls` | 列出目录内容       |
| `cat` | 查看文件内容       |
| `tail` | 查看文件尾部内容     |
| `grep` | 文本搜索匹配       |
| `sed` | 流编辑器，文本替换/处理 |
| `get_weather` | 模拟获取天气信息     |

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
git clone https://github.com/NetSalmon/ferris-bot.git
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

服务将启动在 `http://127.0.0.1:11451`

## 💡 多Agent架构

Ferris Bot 使用 `AgentManager` 管理多个独立的Agent实例，每个Agent拥有自己的：
- AI对话状态
- 工具调用控制
- 消息历史管理

### 工作流程

1. **创建Agent**: 调用 `GET /create` 获取新Agent的UUID
2. **发送消息**: 通过 `POST /agent/{uuid}` 向特定Agent发送消息
3. **监听响应**: 使用 `GET /agent/{uuid}/content` (SSE) 获取AI回复和思考过程
4. **管理工具**: 通过 `GET /agent/{uuid}/tool` 监听工具执行，使用 `POST /agent/{uuid}/tool` 进行审批
5. **清理资源**: 调用 `DELETE /agent/{uuid}` 删除Agent实例

### Agent管理接口

#### 创建Agent实例
```http
GET /create
```
返回创建的Agent的UUID

#### 获取所有Agent列表
```http
GET /all
```
返回所有活跃Agent的UUID列表

#### 移除Agent实例
```http
DELETE /agent/{uuid}
```

### 对话接口

#### 发送消息到Agent
```http
POST /agent/{uuid}
Content-Type: text/plain

你好，请帮我查看当前目录的文件
```

#### 获取AI回复（SSE流）
```http
GET /agent/{uuid}
```
返回SSE流，包含两类事件：
- `content`: AI生成的内容
- `reason`: AI的思考过程

### 工具控制接口

#### 获取工具输出
```http
GET /agent/{uuid}/tool
```
返回SSE流，包含工具执行信息（调用中/输出结果）

#### 审批/驳回工具执行
```http
POST /agent/{uuid}/tool
Content-Type: application/json

true
```
`true` 表示批准，`false` 表示驳回

## 🏗️ 项目结构

```
ferris-bot/
├── src/
│   ├── main.rs              # 程序入口
│   ├── agent.rs             # AI Agent核心逻辑
│   ├── agent_manager.rs     # Agent生命周期管理
│   ├── client.rs            # HTTP客户端配置
│   ├── client/
│   │   ├── batch.rs         # 批量请求处理
│   │   └── stream.rs        # 流式请求处理
│   ├── service.rs           # 服务启动逻辑
│   ├── service/             # Web服务和路由处理
│   │   └── handlers.rs      # HTTP请求处理器
│   ├── tools/               # 工具实现
│   │   ├── mod.rs           # 工具模块管理
│   │   ├── bash.rs          # Bash命令工具
│   │   ├── cat.rs           # 文件查看工具
│   │   ├── control.rs       # 工具控制管理
│   │   ├── get_weather.rs   # 天气查询工具
│   │   ├── grep.rs          # 文本搜索工具
│   │   ├── ls.rs            # 目录列表工具
│   │   ├── sed.rs           # 流编辑器工具
│   │   └── tail.rs          # 文件尾部查看工具
│   ├── entities/            # 数据实体定义
│   │   ├── mod.rs           # 实体模块管理
│   │   ├── impls.rs         # 实体实现
│   │   ├── stream.rs        # 流式数据实体
│   │   └── runtime.rs       # 运行时配置
│   ├── error.rs             # 错误处理
│   └── lib.rs               # 库模块导出
├── Cargo.toml               # Rust依赖配置
├── Cargo.lock               # 依赖锁文件
└── .env                     # 环境变量（不提交到Git）
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
- [x] 通过broadcast channel实现工具审批机制
- [x] 添加AgentManager controllr Agent生命周期
- [ ] 添加channel管理对话消息上下文
- [ ] 支持自定义工具插件

## 🤝 贡献

欢迎提交Issue和Pull Request！

## 📄 许可证

GPL-3.0 License

---

🦀 Powered by Rust
