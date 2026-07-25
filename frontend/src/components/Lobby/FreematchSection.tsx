import React from 'react';
import type { FreeMatchClientInfo } from '../../types';
import { Copy, Check } from 'lucide-react';

interface FreematchSectionProps {
  freematchClients: FreeMatchClientInfo[];
  copiedUrlType: string | null;
  onCopyUrl: (path: string, label: string) => void;
}

export const FreematchSection: React.FC<FreematchSectionProps> = ({
  freematchClients,
  copiedUrlType,
  onCopyUrl,
}) => {
  return (
    <div className="card">
      <h2>
        <span>フリーマッチ (参加中: {freematchClients.length})</span>
        <span className="help-btn-wrapper">
          <span className="help-btn">?</span>
          <span className="help-tooltip">
            接続されているclient同士が自動でマッチングされて対戦します。<br />
            勝負が行われた2つのclientが接続し続けてたいる場合は、クールダウン時間経過後に再度マッチングされます。<br />
            クールダウン時間はurlのパラメータで指定できます。<br />
            例: <code>?cooldown_sec=300</code> (300秒 = 5分)
          </span>
        </span>
      </h2>
      <p className="sub-desc-text">clientを以下のリンクに接続させると、他の接続しているclientと自動対戦できます。</p>

      <div className="url-copy-btn-wrapper">
        <span className="copy-btn-tooltip-wrapper">
          <button
            className="btn btn-blue-copy btn-small"
            onClick={() => onCopyUrl('/client/freematch', 'freematch')}
          >
            {copiedUrlType === 'freematch' ? <Check size={13} /> : <Copy size={13} />}
            <span>{copiedUrlType === 'freematch' ? 'コピー完了' : 'フリーマッチ 接続URL'}</span>
          </button>
          <span className="copy-url-tooltip">
            {(typeof window !== 'undefined' ? window.location.host : 'localhost:8080') + '/client/freematch'}
          </span>
        </span>
      </div>

      {freematchClients.length > 0 ? (
        <div className="tags-flex margin-top-small">
          {freematchClients.map((fc) => (
            <span key={fc.client_id} className="client-tag">
              {fc.player_name} ({fc.assigned_time_ms}ms)
            </span>
          ))}
        </div>
      ) : (
        <div className="empty-state">フリーマッチ参加中の client はいません</div>
      )}
    </div>
  );
};
