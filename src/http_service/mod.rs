use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct RequestData<T> {
    pub content: T,
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
    pub async fn get<T: DeserializeOwned>(&self, table: &String, query: &String) -> RequestData<T> {
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
                let result = self.http_caller.execute(r).await;
                if let Err(e) = result {
                    panic!("{}", e)
                }
                let json = result.unwrap().json::<T>().await;
                match json {
                    Err(e) => panic!("{}", e),
                    Ok(j) => RequestData { content: j },
                }
            }
        }
    }
}
