export interface Question {
  id: string;
  question_type: 'guest_quiz' | 'ich_oder_du';
  text: string;
  option_a?: string;
  option_b?: string;
  option_c?: string;
  option_d?: string;
  correct_answer: string;
  order_index: number;
  points: number;
}

export interface AddGuestQuizRequest {
  text: string;
  option_a: string;
  option_b: string;
  option_c: string;
  option_d: string;
  correct_answer: string;
  points?: number;
}

export interface AddIchOderDuRequest {
  text: string;
  correct_answer: string;
}
