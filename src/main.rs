use crate::entities::runtime::Env;
use crate::error::AppError;
use dotenvy::dotenv;

mod agent;
pub mod agent_manager;
mod client;
mod entities;
mod error;
mod service;
mod tools;

/// TODO LIST
///
/// - [x] 自动添加工具
/// - [x] 从环境变量~~或配置文件~~读取 `base url` 与 `api key`
/// - [x] 更换输出输出方式
/// - [x] 封装 `Agent` struct
/// - [x] 添加 Web 服务器
/// - ~~添加 tui~~ 放弃
/// - ~~bash tool 使用 `overlayfs` （添加合并功能保证事务性） 和命令黑名单~~ 已改为审批模式
/// - [x] 工具请求走`tokio::broadcast::channel`，添加批准或驳回（可能需要添加一个中间层隔离+控制）。最后用 Web Server 转发
/// - [x] 添加 `AgentManager` struct 控制Agent创建、关闭或重启
/// - [ ] 添加 `channel` 管理 `request` 里的 `messages` ，控制上下文

/// LOCAL TESTING

pub static EXIT: &str = "/exit";

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();
    let base_url = std::env::var("FERRIS_BOT_BASE_URL")?;
    let api_key = std::env::var("FERRIS_BOT_API_KEY")?;
    let model = std::env::var("FERRIS_BOT_MODEL")?;

    let env = Env {
        base_url: base_url.parse()?,
        api_key,
        model,
    };

    service::run(env).await?;

    Ok(())
}
