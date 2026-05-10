export interface PlayerScore {
  player_id: string;
  player_name: string;
  total_score: number;
  rounds_played: number;
  last_round_score: number;
  rank: number;
}

export interface ScoreConfig {
  tier1_max_seconds: number;
  tier2_max_seconds: number;
  tier1_multiplier: number;
  tier2_multiplier: number;
  tier3_multiplier: number;
  base_points: number;
}
