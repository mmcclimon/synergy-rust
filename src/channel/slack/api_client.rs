use std::collections::HashMap;

use reqwest::{
    blocking::Client,
    header::{self, HeaderMap, HeaderValue},
};

use serde::Deserialize;

pub struct ApiClient {
    // token: String,
    http: Client,
}

pub fn new(token: String) -> ApiClient {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let http = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    ApiClient { http }
}

fn url_for(method: &str) -> String {
    format!("https://slack.com/api/{}", method)
}

impl ApiClient {
    pub fn load_users(&self) -> Option<HashMap<String, String>> {
        let res = self.http.get(&url_for("users.list")).send();

        #[derive(Debug, Deserialize)]
        struct Member {
            id: String,
            name: String,
        }

        #[derive(Debug, Deserialize)]
        struct UserResponse {
            ok: bool,
            members: Vec<Member>,
        }

        let got: UserResponse = res.unwrap().json().unwrap();

        if !got.ok {
            return None;
        }

        let mut hash = HashMap::new();

        for mem in got.members {
            hash.insert(mem.id, mem.name);
        }

        info!("loaded slack users");
        Some(hash)
    }
}
