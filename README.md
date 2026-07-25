# reversi-server

オセロ（リバーシ）の対局管理・総当たり戦トーナメント・AI（Egaroucid / Random）対戦・Web UI 観戦および全自動フリーマッチを提供する統合サーバーシステムです。

サーバーは Port 8080 の Axum HTTP / WebSocket Bridge に一本化されており、TCP クライアント（Rust / OCaml 等）は軽量プロキシ (`reversi-proxy-rs` または `reversi-proxy-ml`) を経由して接続します。

---
## 機能一覧

| 機能 | 説明 |
| :--- | :--- |
| フリーマッチ | 接続されている Client 同士の自動マッチング・対局 |
| ルーム | ルームを作成して複数ラウンド総当たり戦<br>AI 追加も可能 |
| clientと対戦 | Clientを接続して人間とUIを通して対戦 |
| AI と対戦 | Egaroucid（レベル 0〜20・定石ブック有無選択可）または Random AI に接続して対局<br>人間が Web UI 上で対局もできる |
| 対戦履歴 & 棋譜再生 | 総当たり戦の結果を表で可視化<br>対局履歴からのインタラクティブ棋譜再生 |

---

## Proxy を経由した Client 接続

### Proxy を挟む理由

自作クライアントプログラム（C, Rust, OCaml 等）は、ホストアドレスとポートでアクセスする 標準的な TCP ソケット通信で対局プロトコル（`OPEN`, `MOVE`, `START` など）を送受信します。  
一方で、本サーバー（`reversi-server`）は Web UI から接続することと、パスを変えることで複数の入り口を設けるため WebSocket インターフェースで動作しています。

Proxy (`reversi-proxy-rs` や `reversi-proxy-ml`) は、クライアントからの TCP 接続を受け付け、本サーバーの WebSocket 端点へ相互変換・中継する役割を担います。

### 接続・利用手順のフロー

対局プログラムを接続してプレイ・対戦を行う場合、以下の手順で進めます。

0. **プロキシのレポジトリをクローン**
   ```bash
   git clone https://github.com/Taisei-dev/reversi-proxy
   ```
1. **Web UI で接続先 URL の取得・設定**
   ブラウザで サーバーのURL (ローカルで起動している場合は http://localhost:8080) にアクセスします。  
   「フリーマッチ」「ルーム対戦」「vs AI」「client と対戦」など目的のモードの接続 URL（UI ボタンをクリックすると コピー）から、クエリパラメータを含めたターゲット URL を取得します。
2. **Proxy の起動**
   ターミナルで Proxy を起動します。引数の順序は `<LOCAL_PORT>` `<TARGET_URL>` です。
   `<LOCAL_PORT>` は自作クライアントが接続するローカル TCP ポート番号、`<TARGET_URL>` は Web UI で取得した接続先 URL です。
   `<LoCAL_PORT>` は任意の空いているポート番号を指定してください。  
   `<TARGET_URL>` は Web UI でコピーした URL をそのまま貼り付けます。
   プロキシのレポジトリの README を参照し、以下のように実行します。
   Rust 版 と OCaml 版のどちらでも構いません。
   ```bash
   # Rust 版
   cd reversi-proxy/reversi-proxy-rs
   cargo run --release -- <LOCAL_PORT> <TARGET_URL>
   # または
   # OCaml 版
   cd reversi-proxy/reversi-proxy-ml
   dune exec ./proxy.exe -- <LOCAL_PORT> <TARGET_URL>
   ```
3. **Client プログラムの起動**
   自作クライアントプログラムを起動し、Proxy がリッスンしているローカル TCP ポート (`localhost:<LOCAL_PORT>`) へ接続させます。
   Rust版のクライアントの例
   ```bash
   cargo run -- -p <LOCAL_PORT> -n <PLAYER_NAME>
   ```
   OCaml版のクライアントの例
   ```bash
   ./main.exe -p <LOCAL_PORT> -n <PLAYER_NAME>
   ```
4. **Web UI に戻って操作・対局**
   ブラウザに戻り、接続中のクライアント一覧の確認、対局の開始操作（ルーム対戦のスタート等）、あるいはWeb UI を通してリアルタイム対局を行います。

---

## Client 接続 URL & クエリパラメータ一覧

Proxy で指定可能なパスおよびクエリパラメータの一覧です。

| モード / 用途 | 接続 URL パス | クエリパラメータ | デフォルト値 | 説明 |
| :--- | :--- | :--- | :--- | :--- |
| フリーマッチ | `/client/freematch` | `time_ms`<br>`cooldown_sec`<br>`color` | `1000`<br>`10`<br>`random` | 自動対戦キュー。`cooldown_sec` 秒間は同一対戦相手との再マッチを回避 |
| ルーム対戦 | `/client/room/<ROOM_ID>` | `time_ms`<br>`color` | `1000`<br>`random` | 指定ルームへ参加待機。Web UI からスタートして総当たり戦を実施 |
| Clientと対戦 待機 | `/client/vs-human` | `time_ms` | `1000` | 人間 (Web UI) との対戦待機。対局終了後は自動で待機キューに復帰 |
| vs Egaroucid AI | `/client/ai/egaroucid` | `level`<br>`use_book`<br>`time_ms`<br>`color` | `7`<br>`true`<br>`1000`<br>`random` | Egaroucid AI と対局。`level` 0〜20 、`use_book` で強さを調整 |
| vs Random AI | `/client/ai/random` | `time_ms`<br>`color` | `1000`<br>`random` | ランダムに指し手を選ぶ AI と対局 |

### クエリパラメータの詳細

| パラメータ | 型 | 説明 |
| :--- | :--- | :--- |
| `time_ms` | `integer` | 初期持ち時間（ms）<br>人間とAIには減算・タイムアウト制御は適用されない |
| `cooldown_sec` | `integer` | フリーマッチで同一の相手と連続マッチングするまでの待機時間（秒<br>連戦し続けるのを防ぐため |
| `level` | `integer` | Egaroucid AI の探索深度 / 強さ（`0` 〜 `20`） |
| `use_book` | `boolean` | 定石を使用するかどうか（`true` / `false`） |
| `color` | `string` | 希望手番（`black` / `white` / `random`） |
---

## Proxy 起動コマンド例

Proxyの起動コマンド構文：

```bash
./proxy [-d|--debug] <LOCAL_PORT> <TARGET_URL>
```
`./proxy`の部分はRust版かOCaml版かで異なります。  
`-d` または `--debug` を指定すると、TCP ↔ WebSocket の通信内容をデバッグ出力します。

引数はローカルの TCP ポート番号 (`<LOCAL_PORT>`) が先で、接続先のサーバー URL (`<TARGET_URL>`) が後になります。
この例でのポート番号は一例です。空いているポート番号を指定してください。
プロキシは停止するまで待機し続けます。同一のプロキシで同じ接続先 URL に複数の Client を接続することも可能です。

| 目的 | コマンド例 |
| :--- | :--- |
| フリーマッチに参加する（ポート8000、クールダウン15秒） | `./proxy 8000 localhost:8080/client/freematch?cooldown_sec=15` |
| ルーム 1 に接続（ポート8001） | `./proxy 8001 localhost:8080/client/room/1` |
| Egaroucid Level 5 に先手で対局（ポート8002） | `./proxy 8002 "localhost:8080/client/ai/egaroucid?level=5&color=black"` |
| clientとWeb UIで対戦（ポート8003、デバッグ表示あり） | `./proxy -d 8003 localhost:8080/client/vs-human` |

---

## リポジトリ構成

```
reversi-server/
├── src/               # Rust サーバー本体
├── dist/              # Web UI ビルド成果物 (frontend ビルド時に自動生成、Git管理外)
├── frontend/          # Web UI ソースコード (React + TypeScript)
├── Egaroucid/         # Egaroucid AI エンジン (git submodule)
├── build.rs           # cc クレートで Egaroucid C++ を静的ビルド
├── Cargo.toml
├── Cargo.lock
├── LICENSE
└── README.md
```

### Egaroucid について

リバーシ AI [Egaroucid](https://github.com/Nyanyan/Egaroucid) を git submodule として内包しています。  
C++ ライブラリ (`egaroucid_c_api.cpp`) を `build.rs` 経由で自動的に静的コンパイル・リンクします。

---

## セットアップ & 起動

初回起動時や Web UI を更新した際は、先に `frontend` のビルドを行ってください。

### 1. クローン（submodule 含む）

```bash
git clone --recurse-submodules <このリポジトリのURL>
# または clone 後に
git submodule update --init --recursive
```

### 2. Web UI のビルド

```bash
cd frontend
npm install
npm run build   # ../dist/ ディレクトリへビルド出力
cd ..
```

### 3. サーバーの起動

```bash
cargo run
```

Egaroucid の C++ ソースを初回ビルド時にコンパイルするため、最初の `cargo build` には時間がかかる場合があります。

- Web UI (ロビー画面): http://localhost:8080
- Client WebSocket Bridge: `ws://localhost:8080/client/...`


---

## アーキテクチャ概要

```
TCP Client ─── Proxy (rs/ml) ──┐
                                 ├── WebSocket ─── Axum Server (Port 8080)
Web Browser ─────────────────── ┘         │
                                           ├── FreeMatch Queue
                                           ├── Room Manager (総当たり戦)
                                           ├── WaitHuman Queue
                                           ├── AI Match (Egaroucid C++ FFI / Random)
                                           └── Match History Registry
```

---

### プロトコル（Client ↔ Server）

| 方向 | メッセージ | 意味 |
| :--- | :--- | :--- |
| Client → Server | `OPEN <NAME>` | 接続・名前登録 |
| Server → Client | `START <COLOR> <OPP_NAME> <TIME_MS>` | 対局開始 |
| Client → Server | `MOVE <POS>` | 着手（例: `D3`、パスは `PASS`） |
| Server → Client | `MOVE <POS>` | 相手の着手通知 |
| Server → Client | `ACK` | 着手受理 |
| Server → Client | `END <RESULT> <MY_STONES> <OPP_STONES> <REASON>` | 対局終了 |

---

## ライセンス

本プロジェクトは GNU General Public License v3.0 (GPL-3.0) のもとで公開されています。詳細は `LICENSE` ファイルを参照してください。  
また、内包している Egaroucid は [Nyanyan/Egaroucid](https://github.com/Nyanyan/Egaroucid) のライセンスに従います。
