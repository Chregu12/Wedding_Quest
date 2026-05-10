export interface Session {
  id: string;
  code: string;
  status: string;
  person_a_name: string;
  person_b_name: string;
  host_name?: string;
  created_at?: string;
}

export interface CreateSessionRequest {
  person_a_name: string;
  person_b_name: string;
  host_name: string;
}

export interface CreateSessionResponse {
  session_id: string;
  code: string;
}

export interface JoinSessionRequest {
  display_name: string;
}

export interface JoinSessionResponse {
  player_id: string;
  session_id: string;
}

export interface PlayerInfo {
  id: string;
  display_name: string;
  total_score: number;
}

export interface PlayersResponse {
  session_code: string;
  players: PlayerInfo[];
}
