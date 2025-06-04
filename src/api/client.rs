use reqwest::{ Client };
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use serde_json::{ json, value::RawValue };
use anyhow::Context;

const ROOT_URL: &str = "https://api.github.com/graphql";

/// A quick Github GraphQL API client.
#[derive(Debug)]
pub struct Api {
    client: Client,
    token: String,
    user: String,
}

impl Api {
    pub async fn new(token: String) -> Result<Api, anyhow::Error> {
        let client = Client::new();
        let user = Api::fetch_username(&client, &token).await?;

        Ok(Api { token, client, user })
    }

    /// The username can not be retrieved via the GraphQL API, so we make a REST call instead.
    async fn fetch_username(client: &Client, token: &str) -> Result<String, anyhow::Error> {
        let res = client
            .get("https://api.github.com/user")
            .bearer_auth(&token)
            .header("User-Agent", "jsdw-github-summarizer")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .with_context(|| "Failed to send request to get user")?;

        if res.status().is_success() {
            #[derive(Deserialize)]
            struct UserResponse {
                login: String
            }
            let res: UserResponse = res.json().await
                .with_context(|| "Failed to decode user response")?;
            Ok(res.login)
        } else {
            Err(anyhow::anyhow!("Failed to get user: {}", res.status()))
        }
    }

    /// The API token user.
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Send a GraphQL query with variables.
    pub async fn query<Res: DeserializeOwned>(&self, query: &str, variables: Variables) -> Result<Res, anyhow::Error> {
        let err_context = |msg: &str| {
            // pull out the name given to the query if possible:
            let first_line = query
                .trim_start()
                .lines()
                .next()
                .unwrap_or("<empty query>")
                .trim_end()
                .trim_end_matches('{')
                .trim_end();
            format!("{first_line}: {msg}")
        };

        let res = self.client
            .post(ROOT_URL)
            .bearer_auth(&self.token)
            .header("User-Agent", "jsdw-github-summarizer")
            .json(&json!({
                "query": query,
                "variables": variables.build()
            }))
            .send()
            .await
            .with_context(|| err_context("Failed to send request"))?;

        let status = res.status();
        if status.is_success() {
            // Broken down the steps to allow better debugging in case of issue:
            let text = res.text().await
                .with_context(|| {
                    err_context("Failed to obtain string response")
                })?;

            #[derive(Deserialize)]
            struct QueryData<Res> {
                data: Res   
            }

            // Trying to decode as QueryData first, rather than trying to decode as an enum
            // which can be data or errors, makes for much better error messages on decode fail.
            let body: QueryData<Res> = serde_json::from_str(&text).map_err(|e| {
                #[derive(Deserialize)]
                struct QueryErrors {
                    errors: Vec<QueryError>
                }
                if let Ok(errors) = serde_json::from_str::<QueryErrors>(&text) {
                    ApiError::QueryErrors(errors.errors)
                } else {
                    eprintln!("{text}");
                    ApiError::DecodeError(anyhow::anyhow!("Failed to decode response: {}", e))
                }
            })?;

            Ok(body.data)
        } else {
            let body = res.text().await?;
            Err(ApiError::BadResponse(status.as_u16(), body))
                .with_context(|| err_context("Bad response making request"))
        }
    }
}

/// This represents variables you can pass to a GraphQL query.
pub struct Variables {
    json: Vec<u8>
}

impl Variables {
    pub fn new() -> Self {
        Variables { json: Vec::new() }
    }
    pub fn push<T: Serialize>(&mut self, name: &str, value: T) {
        if self.json.is_empty() {
            self.json.push(b'{');
        } else {
            self.json.push(b',');
        }

        serde_json::to_writer(&mut self.json, name).unwrap();
        self.json.push(b':');
        serde_json::to_writer(&mut self.json, &value).unwrap();
    }
    fn build(mut self) -> Option<Box<RawValue>> {
        if self.json.is_empty() {
            return None;
        }

        self.json.push(b'}');

        // Everything we push is valid UFT8 so this is fine.
        let json = unsafe { String::from_utf8_unchecked(self.json) };

        Some(RawValue::from_string(json).unwrap())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    RequestError(#[from] reqwest::Error),
    #[error("{0} response: {1}")]
    BadResponse(u16, String),
    #[error("Errors with query: {0:?}")]
    QueryErrors(Vec<QueryError>),
    #[error("{0}")]
    DecodeError(#[from] anyhow::Error),
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct QueryError {
    path: Option<Vec<String>>,
    message: String
}

#[macro_export]
macro_rules! variables {
    ($($key:literal : $val:expr), *) => {{
        // May be unused if empty; no params.
        #[allow(unused_mut)]
        let mut params = $crate::api::client::Variables::new();
        $(
            params.push($key, $val);
        )*
        params
    }}
}