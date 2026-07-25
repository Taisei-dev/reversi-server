import React from 'react';
import type { WaitingClientInfo } from '../../types';
import { Copy, Check } from 'lucide-react';

interface VsHumanSectionProps {
  waitingClients: WaitingClientInfo[];
  playerName: string;
  onChangePlayerName: (name: string) => void;
  copiedUrlType: string | null;
  onCopyUrl: (path: string, label: string) => void;
  onSelectClient: (client: WaitingClientInfo) => void;
}

export const VsHumanSection: React.FC<VsHumanSectionProps> = ({
  waitingClients,
  playerName,
  onChangePlayerName,
  copiedUrlType,
  onCopyUrl,
  onSelectClient,
}) => {
  return (
    <>
      <div className="card">
        <h2>
          <span>あなたのプレイヤー名</span>
          <span className="help-btn-wrapper">
            <span className="help-btn">?</span>
            <span className="help-tooltip">
              あなたがclientやAIと対戦するとき、あなたの名前として使われます
            </span>
          </span>
        </h2>
        <div className="input-group">
          <input
            type="text"
            value={playerName}
            onChange={(e) => onChangePlayerName(e.target.value)}
            placeholder="Webプレイヤー名"
            required
          />
        </div>
      </div>

      <div className="card">
        <h2>
          <span>clientと対戦 (待機中: {waitingClients.length})</span>
          <span className="help-btn-wrapper">
            <span className="help-btn">?</span>
            <span className="help-tooltip">
              接続されているclientと人間がWeb UIで対戦できます。<br />
              Clientを以下のurlに接続させることでここに表示されます。
            </span>
          </span>
        </h2>
      <p className="sub-desc-text">clientを以下のリンクに接続させると、ここに表示されWeb UIで人間と対戦できます。</p>

        <div className="url-copy-btn-wrapper margin-top-small">
          <span className="copy-btn-tooltip-wrapper">
            <button
              className="btn btn-blue-copy btn-small"
              onClick={() => onCopyUrl('/client/vs-human', 'vs-human')}
            >
              {copiedUrlType === 'vs-human' ? <Check size={13} /> : <Copy size={13} />}
              <span>{copiedUrlType === 'vs-human' ? 'コピー完了' : 'Client 接続URL'}</span>
            </button>
            <span className="copy-url-tooltip">
              {(typeof window !== 'undefined' ? window.location.host : 'localhost:8080') + '/client/vs-human'}
            </span>
          </span>
        </div>

        {waitingClients.length > 0 ? (
          <div className="client-wait-list margin-top-small">
            {waitingClients.map((client) => (
              <div key={client.client_id} className="client-wait-card">
                <div className="client-info">
                  <span className="client-name">{client.player_name}</span>
                  <span className="client-time">{client.assigned_time_ms}ms</span>
                </div>
                <div className="client-card-actions">
                  <button
                    className="btn btn-primary btn-small"
                    onClick={() => onSelectClient(client)}
                  >
                    <span>対戦する</span>
                  </button>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="empty-state">現在対戦を待っている client は居ません</div>
        )}
      </div>
    </>
  );
};
