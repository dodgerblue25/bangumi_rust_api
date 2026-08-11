//! Async client for the public [Bangumi API](https://bangumi.github.io/api/).

use reqwest::{Client as HttpClient, Method, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use thiserror::Error;
use url::Url;

pub mod model {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Page<T> {
        pub data: Vec<T>,
        pub total: u64,
        #[serde(default)]
        pub limit: u32,
        #[serde(default)]
        pub offset: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Weekday {
        pub en: Option<String>,
        pub cn: Option<String>,
        pub ja: Option<String>,
        pub id: Option<u8>,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CalendarItem {
        pub weekday: Weekday,
        pub items: Vec<SubjectSmall>,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SubjectSmall {
        pub id: u64,
        pub name: String,
        pub name_cn: String,
        pub url: Option<String>,
        pub images: Option<Images>,
        pub collection: Option<CollectionSummary>,
        pub eps: Option<u32>,
        pub air_date: Option<String>,
        pub air_weekday: Option<Weekday>,
    }
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct Images {
        pub large: Option<String>,
        pub common: Option<String>,
        pub medium: Option<String>,
        pub grid: Option<String>,
        pub small: Option<String>,
    }
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct CollectionSummary {
        pub wish: Option<u64>,
        pub collect: Option<u64>,
        pub doing: Option<u64>,
        pub on_hold: Option<u64>,
        pub dropped: Option<u64>,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Subject {
        #[serde(flatten)]
        pub fields: Value,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Character {
        #[serde(flatten)]
        pub fields: Value,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Person {
        #[serde(flatten)]
        pub fields: Value,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Episode {
        #[serde(flatten)]
        pub fields: Value,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct User {
        #[serde(flatten)]
        pub fields: Value,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Collection {
        #[serde(flatten)]
        pub fields: Value,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchRequest {
        pub keyword: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sort: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filter: Option<Value>,
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("Bangumi API returned {status}: {body}")]
    Api { status: StatusCode, body: String },
}

#[derive(Clone)]
pub struct Client {
    http: HttpClient,
    base_url: Url,
    token: Option<String>,
}

impl Client {
    pub fn new() -> Self {
        Self::with_base_url("https://api.bgm.tv").expect("default URL is valid")
    }
    pub fn with_base_url(base_url: &str) -> Result<Self, Error> {
        Ok(Self {
            http: HttpClient::builder()
                .user_agent("bgm-api-rust-client/0.1")
                .build()?,
            base_url: Url::parse(base_url)?,
            token: None,
        })
    }
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }
    async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        query: Option<&[(&str, String)]>,
        body: Option<&impl Serialize>,
    ) -> Result<T, Error> {
        let url = self.base_url.join(path)?;
        let mut req = self.http.request(method, url);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        if let Some(query) = query {
            req = req.query(query);
        }
        if let Some(body) = body {
            req = req.json(body);
        }
        let response = req.send().await?;
        if !response.status().is_success() {
            let status = response.status();
            return Err(Error::Api {
                status,
                body: response.text().await.unwrap_or_default(),
            });
        }
        Ok(response.json().await?)
    }
    async fn empty(&self, method: Method, path: String) -> Result<(), Error> {
        let url = self.base_url.join(&path)?;
        let mut req = self.http.request(method, url);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        let response = req.send().await?;
        if !response.status().is_success() {
            let status = response.status();
            return Err(Error::Api {
                status,
                body: response.text().await.unwrap_or_default(),
            });
        }
        Ok(())
    }
    pub async fn calendar(&self) -> Result<Vec<model::CalendarItem>, Error> {
        self.request(Method::GET, "/calendar", None, None::<&()>)
            .await
    }
    pub async fn subject(&self, id: u64) -> Result<model::Subject, Error> {
        self.request(
            Method::GET,
            &format!("/v0/subjects/{id}"),
            None,
            None::<&()>,
        )
        .await
    }
    pub async fn search_subjects(
        &self,
        request: &model::SearchRequest,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<model::Page<model::Subject>, Error> {
        self.search("/v0/search/subjects", request, limit, offset)
            .await
    }
    pub async fn search_characters(
        &self,
        request: &model::SearchRequest,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<model::Page<model::Character>, Error> {
        self.search("/v0/search/characters", request, limit, offset)
            .await
    }
    pub async fn search_persons(
        &self,
        request: &model::SearchRequest,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<model::Page<model::Person>, Error> {
        self.search("/v0/search/persons", request, limit, offset)
            .await
    }
    async fn search<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &model::SearchRequest,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<model::Page<T>, Error> {
        let q = vec![
            limit.map(|v| ("limit", v.to_string())),
            offset.map(|v| ("offset", v.to_string())),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        self.request(Method::POST, path, Some(&q), Some(body)).await
    }
    pub async fn episode(&self, id: u64) -> Result<model::Episode, Error> {
        self.request(
            Method::GET,
            &format!("/v0/episodes/{id}"),
            None,
            None::<&()>,
        )
        .await
    }
    pub async fn person(&self, id: u64) -> Result<model::Person, Error> {
        self.request(Method::GET, &format!("/v0/persons/{id}"), None, None::<&()>)
            .await
    }
    pub async fn character(&self, id: u64) -> Result<model::Character, Error> {
        self.request(
            Method::GET,
            &format!("/v0/characters/{id}"),
            None,
            None::<&()>,
        )
        .await
    }
    pub async fn user(&self, name: &str) -> Result<model::User, Error> {
        self.request(Method::GET, &format!("/v0/users/{name}"), None, None::<&()>)
            .await
    }
    pub async fn me(&self) -> Result<model::User, Error> {
        self.request(Method::GET, "/v0/me", None, None::<&()>).await
    }
    pub async fn collect_subject(&self, id: u64) -> Result<(), Error> {
        self.empty(Method::POST, format!("/v0/subjects/{id}/collect"))
            .await
    }
    pub async fn uncollect_subject(&self, id: u64) -> Result<(), Error> {
        self.empty(Method::DELETE, format!("/v0/subjects/{id}/collect"))
            .await
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::model::SearchRequest;

    #[test]
    fn search_request_omits_optional_fields() {
        let request = SearchRequest {
            keyword: "葬送的芙莉莲".into(),
            sort: None,
            filter: None,
        };
        assert_eq!(
            serde_json::to_value(request).unwrap(),
            serde_json::json!({"keyword": "葬送的芙莉莲"})
        );
    }
}
