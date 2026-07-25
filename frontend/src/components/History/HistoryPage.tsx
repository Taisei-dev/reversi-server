import React, { useState, useEffect } from 'react';
import type { HistoryData, GameConfig, TournamentMatchDetail } from '../../types';
import { ArrowLeft } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

interface HistoryPageProps {
  onStartGame: (config: GameConfig) => void;
}

export const HistoryPage: React.FC<HistoryPageProps> = ({ onStartGame }) => {
  const [historyData, setHistoryData] = useState<HistoryData>({
    recent_matches: [],
    tournament_results: [],
  });
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [expandedTournaments, setExpandedTournaments] = useState<{ [key: number]: boolean }>({});
  const [selectedMatrixMatch, setSelectedMatrixMatch] = useState<TournamentMatchDetail | null>(null);

  const navigate = useNavigate();

  const fetchHistory = async () => {
    setIsRefreshing(true);
    try {
      const res = await fetch('/api/history');
      if (res.ok) {
        const data = await res.json();
        setHistoryData(data);
      }
    } catch (e) {
      console.error('Failed to fetch history', e);
    } finally {
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    fetchHistory();
  }, []);

  const toggleTournamentMatrix = (idx: number) => {
    setExpandedTournaments((prev) => ({
      ...prev,
      [idx]: !prev[idx],
    }));
  };

  const handleReplayMatch = (moves: string[], blackName: string, whiteName: string) => {
    onStartGame({
      playerName: 'Player_Web',
      mode: 'kifu-replay',
      color: 'black',
      timeMs: 0,
      replayMoves: moves,
      replayBlackName: blackName,
      replayWhiteName: whiteName,
    });
  };

  return (
    <div className="history-page-container">
      <header className="app-header">
        <button className="btn btn-outline btn-small" onClick={() => navigate('/')}>
          <ArrowLeft size={14} />
          <span>ロビーへ戻る</span>
        </button>
        <div className="logo">
          <span>⚫⚪ 対局履歴・成績一覧</span>
        </div>
        <button className="btn btn-outline btn-small" onClick={fetchHistory} disabled={isRefreshing}>
          <span>更新</span>
        </button>
      </header>

      <main className="history-page-main" style={{ padding: '24px', maxWidth: '1200px', margin: '0 auto' }}>
        <div className="history-page-grid" style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px', alignItems: 'start' }}>
          {/* Column 1: 総当たりルーム結果一覧 */}
          <section className="card">
            <h2>総当たり戦ルーム 結果履歴 ({historyData.tournament_results.length})</h2>
            {historyData.tournament_results.length > 0 ? (
              <div className="tournament-results-list margin-top-small">
                {historyData.tournament_results.map((tr, idx) => {
                  const isExpanded = expandedTournaments[idx] || false;
                  return (
                    <div key={idx} className="tournament-card">
                      <div className="tournament-header">
                        <span>Room #{tr.room_id} 最終順位</span>
                        <button
                          className="btn btn-outline btn-small"
                          onClick={() => toggleTournamentMatrix(idx)}
                        >
                          <span>{isExpanded ? '対戦表を閉じる' : '対戦表を開く'}</span>
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

                      {/* Accordion Toggle for Matrix Table */}
                      {isExpanded && (
                        <div className="matrix-table-wrapper accordion-open margin-top-small">
                          <div className="matrix-title" style={{ fontWeight: 700, marginBottom: '6px', fontSize: '0.85rem' }}>
                            対戦マトリクス表 (セルクリックで詳細)
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
                                    const match = tr.match_matrix.find(
                                      (m) =>
                                        (m.p1_name === rowPlayer.player_name && m.p2_name === colPlayer.player_name) ||
                                        (m.p2_name === rowPlayer.player_name && m.p1_name === colPlayer.player_name)
                                    );
                                    if (!match) return <td key={colPlayer.player_name}>-</td>;

                                    const isP1 = match.p1_name === rowPlayer.player_name;
                                    const myScore = isP1 ? match.p1_score : match.p2_score;
                                    const oppScore = isP1 ? match.p2_score : match.p1_score;
                                    let outcomeText = match.result_text;
                                    if (!isP1 && outcomeText !== 'draw') {
                                      outcomeText = outcomeText === 'win' ? 'lose' : 'win';
                                    }

                                    return (
                                      <td
                                        key={colPlayer.player_name}
                                        className={`matrix-cell clickable ${outcomeText}`}
                                        onClick={() => setSelectedMatrixMatch(match)}
                                      >
                                        <div className="m-scores">{myScore}-{oppScore}</div>
                                        <div className="m-result">({outcomeText})</div>
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
            ) : (
              <div className="empty-state">総当たりルームの履歴はまだありません</div>
            )}
          </section>

          {/* Column 2: 1vs1 対局結果履歴 */}
          <section className="card">
            <h2>1vs1 対局 結果履歴 ({historyData.recent_matches.length})</h2>
            {historyData.recent_matches.length > 0 ? (
              <div className="history-matches-list margin-top-small">
                {historyData.recent_matches.map((hm) => (
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
                        onClick={() => handleReplayMatch(hm.moves, hm.black_name, hm.white_name)}
                      >
                        <span>再生</span>
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="empty-state">1vs1 の対局履歴はまだありません</div>
            )}
          </section>
        </div>
      </main>

      {/* Modal: 対局詳細 */}
      {selectedMatrixMatch && (
        <div className="modal-overlay">
          <div className="modal-card">
            <h3>対局詳細カード</h3>
            <div className="modal-match-item-horizontal margin-top-small">
              <div className="m-versus-row">
                <span className="p-name black">⚫ {selectedMatrixMatch.p1_name} ({selectedMatrixMatch.p1_score})</span>
                <span className="m-sep">-</span>
                <span className="p-name white">⚪ {selectedMatrixMatch.p2_name} ({selectedMatrixMatch.p2_score})</span>
              </div>
            </div>
            <div className="modal-button-group">
              <button type="button" className="btn btn-outline" onClick={() => setSelectedMatrixMatch(null)}>
                閉じる
              </button>
              <button
                type="button"
                className="btn btn-primary"
                onClick={() => {
                  const m = selectedMatrixMatch;
                  setSelectedMatrixMatch(null);
                  handleReplayMatch(m.moves, m.p1_name, m.p2_name);
                }}
              >
                <span>棋譜を再生</span>
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
