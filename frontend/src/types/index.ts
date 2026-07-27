export interface ActiveMatchInfo {
  match_id: string;
  mode: string;
  black_name: string;
  white_name: string;
  move_count: number;
  started_at_sec: number;
}

export interface RoomInfo {
  room_id: string;
  current_clients: number;
  ai_count: number;
  assigned_time_ms: number;
  match_count: number;
  client_names: string[];
  ai_names: string[];
  status: 'waiting' | 'running' | string;
}

export interface WaitingClientInfo {
  client_id: string;
  player_name: string;
  assigned_time_ms: number;
}

export interface FreeMatchClientInfo {
  client_id: string;
  player_name: string;
  assigned_time_ms: number;
  cooldown_sec: number;
}

export interface LobbyData {
  open_rooms: RoomInfo[];
  waiting_clients: WaitingClientInfo[];
  freematch_clients: FreeMatchClientInfo[];
  active_matches: ActiveMatchInfo[];
}

export interface PlayerStat {
  player_name: string;
  score: number;
  wins: number;
  loses: number;
}

export interface FinishedMatchInfo {
  match_id: string;
  mode: string;
  black_name: string;
  white_name: string;
  winner_color: string | null;
  black_stones: number;
  white_stones: number;
  reason: string;
  moves: string[];
  finished_at_sec: number;
}

export interface TournamentMatchDetail {
  p1_name: string;
  p2_name: string;
  p1_score: number;
  p2_score: number;
  result_text: string;
  moves: string[];
  round_index: number;
}

export interface RoomTournamentResult {
  room_id: string;
  finished_at_sec: number;
  stats: PlayerStat[];
  match_matrix: TournamentMatchDetail[];
}

export interface HistoryData {
  recent_matches: FinishedMatchInfo[];
  tournament_results: RoomTournamentResult[];
}

export type PlayerColor = 'black' | 'white';
export type MatchMode = 'vs-human' | 'vs-ai' | 'vs-ai-egaroucid' | 'room' | 'kifu-replay';

// 対局をまたいで持ち越されるユーザーの好み (localStorage に永続化される)
export interface GamePreferences {
  playerName: string;
  color: PlayerColor;
  aiType: 'random' | 'egaroucid';
  aiLevel: number;
  aiUseBook: boolean;
}

// 1対局を開始するための確定パラメータ
export interface GameConfig {
  preferences: GamePreferences;
  mode: MatchMode;
  timeMs: number;
  targetClientId?: string;
  roomId?: string;
  replayMoves?: string[];
  replayBlackName?: string;
  replayWhiteName?: string;
}

export type BoardState = number[][];
