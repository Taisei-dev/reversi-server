import React from 'react';
import type { RoomInfo } from '../../types';
import { Copy, Check } from 'lucide-react';

interface RoomListSectionProps {
  openRooms: RoomInfo[];
  copiedUrlType: string | null;
  onCopyUrl: (path: string, label: string) => void;
  onOpenCreateRoomModal: () => void;
  onOpenEditRoomModal: (room: RoomInfo) => void;
  onDeleteRoom: (roomId: string) => void;
  onOpenAddAiModal: (roomId: string) => void;
  onRemoveAi: (roomId: string, aiName: string) => void;
  onStartTournament: (roomId: string) => void;
}

export const RoomListSection: React.FC<RoomListSectionProps> = ({
  openRooms,
  copiedUrlType,
  onCopyUrl,
  onOpenCreateRoomModal,
  onOpenEditRoomModal,
  onDeleteRoom,
  onOpenAddAiModal,
  onRemoveAi,
  onStartTournament,
}) => {
  return (
    <div className="card column-header-card">
      <div className="card-header-with-action">
        <h2>
          <span>総当たり戦ルーム ({openRooms.length})</span>
          <span className="help-btn-wrapper">
            <span className="help-btn">?</span>
            <span className="help-tooltip">
              複数の Client や AI を追加して総当たり戦を実施できます。<br />
              各clientは自分以外の相手とラウンドの回数分だけ対戦します。<br />
              持ち時間はclientのみに適用され、AIに制限はありません。<br />
              誰かがスタートを押すと対戦が開始されます。<br />
              開始には2つ以上のclientまたはAIが必要です。
            </span>
          </span>
        </h2>
        <button
          className="btn btn-primary btn-small"
          onClick={onOpenCreateRoomModal}
        >
          <span>+ 新規ルーム作成</span>
        </button>
      </div>

      {openRooms.length > 0 ? (
        <div className="room-list">
          {openRooms.map((room) => (
            <div key={room.room_id} className="room-card-advanced">
              <div className="room-header">
                <div className="room-title-meta">
                  <span className="room-id-title">Room #{room.room_id}</span>
                  <span className="room-time-badge">{room.match_count}ラウンド</span>
                  <span className="room-time-badge">{room.assigned_time_ms}ms</span>
                </div>
                <div className="room-header-actions">
                  <span className="copy-btn-tooltip-wrapper">
                    <button
                      className="btn btn-blue-copy btn-small copy-url-btn"
                      onClick={() => onCopyUrl(`/client/room/${room.room_id}`, `room-${room.room_id}`)}
                    >
                      {copiedUrlType === `room-${room.room_id}` ? <Check size={12} /> : <Copy size={12} />}
                      <span>{copiedUrlType === `room-${room.room_id}` ? 'コピー完了' : '接続URL'}</span>
                    </button>
                    <span className="copy-url-tooltip">
                      {(typeof window !== 'undefined' ? window.location.origin : 'http://localhost:8080') + `/client/room/${room.room_id}`}
                    </span>
                  </span>
                  {room.status === 'waiting' && (
                    <button
                      className="delete-room-btn"
                      onClick={() => {
                        if (confirm('ルームを削除してもよろしいですか？')) {
                          onDeleteRoom(room.room_id);
                        }
                      }}
                      title="ルームを削除"
                    >
                      ✕
                    </button>
                  )}
                </div>
              </div>

              <div className="room-tags-wrapper">
                <div className="tag-group">
                  <label>Client ({room.current_clients}):</label>
                  <div className="tags-flex">
                    {room.client_names.length > 0 ? (
                      room.client_names.map((name, i) => (
                        <span key={i} className="client-tag">
                          {name}
                        </span>
                      ))
                    ) : (
                      <span className="no-member-text">なし</span>
                    )}
                  </div>
                </div>

                <div className="tag-group">
                  <label>AI ({room.ai_count}):</label>
                  <div className="tags-flex">
                    {room.ai_names.length > 0 ? (
                      room.ai_names.map((name, i) => (
                        <span key={i} className="ai-tag removable-tag">
                          <span>{name}</span>
                          {room.status === 'waiting' && (
                            <button
                              className="remove-tag-btn"
                              onClick={() => onRemoveAi(room.room_id, name)}
                              title="AIを削除"
                            >
                              ✕
                            </button>
                          )}
                        </span>
                      ))
                    ) : (
                      <span className="no-member-text">なし</span>
                    )}
                  </div>
                </div>
              </div>

              {room.status === 'waiting' && (
                <div className="room-actions-bar">
                  <button
                    className="btn btn-outline btn-small"
                    onClick={() => onOpenEditRoomModal(room)}
                  >
                    <span>編集</span>
                  </button>

                  <button
                    className="btn btn-outline btn-small"
                    onClick={() => onOpenAddAiModal(room.room_id)}
                  >
                    <span>+ AI追加</span>
                  </button>

                  <button
                    className="btn btn-primary btn-small"
                    onClick={() => onStartTournament(room.room_id)}
                  >
                    <span>スタート</span>
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      ) : (
        <div className="empty-state">現在開いているルームはありません</div>
      )}
    </div>
  );
};
