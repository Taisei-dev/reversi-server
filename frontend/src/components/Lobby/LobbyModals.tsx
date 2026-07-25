import React from 'react';
import type { PlayerColor, WaitingClientInfo, TournamentMatchDetail, RoomInfo } from '../../types';

interface LobbyModalsProps {
  showCreateRoomModal: boolean;
  onCloseCreateRoomModal: () => void;
  onCreateRoomSubmit: (e: React.FormEvent) => void;
  newRoomId: string;
  onChangeNewRoomId: (val: string) => void;
  newRoomMatchCount: number;
  onChangeNewRoomMatchCount: (val: number) => void;
  newRoomTimeMs: number;
  onChangeNewRoomTimeMs: (val: number) => void;

  editingRoom: RoomInfo | null;
  onCloseEditRoomModal: () => void;
  onEditRoomSubmit: (e: React.FormEvent) => void;
  editMatchCount: number;
  onChangeEditMatchCount: (val: number) => void;
  editTimeMs: number;
  onChangeEditTimeMs: (val: number) => void;

  showAddAiModal: string | null;
  onCloseAddAiModal: () => void;
  onAddAiSubmit: (e: React.FormEvent) => void;

  showStartAiModal: boolean;
  onCloseStartAiModal: () => void;
  onStartAiSubmit: (e: React.FormEvent) => void;

  aiType: 'random' | 'egaroucid';
  onChangeAiType: (val: 'random' | 'egaroucid') => void;
  aiLevel: number;
  onChangeAiLevel: (val: number) => void;
  aiUseBook: boolean;
  onChangeAiUseBook: (val: boolean) => void;

  playerColor: PlayerColor;
  onChangePlayerColor: (val: PlayerColor) => void;

  selectedClientForMatch: WaitingClientInfo | null;
  onCloseVsHumanModal: () => void;
  onConfirmStartVsHuman: (e: React.FormEvent) => void;
  vsHumanColor: PlayerColor;
  onChangeVsHumanColor: (val: PlayerColor) => void;

  selectedCellMatches: TournamentMatchDetail[] | null;
  onCloseCellMatchesModal: () => void;
  onReplayMatch: (moves: string[], blackName: string, whiteName: string) => void;
}

export const LobbyModals: React.FC<LobbyModalsProps> = ({
  showCreateRoomModal,
  onCloseCreateRoomModal,
  onCreateRoomSubmit,
  newRoomId,
  onChangeNewRoomId,
  newRoomMatchCount,
  onChangeNewRoomMatchCount,
  newRoomTimeMs,
  onChangeNewRoomTimeMs,

  editingRoom,
  onCloseEditRoomModal,
  onEditRoomSubmit,
  editMatchCount,
  onChangeEditMatchCount,
  editTimeMs,
  onChangeEditTimeMs,

  showAddAiModal,
  onCloseAddAiModal,
  onAddAiSubmit,

  showStartAiModal,
  onCloseStartAiModal,
  onStartAiSubmit,

  aiType,
  onChangeAiType,
  aiLevel,
  onChangeAiLevel,
  aiUseBook,
  onChangeAiUseBook,

  playerColor,
  onChangePlayerColor,

  selectedClientForMatch,
  onCloseVsHumanModal,
  onConfirmStartVsHuman,
  vsHumanColor,
  onChangeVsHumanColor,

  selectedCellMatches,
  onCloseCellMatchesModal,
  onReplayMatch,
}) => {
  return (
    <>
      {/* Modal 1: 新規ルーム作成 */}
      {showCreateRoomModal && (
        <div className="modal-overlay">
          <div className="modal-card">
            <h3>新規ルームの作成</h3>
            <form onSubmit={onCreateRoomSubmit} className="form-group-wrapper">
              <div className="input-group">
                <label>ルームID / 名前</label>
                <input
                  type="text"
                  value={newRoomId}
                  onChange={(e) => onChangeNewRoomId(e.target.value)}
                  required
                />
              </div>

              <div className="input-group">
                <label>同じ相手との対戦ラウンド数</label>
                <input
                  type="number"
                  min="1"
                  max="50"
                  value={newRoomMatchCount}
                  onChange={(e) => onChangeNewRoomMatchCount(parseInt(e.target.value, 10) || 1)}
                  required
                />
              </div>

              <div className="input-group">
                <label>持ち時間 (ms) (clientのみに適用,AIは制限なし)</label>
                <input
                  type="number"
                  min="100"
                  step="100"
                  value={newRoomTimeMs}
                  onChange={(e) => onChangeNewRoomTimeMs(parseInt(e.target.value, 10))}
                />
              </div>

              <div className="modal-button-group">
                <button type="button" className="btn btn-outline" onClick={onCloseCreateRoomModal}>
                  キャンセル
                </button>
                <button type="submit" className="btn btn-primary">
                  作成
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal 1.5: ルームの編集 */}
      {editingRoom && (
        <div className="modal-overlay">
          <div className="modal-card">
            <h3>Room #{editingRoom.room_id} の編集</h3>
            <form onSubmit={onEditRoomSubmit} className="form-group-wrapper">
              <div className="input-group">
                <label>対戦ラウンド数</label>
                <input
                  type="number"
                  min="1"
                  max="50"
                  value={editMatchCount}
                  onChange={(e) => onChangeEditMatchCount(parseInt(e.target.value, 10) || 1)}
                  required
                />
              </div>

              <div className="input-group">
                <label>持ち時間 (ms) (clientのみに適用,AIは制限なし)</label>
                <input
                  type="number"
                  min="100"
                  step="100"
                  value={editTimeMs}
                  onChange={(e) => onChangeEditTimeMs(parseInt(e.target.value, 10))}
                  required
                />
              </div>

              <div className="modal-button-group">
                <button type="button" className="btn btn-outline" onClick={onCloseEditRoomModal}>
                  キャンセル
                </button>
                <button type="submit" className="btn btn-primary">
                  保存
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal 2: ルームに AI を追加 */}
      {showAddAiModal && (
        <div className="modal-overlay">
          <div className="modal-card">
            <h3>Room #{showAddAiModal} に AI を追加</h3>
            <form onSubmit={onAddAiSubmit} className="form-group-wrapper">
              <div className="input-group">
                <label>AI の種類</label>
                <div className="segmented-control">
                  <button
                    type="button"
                    className={`segmented-btn ${aiType === 'egaroucid' ? 'active' : ''}`}
                    onClick={() => onChangeAiType('egaroucid')}
                  >
                    Egaroucid
                  </button>
                  <button
                    type="button"
                    className={`segmented-btn ${aiType === 'random' ? 'active' : ''}`}
                    onClick={() => onChangeAiType('random')}
                  >
                    Random
                  </button>
                </div>
              </div>

              {aiType === 'egaroucid' && (
                <>
                  <div className="input-group">
                    <label>Egaroucid レベル (0〜20)</label>
                    <input
                      type="number"
                      min="0"
                      max="20"
                      value={aiLevel}
                      onChange={(e) => onChangeAiLevel(parseInt(e.target.value, 10))}
                    />
                  </div>
                  <div className="input-group">
                    <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
                      <input
                        type="checkbox"
                        checked={aiUseBook}
                        onChange={(e) => onChangeAiUseBook(e.target.checked)}
                        style={{ width: 'auto', margin: 0 }}
                      />
                      定石を使用する (Use Book)
                    </label>
                  </div>
                </>
              )}

              <div className="modal-button-group">
                <button type="button" className="btn btn-outline" onClick={onCloseAddAiModal}>
                  キャンセル
                </button>
                <button type="submit" className="btn btn-primary">
                  AIを追加
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal 3: AIと対戦設定 */}
      {showStartAiModal && (
        <div className="modal-overlay">
          <div className="modal-card">
            <h3>AIとの対戦の設定</h3>
            <form onSubmit={onStartAiSubmit} className="form-group-wrapper">
              <div className="input-group">
                <label>AI の種類</label>
                <div className="segmented-control">
                  <button
                    type="button"
                    className={`segmented-btn ${aiType === 'egaroucid' ? 'active' : ''}`}
                    onClick={() => onChangeAiType('egaroucid')}
                  >
                    Egaroucid
                  </button>
                  <button
                    type="button"
                    className={`segmented-btn ${aiType === 'random' ? 'active' : ''}`}
                    onClick={() => onChangeAiType('random')}
                  >
                    Random
                  </button>
                </div>
              </div>

              {aiType === 'egaroucid' && (
                <>
                  <div className="input-group">
                    <label>Egaroucid レベル (0〜20)</label>
                    <input
                      type="number"
                      min="0"
                      max="20"
                      value={aiLevel}
                      onChange={(e) => onChangeAiLevel(parseInt(e.target.value, 10))}
                    />
                  </div>
                  <div className="input-group">
                    <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
                      <input
                        type="checkbox"
                        checked={aiUseBook}
                        onChange={(e) => onChangeAiUseBook(e.target.checked)}
                        style={{ width: 'auto', margin: 0 }}
                      />
                      定石を使用する (Use Book)
                    </label>
                  </div>
                </>
              )}

              <div className="input-group">
                <label>あなたの手番</label>
                <div className="segmented-control">
                  <button
                    type="button"
                    className={`segmented-btn ${playerColor === 'black' ? 'active' : ''}`}
                    onClick={() => onChangePlayerColor('black')}
                  >
                    先手 (黒)
                  </button>
                  <button
                    type="button"
                    className={`segmented-btn ${playerColor === 'white' ? 'active' : ''}`}
                    onClick={() => onChangePlayerColor('white')}
                  >
                    後手 (白)
                  </button>
                </div>
              </div>

              <div className="modal-button-group">
                <button type="button" className="btn btn-outline" onClick={onCloseStartAiModal}>
                  キャンセル
                </button>
                <button type="submit" className="btn btn-primary">
                  対戦開始
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal 5: clientと対戦開始時の手番選択ダイアログ */}
      {selectedClientForMatch && (
        <div className="modal-overlay">
          <div className="modal-card">
            <h3>{selectedClientForMatch.player_name} と対戦</h3>
            <form onSubmit={onConfirmStartVsHuman} className="form-group-wrapper">
              <div className="input-group">
                <label>あなたの手番を選んでください</label>
                <div className="segmented-control">
                  <button
                    type="button"
                    className={`segmented-btn ${vsHumanColor === 'black' ? 'active' : ''}`}
                    onClick={() => onChangeVsHumanColor('black')}
                  >
                    先手 (黒)
                  </button>
                  <button
                    type="button"
                    className={`segmented-btn ${vsHumanColor === 'white' ? 'active' : ''}`}
                    onClick={() => onChangeVsHumanColor('white')}
                  >
                    後手 (白)
                  </button>
                </div>
              </div>

              <div className="modal-button-group">
                <button
                  type="button"
                  className="btn btn-outline"
                  onClick={onCloseVsHumanModal}
                >
                  キャンセル
                </button>
                <button type="submit" className="btn btn-primary">
                  対戦スタート
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal 4: マトリクスセル選択時の全対局一覧 & 棋譜再生ダイアログ */}
      {selectedCellMatches && (
        <div className="modal-overlay">
          <div className="modal-card modal-card-wide">
            <h3>対戦詳細カード ({selectedCellMatches.length}対局)</h3>
            <div className="modal-matches-list">
              {selectedCellMatches.map((m, idx) => (
                <div key={idx} className="modal-match-item-horizontal">
                  <div className="m-versus-row">
                    <span className="p-name black">⚫ {m.p1_name} ({m.p1_score})</span>
                    <span className="m-sep">-</span>
                    <span className="p-name white">⚪ {m.p2_name} ({m.p2_score})</span>
                  </div>
                  <button
                    type="button"
                    className="btn btn-primary btn-small"
                    onClick={() => {
                      onCloseCellMatchesModal();
                      onReplayMatch(m.moves, m.p1_name, m.p2_name);
                    }}
                  >
                    <span>棋譜再生</span>
                  </button>
                </div>
              ))}
            </div>
            <div className="modal-button-group">
              <button type="button" className="btn btn-outline" onClick={onCloseCellMatchesModal}>
                閉じる
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
};
