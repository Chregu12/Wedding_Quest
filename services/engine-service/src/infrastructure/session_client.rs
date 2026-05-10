use serde::Deserialize;
use uuid::Uuid;

pub struct SessionClient {
    base_url: String,
    client: reqwest::Client,
}

impl SessionClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_questions(
        &self,
        session_code: &str,
    ) -> anyhow::Result<Vec<QuestionDto>> {
        let url = format!("{}/sessions/{}/questions", self.base_url, session_code);
        let questions = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Vec<QuestionDto>>()
            .await?;
        Ok(questions)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuestionDto {
    pub id: Uuid,
    pub question_type: String,
    pub text: String,
    pub option_a: Option<String>,
    pub option_b: Option<String>,
    pub option_c: Option<String>,
    pub option_d: Option<String>,
    pub correct_answer: String,
    pub order_index: i32,
    pub points: i32,
}
