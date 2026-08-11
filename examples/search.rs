use bgm_api::{Client, model::SearchRequest};

#[tokio::main]
async fn main() -> Result<(), bgm_api::Error> {
    let client = match std::env::var("BANGUMI_ACCESS_TOKEN") {
        Ok(token) => Client::new().with_token(token),
        Err(_) => Client::new(),
    };

    let result = client
        .search_subjects(
            &SearchRequest {
                keyword: "葬送的芙莉莲".to_owned(),
                sort: Some("heat".to_owned()),
                filter: None,
            },
            Some(5),
            Some(0),
        )
        .await?;

    println!("total: {}", result.total);
    for subject in result.data {
        println!("{}: {}", subject.fields["id"], subject.fields["name_cn"]);
    }

    if std::env::var("BANGUMI_ACCESS_TOKEN").is_ok() {
        let me = client.me().await?;
        println!("current user: {}", me.fields["username"]);
    }

    Ok(())
}
