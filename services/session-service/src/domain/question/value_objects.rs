use crate::errors::AppError;

#[derive(Debug, Clone, PartialEq)]
pub enum QuestionType {
    GuestQuiz,
    IchOderDu,
}

impl QuestionType {
    pub fn as_str(&self) -> &str {
        match self {
            QuestionType::GuestQuiz => "guest_quiz",
            QuestionType::IchOderDu => "ich_oder_du",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, AppError> {
        match s {
            "guest_quiz" => Ok(QuestionType::GuestQuiz),
            "ich_oder_du" => Ok(QuestionType::IchOderDu),
            _ => Err(AppError::BadRequest(format!("Unknown question type: {s}"))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnswerOption {
    A,
    B,
    C,
    D,
}

impl AnswerOption {
    pub fn from_str(s: &str) -> Result<Self, AppError> {
        match s.to_uppercase().as_str() {
            "A" => Ok(AnswerOption::A),
            "B" => Ok(AnswerOption::B),
            "C" => Ok(AnswerOption::C),
            "D" => Ok(AnswerOption::D),
            _ => Err(AppError::BadRequest(format!(
                "Invalid answer option: {s}. Must be A, B, C or D"
            ))),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            AnswerOption::A => "A".into(),
            AnswerOption::B => "B".into(),
            AnswerOption::C => "C".into(),
            AnswerOption::D => "D".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CoupleAnswer {
    Ich,
    Du,
}

impl CoupleAnswer {
    pub fn from_str(s: &str) -> Result<Self, AppError> {
        match s.to_lowercase().as_str() {
            "ich" => Ok(CoupleAnswer::Ich),
            "du" => Ok(CoupleAnswer::Du),
            _ => Err(AppError::BadRequest(format!(
                "Invalid couple answer: {s}. Must be 'ich' or 'du'"
            ))),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            CoupleAnswer::Ich => "ich".into(),
            CoupleAnswer::Du => "du".into(),
        }
    }
}
