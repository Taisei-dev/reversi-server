import React from 'react';
import type { FinishedMatchInfo, RoomTournamentResult, TournamentMatchDetail } from '../../types';
import { ChevronUp, ChevronDown } from 'lucide-react';

interface RecentHistorySectionProps {
  recentMatches: FinishedMatchInfo[];
  recentTournaments: RoomTournamentResult[];
  expandedTournaments: { [key: number]: boolean };
  onToggleMatrix: (idx: number) => void;
  onSelectCellMatches: (matches: TournamentMatchDetail[]) => void;
  onReplayMatch: (moves: string[], blackName: string, whiteName: string) => void;
  onNavigateHistory: () => void;
}

export const RecentHistorySection: React.FC<RecentHistorySectionProps> = ({
  recentMatches,
  recentTournaments,
  expandedTournaments,
  onToggleMatrix,
  onSelectCellMatches,
  onReplayMatch,
  onNavigateHistory,
}) => {
  return (
    <div className="card">
      <div className="card-header-with-action">
        <h2>
          <span>直近の対戦結果(3件)</span>
          <span className="help-btn-wrapper">
            <span className="help-btn">?</span>
            <span className="help-tooltip">
              直近に終了した1vs1対局およびルーム総当たり戦の成績記録です。<br />
              最近のものが上から順に3件まで表示され、それ以前の履歴は「全履歴を見る」ボタンから確認できます。<br />
              棋譜再生ボタンを押すことで履歴が確認できます。
            </span>
          </span>
        </h2>
        <button className="btn btn-outline btn-small" onClick={onNavigateHistory}>
          <span>全履歴を見る →</span>
        </button>
      </div>

      {recentTournaments.length > 0 && (
        <div className="tournament-history-section">
          <h3>総当たり戦ルーム</h3>
          <div className="tournament-results-list">
            {recentTournaments.map((tr, idx) => {
              const isExpanded = expandedTournaments[idx] || false;
              return (
                <div key={idx} className="tournament-card">
                  <div className="tournament-header">
                    <span>Room #{tr.room_id} 最終順位</span>
                    <button
                      className="btn btn-outline btn-small toggle-btn-compact"
                      onClick={() => onToggleMatrix(idx)}
                    >
                      {isExpanded ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
                      <span>{isExpanded ? '表を閉じる' : '対戦表を開く'}</span>
                    </button>
                  </div>

                  <table className="tournament-table">
                    <thead>
                      <tr>
                        <th>順位</th>
                        <th>プレイヤー</th>
                        <th>勝</th>
                        <th>負</th>
                        <th>点</th>
                      </tr>
                    </thead>
                    <tbody>
                      {(() => {
                        let currentRank = 1;
                        return tr.stats.map((st, sIdx) => {
                          if (sIdx > 0 && st.score < tr.stats[sIdx - 1].score) {
                            currentRank = sIdx + 1;
                          }
                          return (
                            <tr key={st.player_name}>
                              <td>{currentRank}</td>
                              <td className="p-name-cell">{st.player_name}</td>
                              <td>{st.wins}</td>
                              <td>{st.loses}</td>
                              <td className="score-cell">{st.score}</td>
                            </tr>
                          );
                        });
                      })()}
                    </tbody>
                  </table>

                  {isExpanded && (
                    <div className="matrix-table-wrapper accordion-open margin-top-small">
                      <div className="matrix-title" style={{ fontWeight: 700, marginBottom: '6px', fontSize: '0.825rem' }}>
                        対戦マトリクス表 (セルクリックで詳細・棋譜再生)
                      </div>
                      <table className="matrix-table">
                        <thead>
                          <tr>
                            <th>対戦</th>
                            {tr.stats.map((st) => (
                              <th key={st.player_name}>{st.player_name}</th>
                            ))}
                          </tr>
                        </thead>
                        <tbody>
                          {tr.stats.map((rowPlayer) => (
                            <tr key={rowPlayer.player_name}>
                              <th className="row-header">{rowPlayer.player_name}</th>
                              {tr.stats.map((colPlayer) => {
                                if (rowPlayer.player_name === colPlayer.player_name) {
                                  return <td key={colPlayer.player_name} className="same-cell">-</td>;
                                }
                                const matches = tr.match_matrix.filter(
                                  (m) =>
                                    (m.p1_name === rowPlayer.player_name && m.p2_name === colPlayer.player_name) ||
                                    (m.p2_name === rowPlayer.player_name && m.p1_name === colPlayer.player_name)
                                );
                                if (matches.length === 0) return <td key={colPlayer.player_name}>-</td>;

                                let winCount = 0;
                                let loseCount = 0;
                                matches.forEach((m) => {
                                  const isP1 = m.p1_name === rowPlayer.player_name;
                                  let res = m.result_text;
                                  if (!isP1 && res !== 'draw') res = res === 'win' ? 'lose' : 'win';
                                  if (res === 'win') winCount++;
                                  if (res === 'lose') loseCount++;
                                });

                                return (
                                  <td
                                    key={colPlayer.player_name}
                                    className="matrix-cell clickable win"
                                    onClick={() => onSelectCellMatches(matches)}
                                  >
                                    <div className="m-scores">
                                      {matches.length === 1
                                        ? `${matches[0].p1_score}-${matches[0].p2_score}`
                                        : `${winCount}勝${loseCount}負`}
                                    </div>
                                    <div className="m-result">({matches.length}戦)</div>
                                  </td>
                                );
                              })}
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* 1vs1 対局履歴が存在する場合のみ表示 */}
      {recentMatches.length > 0 && (
        <div className="single-history-section margin-top-small">
          <h3>1vs1 対局</h3>
          <div className="history-matches-list">
            {recentMatches.map((hm) => (
              <div key={hm.match_id} className="history-match-card-horizontal">
                <div className="hm-left-info">
                  <div className="hm-versus-text">
                    <span className="hm-player black">⚫ {hm.black_name} ({hm.black_stones})</span>
                    <span className="hm-separator">-</span>
                    <span className="hm-player white">⚪ {hm.white_name} ({hm.white_stones})</span>
                  </div>
                </div>
                <div className="hm-right-actions">
                  <button
                    className="btn btn-outline btn-small"
                    onClick={() => onReplayMatch(hm.moves, hm.black_name, hm.white_name)}
                  >
                    <span>再生</span>
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {recentTournaments.length === 0 && recentMatches.length === 0 && (
        <div className="empty-state">対局履歴はまだありません</div>
      )}
    </div>
  );
};
