import React from 'react';
import type { ActiveMatchInfo } from '../../types';

interface ActiveMatchesSectionProps {
  activeMatches: ActiveMatchInfo[];
}

export const ActiveMatchesSection: React.FC<ActiveMatchesSectionProps> = ({ activeMatches }) => {
  return (
    <div className="card">
      <h2>
        <span>進行中の対局 ({activeMatches.length})</span>
        <span className="help-btn-wrapper">
          <span className="help-btn">?</span>
          <span className="help-tooltip">
            現在リアルタイムに進行している対局の一覧です。<br />
            終了すると結果は対戦結果に保存され、履歴を確認できます。
          </span>
        </span>
      </h2>
      {activeMatches.length > 0 ? (
        <div className="active-matches-list">
          {activeMatches.map((m) => (
            <div key={m.match_id} className="match-detail-card">
              <div className="match-card-header">
                <span className="match-move-count">{m.move_count} 手経過</span>
              </div>
              <div className="match-vs-row">
                <span className="player-name-black">⚫ {m.black_name}</span>
                <span className="vs-symbol">VS</span>
                <span className="player-name-white">⚪ {m.white_name}</span>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="empty-state">現在進行中の対局はありません</div>
      )}
    </div>
  );
};
