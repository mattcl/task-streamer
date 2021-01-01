use reqwest;

use crate::config::Config;
use crate::tasks::TaskClient;
use crate::error::Result;

#[derive(Debug)]
enum Verb {
    POST,
}

#[derive(Debug)]
pub struct Client {
    base_url: String,
    api_key: String,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Client {
            base_url: config.client.server.unwrap(),
            api_key: config.client.api_key.unwrap(),
        }
    }

    fn base_request(&self, verb: Verb, path: &str) -> reqwest::RequestBuilder {
        let full_path = format!("{}/{}", self.base_url, path);
        let client = reqwest::Client::new();

        let builder = match verb {
            Verb::POST => client.post(&full_path),
        };

        // TODO: figure out auth header - MCL - 2020-12-31
        // builder.header("", self.api_key.clone())
        builder
    }

    pub async fn push_tasks(&self, task_client: &TaskClient) -> Result<()> {
        self.base_request(Verb::POST, "tasks")
            .json(&task_client.tasks)
            .send()
            .await?;
        Ok(())
    }
}
