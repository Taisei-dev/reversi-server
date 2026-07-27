import React from 'react';

export const ProxyGuideBanner: React.FC = () => {
  return (
    <div className="proxy-info-banner-simple">
      <div className="proxy-banner-content">
        <div className="proxy-banner-title">
          これは何? - 色々できるリバーシサーバー。人間が自作のclientやAIと戦ったり、自作clientをAIや他の人と戦わせることができる。<br />
          <strong>接続にはProxyが必要です!</strong> 自作ClientはTCPしか喋れないので WebSocket 変換プロキシを経由して本サーバーに接続してください。<br />
          Rust版のProxyが用意されています。<a href="https://github.com/Taisei-dev/reversi-proxy" target="_blank" rel="noopener noreferrer">GitHub</a>
        </div>
        <div className="proxy-banner-commands">
          <span>• <strong>Rust Proxy</strong>: <code>cargo run -- [-d] &lt;LOCAL_PORT&gt; &lt;接続URL&gt;</code> </span>
        </div>
      </div>
    </div>
  );
};
