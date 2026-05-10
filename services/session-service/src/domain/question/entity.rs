use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::value_objects::{AnswerOption, CoupleAnswer, QuestionType};

#[derive(Debug, Clone)]
pub struct Question {
    pub id: Uuid,
    pub session_id: Uuid,
    pub question_type: QuestionType,
    pub text: String,
    pub option_a: Option<String>,
    pub option_b: Option<String>,
    pub option_c: Option<String>,
    pub option_d: Option<String>,
    pub correct_answer: String, // "A"|"B"|"C"|"D" for guest_quiz, "ich"|"du" for ich_oder_du
    pub order_index: i32,
    pub points: i32,
    pub created_at: DateTime<Utc>,
}

impl Question {
    pub fn create_guest_quiz(
        session_id: Uuid,
        text: String,
        option_a: String,
        option_b: String,
        option_c: String,
        option_d: String,
        correct_answer: AnswerOption,
        order_index: i32,
        points: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            question_type: QuestionType::GuestQuiz,
            text,
            option_a: Some(option_a),
            option_b: Some(option_b),
            option_c: Some(option_c),
            option_d: Some(option_d),
            correct_answer: correct_answer.to_string(),
            order_index,
            points,
            created_at: Utc::now(),
        }
    }

    pub fn create_ich_oder_du(
        session_id: Uuid,
        text: String,
        correct_answer: CoupleAnswer,
        order_index: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            question_type: QuestionType::IchOderDu,
            text,
            option_a: None,
            option_b: None,
            option_c: None,
            option_d: None,
            correct_answer: correct_answer.to_string(),
            order_index,
            points: 0,
            created_at: Utc::now(),
        }
    }
}
