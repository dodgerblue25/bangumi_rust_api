# bangumi_rust_api

一个基于 `reqwest` 的异步 Rust 客户端，用于访问 [Bangumi API](https://bangumi.github.io/api/)。项目使用 Rust 2024 edition，默认 API 地址为 `https://api.bgm.tv`。

## 功能

- 每日放送日历
- 条目、角色、人物和章节详情
- 条目、角色和人物搜索
- 用户详情和当前用户信息
- 条目收藏与取消收藏
- Bearer Token 认证
- 分页响应和统一错误类型

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
bgm_api = "0.1.0"
```

如果你在本地使用此项目，也可以直接依赖本地路径：

```toml
bgm_api = { path = "../bgm_api" }
```

## 快速开始

```rust
use bgm_api::{model::SearchRequest, Client};

#[tokio::main]
async fn main() -> Result<(), bgm_api::Error> {
    let client = Client::new();

    let subject = client.subject(1).await?;
    println!("subject = {:?}", subject.fields);

    let result = client
        .search_subjects(
            &SearchRequest {
                keyword: "葬送的芙莉莲".to_owned(),
                sort: Some("heat".to_owned()),
                filter: None,
            },
            Some(10),
            Some(0),
        )
        .await?;
    println!("found {} subjects", result.total);

    Ok(())
}
```

上面的示例需要在依赖中启用 Tokio：

```toml
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Access Token

需要登录权限的接口使用 Bangumi Access Token。可以在 [Access Token 页面](https://next.bgm.tv/demo/access-token)生成 Token，并通过 `with_token` 配置：

```rust
let client = Client::new().with_token("your-access-token");
let me = client.me().await?;
client.collect_subject(1).await?;
```

建议从环境变量读取 Token，不要将 Token 提交到 Git：

```rust
let client = Client::new().with_token(std::env::var("BANGUMI_ACCESS_TOKEN")?);
```

## 示例

运行仓库中的完整示例：

```bash
cargo run --example search
```

设置 `BANGUMI_ACCESS_TOKEN` 后，示例还会请求当前用户信息：

```bash
BANGUMI_ACCESS_TOKEN=your-token cargo run --example search
```

## 错误处理

所有客户端方法返回 `Result<T, bgm_api::Error>`。非 2xx 响应会返回 `Error::Api`，其中包含 HTTP 状态码和 API 返回的响应正文；网络错误和 URL 错误分别通过 `Error::Request` 与 `Error::Url` 表示。

## 说明

部分 Bangumi API schema 仍标记为实验性，且可能随服务端调整。复杂或变化频繁的对象目前保留在 `model::*::fields` 的 `serde_json::Value` 中，以便客户端兼容新增字段。

API 文档：[bangumi.github.io/api](https://bangumi.github.io/api/)
