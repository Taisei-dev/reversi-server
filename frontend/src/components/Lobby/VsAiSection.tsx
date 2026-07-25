import React from 'react';
import { Copy, Check } from 'lucide-react';

interface VsAiSectionProps {
  copiedUrlType: string | null;
  onCopyUrl: (path: string, label: string) => void;
  onOpenStartAiModal: () => void;
}

export const VsAiSection: React.FC<VsAiSectionProps> = ({
  copiedUrlType,
  onCopyUrl,
  onOpenStartAiModal,
}) => {
  return (
    <div className="card">
      <h2>
        <span>AIと対戦する/させる</span>
        <span className="help-btn-wrapper">
          <span className="help-btn">?</span>
          <span className="help-tooltip">
            Egaroucid(Level 0~20) または Random AI と対局を行えます。<br />
            人間がWeb UIで対局することもClientを以下のurlに接続させて対戦させることもできます。<br />
            clientと対戦させるEgaroucidのレベルと定石を使用するかはurlのパラメータで指定できます。<br />
            例: <code>?level=7&use_book=true</code> (Egaroucid Level 7, 定石使用)
          </span>
        </span>
      </h2>
      <p className="sub-desc-text">clientを以下のリンクに接続させると、Egaroucid または Random AI と対局させることができます。</p>

      <div className="ai-copy-urls-group">
        <span className="copy-btn-tooltip-wrapper">
          <button
            className="btn btn-blue-copy btn-small"
            onClick={() => onCopyUrl('/client/ai/random', 'ai-random')}
          >
            {copiedUrlType === 'ai-random' ? <Check size={12} /> : <Copy size={12} />}
            <span>{copiedUrlType === 'ai-random' ? 'コピー完了' : 'ランダムAI 接続URL'}</span>
          </button>
          <span className="copy-url-tooltip">
            {(typeof window !== 'undefined' ? window.location.host : 'localhost:8080') + '/client/ai/random'}
          </span>
        </span>

        <span className="copy-btn-tooltip-wrapper">
          <button
            className="btn btn-blue-copy btn-small"
            onClick={() => onCopyUrl('/client/ai/egaroucid?level=7&use_book=true', 'ai-egaroucid')}
          >
            {copiedUrlType === 'ai-egaroucid' ? <Check size={12} /> : <Copy size={12} />}
            <span>{copiedUrlType === 'ai-egaroucid' ? 'コピー完了' : 'Egaroucid AI 接続URL'}</span>
          </button>
          <span className="copy-url-tooltip">
            {(typeof window !== 'undefined' ? window.location.host : 'localhost:8080') + '/client/ai/egaroucid?level=7&use_book=true'}
          </span>
        </span>
      </div>

      <button className="btn btn-primary btn-full margin-top-small" onClick={onOpenStartAiModal}>
        <span>AIと対戦する</span>
      </button>
    </div>
  );
};
