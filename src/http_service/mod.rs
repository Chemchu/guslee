use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Error;

#[derive(Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct ResponseData<T> {
    pub content: T,
}

impl<T: Clone> Clone for ResponseData<T> {
    fn clone(&self) -> Self {
        ResponseData {
            content: self.content.clone(),
        }
    }
}

pub struct HttpService {
    api_key: String,
    endpoint_url: String,
    http_caller: reqwest::Client,
}

impl HttpService {
    pub fn new(api_key: String, endpoint_url: String) -> Self {
        Self {
            api_key,
            endpoint_url,
            http_caller: reqwest::Client::new(),
        }
    }
    pub async fn get<T: DeserializeOwned + std::clone::Clone>(
        &self,
        table: &String,
        query: &String,
    ) -> ResponseData<T> {
        let request = self
            .http_caller
            .get(format!(
                "{}/rest/v1/{}?{}",
                &self.endpoint_url, table, query
            ))
            .header("apiKey", &self.api_key)
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .header("Content-Type", "application/json")
            .build();

        match request {
            Err(e) => panic!("{}", e),
            Ok(r) => {
                let response = self.http_caller.execute(r).await;
                if let Err(e) = response {
                    panic!("{}", e)
                }
                let text = response.unwrap().text().await.unwrap();
                let data: Result<Vec<T>, Error> = serde_json::from_str(&text);
                match data {
                    Err(e) => panic!("{}", e),
                    Ok(d) => match d.first() {
                        // TODO: replace this panic with an empty generic not found
                        None => panic!("Array vacio"),
                        Some(c) => ResponseData {
                            content: c.to_owned(),
                        },
                    },
                }
            }
        }
    }
}
