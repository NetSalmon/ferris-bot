# Ferris Bot API 文档

## 目录

- [概述](#概述)
- [基础信息](#基础信息)
- [统一响应格式](#统一响应格式)
- [端点概览](#端点概览)
- [端点详情](#端点详情)
- [数据类型定义](#数据类型定义)
- [使用示例](#使用示例)
- [错误处理](#错误处理)
- [工具系统](#工具系统)
- [Server-Sent Events (SSE) 详细说明](#server-sent-events-sse-详细说明)
- [注意事项](#注意事项)

## 概述

Ferris Bot 是一个基于 Rust 和 Axum 框架的 AI 代理服务。它提供了创建和管理 AI 代理的 RESTful API，支持工具调用、流式响应和消息管理。

## 基础信息

- **基础 URL**: `http://127.0.0.1:11451`
- **响应格式**: JSON
- **错误处理**: 所有端点返回统一的 `ApiResponse<T>` 格式

## 统一响应格式

所有 API 端点都返回以下格式的 JSON 响应：

```json
{
  "ok": boolean,
  "result": T
}
```

- `ok`: 操作是否成功
- `result`: 实际结果数据，类型取决于端点

## 端点概览

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/all` | 获取所有活跃代理的 UUID 列表 |
| POST | `/create` | 创建新的 AI 代理 |
| POST | `/agent/{uuid}` | 向指定代理发送输入消息 |
| GET | `/agent/{uuid}` | 获取代理的流式输出 (SSE) |
| DELETE | `/agent/{uuid}` | 删除指定代理 |
| POST | `/agent/{uuid}/tool` | 批准或驳回工具调用请求 |
| GET | `/agent/{uuid}/messages` | 获取代理的完整消息历史 |
| GET | `/tools` | 获取可用工具列表 |

## 端点详情

### 1. 获取所有代理列表

**GET** `/all`

**响应**:
```json
{
  "ok": true,
  "result": ["uuid1", "uuid2", ...]
}
```

### 2. 创建新代理

**POST** `/create`

**请求体** (`AgentSettings`):
```json
{
  "system_prompt": "可选的系统提示",
  "temperature": 0.7,
  "tools": ["bash", "grep", "cat"],
  "tool_choice": "auto",
  "stream": true,
  "thinking": false,
  "model": "claude-3-5-sonnet-20241022",
  "max_tokens": 4096
}
```

**字段说明**:
- `system_prompt`: 可选，系统提示信息
- `temperature`: 可选，生成温度 (0.0-2.0)
- `tools`: 可选，启用的工具名称列表
- `tool_choice`: 可选，工具调用策略 ("auto", "none", 或工具名)
- `stream`: 可选，是否启用流式响应
- `thinking`: 可选，是否启用思考模式
- `model`: **必需**，使用的模型名称
- `max_tokens`: 可选，最大生成 token 数

**响应**:
```json
{
  "ok": true,
  "result": "新代理的 UUID 字符串"
}
```

### 3. 向代理发送输入

**POST** `/agent/{uuid}`

**路径参数**:
- `uuid`: 代理的 UUID

**请求体**: 纯文本字符串

**响应**:
```json
{
  "ok": true,
  "result": "Message sent to agent"  # 或 "Agent is not listening"
}
```

### 4. 获取代理流式输出 (SSE)

**GET** `/agent/{uuid}`

**路径参数**:
- `uuid`: 代理的 UUID

**响应**: Server-Sent Events (SSE) 流

**事件格式**:
```
event: data
data: {"type": "content_chunk", "content": "输出内容"}
```

**Chunk 类型** (`Chunk` 枚举):
- `content_chunk`: 文本内容块
- `reason_chunk`: 思考内容块
- `tool_call`: 工具调用请求
- `tool_output`: 工具执行结果
- `block`: 阻塞提示 (等待批准)
- `messages`: 完整消息历史
- `event_end`: 流结束标记

**完整 Chunk 结构**:
```json
// content_chunk
{
  "type": "content_chunk",
  "content": "文本内容"
}

// reason_chunk
{
  "type": "reason_chunk",
  "content": "思考内容"
}

// tool_call
{
  "type": "tool_call",
  "name": "工具名称",
  "arguments": "JSON 参数字符串"
}

// tool_output
{
  "type": "tool_output",
  "stdout": "标准输出",
  "stderr": "标准错误",
  "status": 0
}

// block
{
  "type": "block",
  "reason": "blocked",
  "content": "工具调用需要批准，请使用 /agent/{uuid}/tool 端点批准或驳回"
}

// messages
{
  "type": "messages",
  "messages": [...]
}

// event_end
{
  "type": "event_end"
}
```

### 5. 删除代理

**DELETE** `/agent/{uuid}`

**路径参数**:
- `uuid`: 代理的 UUID

**响应**:
```json
{
  "ok": true,
  "result": null
}
```

### 6. 工具调用控制

**POST** `/agent/{uuid}/tool`

**路径参数**:
- `uuid`: 代理的 UUID

**请求体**: `true` (批准) 或 `false` (驳回)

```json
true
```

**响应**:
```json
{
  "ok": true,
  "result": null
}
```

### 7. 获取代理消息历史

**GET** `/agent/{uuid}/messages`

**路径参数**:
- `uuid`: 代理的 UUID

**响应**:
```json
{
  "ok": true,
  "result": {
    "type": "messages",
    "messages": [...]
  }
}
```

### 8. 获取可用工具列表

**GET** `/tools`

**响应**:
```json
{
  "ok": true,
  "result": "[\"bash\", \"cat\", \"grep\", \"ls\", \"sed\", \"tail\"]"
}
```

## 数据类型定义

### AgentSettings
```rust
pub struct AgentSettings {
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<String>>,
    pub tool_choice: Option<String>,
    pub stream: Option<bool>,
    pub thinking: Option<bool>,
    pub model: String,
    pub max_tokens: Option<u32>,
}
```

### ApiResponse
```rust
pub struct ApiResponse<T> {
    pub ok: bool,
    pub result: T,
}
```

### Chunk (SSE 事件数据)
```rust
pub enum Chunk {
    Block {
        reason: String,
        content: String,
    },
    ToolCall {
        name: String,
        arguments: String,
    },
    Messages {
        messages: Vec<MarkedMessage>,
    },
    ToolOutput {
        stdout: String,
        stderr: String,
        status: Option<i32>,
    },
    ReasonChunk {
        content: String,
    },
    ContentChunk {
        content: String,
    },
    EventEnd,
}
```

### Message (AI 消息)
```rust
pub enum Message {
    System {
        content: String,
        name: Option<String>,
    },
    User {
        content: String,
        name: Option<String>,
    },
    Assistant {
        content: Option<String>,
        reasoning_content: Option<String>,
        tool_calls: Option<Vec<ToolCall>>,
    },
    Tool {
        content: String,
        tool_call_id: String,
    },
}
```

### MarkedMessage
```rust
pub struct MarkedMessage {
    id: usize,          // 内部使用，不序列化
    message: Message,   // 实际消息内容
}
```

### 其他数据类型

#### Request (AI 请求)
```rust
pub struct Request {
    pub model: String,
    pub messages: Vec<MarkedMessage>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<String>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    enable_thinking: Option<bool>,
}
```

#### Response (AI 响应)
```rust
pub struct Response {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}
```

#### Tool (工具定义)
```rust
pub struct Tool {
    pub r#type: String,
    pub function: FunctionDetail,
}
```

#### FunctionDetail (函数详情)
```rust
pub struct FunctionDetail {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
```

#### ToolCall (工具调用)
```rust
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: CallFunction,
}
```

#### CallFunction (调用函数)
```rust
pub struct CallFunction {
    pub name: String,
    pub arguments: String,
}
```

#### Choice (响应选项)
```rust
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: FinishReason,
}
```

#### Usage (使用统计)
```rust
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

#### FinishReason (结束原因)
```rust
pub enum FinishReason {
    Stop,
    ToolCalls,
    Length,
    ContentFilter,
    Null,
}
```

## 使用示例

### 创建代理
```bash
curl -X POST http://127.0.0.1:11451/create \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-3-5-sonnet-20241022",
    "system_prompt": "你是一个有用的助手",
    "temperature": 0.7,
    "tools": ["bash", "cat"],
    "stream": true
  }'
```

### 发送消息
```bash
curl -X POST http://127.0.0.1:11451/agent/123e4567-e89b-12d3-a456-426614174000 \
  -H "Content-Type: text/plain" \
  -d "列出当前目录的文件"
```

### 监听流式响应
```bash
curl -N http://127.0.0.1:11451/agent/123e4567-e89b-12d3-a456-426614174000
```

### 批准工具调用
```bash
curl -X POST http://127.0.0.1:11451/agent/123e4567-e89b-12d3-a456-426614174000/tool \
  -H "Content-Type: application/json" \
  -d "true"
```

## 错误处理

当发生错误时，响应中的 `ok` 字段为 `false`，`result` 字段包含错误信息字符串。所有错误都返回统一的 `ApiResponse` 格式：

```json
{
  "ok": false,
  "result": "错误描述信息"
}
```

### HTTP 状态码映射

| 状态码 | 错误类型 | 描述 |
|--------|----------|------|
| 400 Bad Request | `HeaderValueError`, `UrlParseError`, `JSONError`, `Utf8Error` | 请求头、URL、JSON 或编码错误 |
| 403 Forbidden | `NoApprovementActionError` | 工具调用未获得批准 |
| 404 Not Found | `NotFound`, `NoSuchToolError` | 代理或工具不存在 |
| 500 Internal Server Error | 其他所有错误 | 服务器内部错误 |

### 常见错误信息

1. **代理不存在**
   ```json
   {"ok": false, "result": "Not found: {uuid}"}
   ```

2. **工具不存在**
   ```json
   {"ok": false, "result": "Tool not found: {tool_name}"}
   ```

3. **工具调用未批准**
   ```json
   {"ok": false, "result": "Action not approved: {action}"}
   ```

4. **请求参数无效**
   ```json
   {"ok": false, "result": "JSON error: {error_details}"}
   ```

5. **内部错误**
   ```json
   {"ok": false, "result": "Internal error: {error_details}"}
   ```

## 工具系统

当前支持的工具:
- `bash`: 执行 bash 命令 (需要批准)
- `cat`: 查看文件内容
- `grep`: 搜索文件内容
- `ls`: 列出目录内容
- `sed`: 文本流编辑器
- `tail`: 查看文件尾部

**注意**: `/tools` 端点只返回工具名称列表。工具的具体描述、参数和用法信息需要通过 AI 代理查询（例如询问 "describe the bash tool"）。

所有工具调用都需要通过 `/agent/{uuid}/tool` 端点进行批准。

## Server-Sent Events (SSE) 详细说明

### 事件格式

SSE 流发送 `data` 类型的事件，每个事件包含一个 JSON 字符串化的 `Chunk` 对象。

原始 SSE 格式:
```
event: data
data: {"type": "content_chunk", "content": "Hello"}
```

### Chunk 类型详解

1. **content_chunk** - 普通文本输出
   ```json
   {"type": "content_chunk", "content": "文本内容"}
   ```

2. **reason_chunk** - 思考内容 (当启用 thinking 模式时)
   ```json
   {"type": "reason_chunk", "content": "思考内容"}
   ```

3. **tool_call** - 工具调用请求
   ```json
   {
     "type": "tool_call",
     "name": "bash",
     "arguments": "{\"args\":[\"ls\",\"-la\"]}"
   }
   ```
   **注意**: `arguments` 字段是 JSON 字符串，需要解析两次。

4. **tool_output** - 工具执行结果
   ```json
   {
     "type": "tool_output",
     "stdout": "输出内容",
     "stderr": "错误信息",
     "status": 0
   }
   ```

5. **block** - 阻塞提示 (等待工具调用批准)
   ```json
   {
     "type": "block",
     "reason": "blocked",
     "content": "工具调用需要批准，请使用 /agent/{uuid}/tool 端点批准或驳回"
   }
   ```
   收到此事件后，必须调用 `/agent/{uuid}/tool` 端点进行批准或驳回。

6. **messages** - 完整消息历史
   ```json
   {
     "type": "messages",
     "messages": [...]
   }
   ```

7. **event_end** - 流结束标记
   ```json
   {"type": "event_end"}
   ```

### 客户端处理流程

1. 建立 SSE 连接: `GET /agent/{uuid}`
2. 监听 `data` 事件
3. 解析事件数据为 JSON
4. 根据 `type` 字段处理不同的 Chunk 类型
5. 遇到 `block` 类型时，暂停并等待用户批准/驳回
6. 收到 `event_end` 时，连接可能保持打开等待更多数据

### JavaScript 示例

```javascript
const eventSource = new EventSource('http://127.0.0.1:11451/agent/123e4567-e89b-12d3-a456-426614174000');

eventSource.addEventListener('data', (event) => {
  const chunk = JSON.parse(event.data);

  switch (chunk.type) {
    case 'content_chunk':
      console.log('内容:', chunk.content);
      break;
    case 'tool_call':
      console.log('工具调用:', chunk.name);
      console.log('参数:', JSON.parse(chunk.arguments));
      break;
    case 'block':
      console.log('需要批准工具调用');
      // 调用批准端点
      fetch(`/agent/${uuid}/tool`, {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: 'true'  // 或 'false' 驳回
      });
      break;
    case 'event_end':
      console.log('流结束');
      break;
  }
});

eventSource.onerror = (error) => {
  console.error('SSE 错误:', error);
  eventSource.close();
};
```

## 注意事项

1. 工具调用默认需要人工批准
2. SSE 连接需要客户端支持 Server-Sent Events
3. 响应中的 UUID 是字符串格式