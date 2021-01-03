use reqwest;

use crate::config::Config;
use crate::error::Result;
use crate::schema::Topic;
use crate::tasks::TaskClient;

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

        builder.header("Authorization", format!("Bearer {}", self.api_key))
    }

    pub async fn push_tasks(&self, task_client: &TaskClient) -> Result<()> {
        let response = self
            .base_request(Verb::POST, "tasks")
            .json(&task_client.tasks)
            .send()
            .await?;

        response.error_for_status()?;
        Ok(())
    }

    pub async fn set_topic(&self, topic: Topic) -> Result<()> {
        let response = self
            .base_request(Verb::POST, "topic")
            .json(&topic)
            .send()
            .await?;

        response.error_for_status()?;
        Ok(())
    }
}
