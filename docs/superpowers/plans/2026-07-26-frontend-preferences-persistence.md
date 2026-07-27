# frontend 設定の localStorage 永続化 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** ロビー画面で毎回入力していたプレイヤー名と対局設定を localStorage に保存し、再訪時に復元する。

**Architecture:** 新しい設定概念を並立させず、既存の `GameConfig` から「対局をまたいで持ち越される部分」を `GamePreferences` として切り出す。`GameConfig` は `GamePreferences` を継承せず、`preferences` フィールドとして**保持する**（合成）。永続化されるのはこの `GamePreferences` だけで、読み書きとバリデーションは `usePreferences` フックに閉じ込める。

**Tech Stack:** React 19 / TypeScript 6 / Vite 8 / oxlint（テストランナーは導入しない）

## Global Constraints

- 作業ディレクトリは `frontend/`。パッケージマネージャは pnpm。
- 検証は `pnpm build`（`tsc -b && vite build`）と `pnpm lint`（oxlint）。テストフレームワークは導入しない。
- 各タスク終了時点で `pnpm build` が通る（＝ビルドを壊したまま次のタスクに進まない）。
- 型の再利用は継承ではなく合成で行う。`extends` を使わない。
- localStorage のキーは `reversi:preferences`。
- localStorage が使えない環境（プライベートモード等）でもアプリは動作すること。読み書きは必ず try/catch で保護し、失敗時はデフォルト値で動く。
- 保存された JSON は信用しない。フィールド単位で検証し、型不一致・欠損・範囲外はデフォルト値で補完する。
- コメントは日本語。冗長にしない。

## 型の構造

```ts
// 対局をまたいで持ち越されるユーザーの好み (localStorage に永続化される)
interface GamePreferences {
  playerName: string;
  color: PlayerColor;
  aiType: 'random' | 'egaroucid';
  aiLevel: number;
  aiUseBook: boolean;
}

// 1対局を開始するための確定パラメータ
interface GameConfig {
  preferences: GamePreferences;  // 継承ではなく保持する
  mode: MatchMode;
  timeMs: number;
  // ... モード固有の値
}
```

## 永続化する設定 / しない設定

| 対象 | 永続化 | 理由 |
| --- | --- | --- |
| `playerName` | する | 本要望の主目的 |
| `color`（手番） | する | vs AI / vs client で共通の好み |
| `aiType` / `aiLevel` / `aiUseBook` | する | 対戦モーダルと AI 追加モーダルで共有される |
| `newRoomId` | しない | ルームごとの識別子で毎回変える性質 |
| `newRoomTimeMs` / `newRoomMatchCount` | しない | ルーム設定であり `GameConfig` とは別概念 |
| 編集モーダルの値 (`editMatchCount` 等) | しない | 対象ルーム由来の値 |
| モーダル開閉などの一時状態 | しない | UI の一時状態 |

## 挙動変更（意図的）

- 現在 `playerColor`（vs AI 用）と `vsHumanColor`（vs client 用）は独立した state で、クライアント選択のたびに `vsHumanColor` は `black` にリセットされる。これを `prefs.color` 1つに統合するため、**vs client の手番選択も前回の選択を記憶する**ようになる。

## File Structure

| ファイル | 責務 | 変更 |
| --- | --- | --- |
| `frontend/src/types/index.ts` | ドメイン型定義 | `GamePreferences` を追加し、`GameConfig` にそれを保持させる |
| `frontend/src/hooks/usePreferences.ts` | 設定の永続化（デフォルト値・検証・load/save・React フック） | 新規作成 |
| `frontend/src/config/constants.ts` | ビルド時定数 | 未使用の `DEFAULT_COOLDOWN_SEC` を削除 |
| `frontend/src/components/Lobby/LobbyPage.tsx` | ロビーの状態管理 | 設定系 6 state を `usePreferences` に置換 |
| `frontend/src/components/Lobby/LobbyModals.tsx` | ロビーの各モーダル | `vsHumanColor` 系 props を削除し `playerColor` に統合 |
| `frontend/src/components/History/HistoryPage.tsx` | 履歴ページ | 棋譜再生時の `GameConfig` 生成を追随 |
| `frontend/src/App.tsx` | ルーティングと `GameConfig` 保持 | `defaultConfig` を追随 |
| `frontend/src/components/Game/GamePage.tsx` | 対局画面 | `config.preferences` 経由の参照に変更 |

---

### Task 1: `GamePreferences` 型と `usePreferences` フック

この時点では `GameConfig` は変更しない（追加のみ）。既存コードは無傷でビルドが通る。

**Files:**
- Modify: `frontend/src/types/index.ts:84`（`MatchMode` の直後に追加）
- Create: `frontend/src/hooks/usePreferences.ts`
- Modify: `frontend/src/config/constants.ts:2`

**Interfaces:**
- Consumes: `PlayerColor`（`types/index.ts:83` に既存）
- Produces:
  - `interface GamePreferences { playerName: string; color: PlayerColor; aiType: 'random' | 'egaroucid'; aiLevel: number; aiUseBook: boolean }`（`types` からエクスポート）
  - `const DEFAULT_PREFERENCES: GamePreferences`（`hooks/usePreferences` からエクスポート）
  - `const usePreferences: () => { prefs: GamePreferences; update: (patch: Partial<GamePreferences>) => void }`（`hooks/usePreferences` からエクスポート）

- [ ] **Step 1: `GamePreferences` を型定義に追加する**

`frontend/src/types/index.ts` の `export type MatchMode = ...`（84行目）の直後、`export interface GameConfig` の直前に挿入する。`GameConfig` 自体はこのタスクでは変更しない。

```ts
// 対局をまたいで持ち越されるユーザーの好み (localStorage に永続化される)
export interface GamePreferences {
  playerName: string;
  color: PlayerColor;
  aiType: 'random' | 'egaroucid';
  aiLevel: number;
  aiUseBook: boolean;
}
```

- [ ] **Step 2: `usePreferences` フックを新規作成する**

`frontend/src/hooks/usePreferences.ts` を新規作成（`hooks` ディレクトリも新規）。デフォルト値は現在 `LobbyPage.tsx:44,61-64` にある初期値をそのまま引き継ぐ（`Player_Web` / `egaroucid` / `7` / `true` / `black`）。

```ts
import { useCallback, useState } from 'react';
import type { GamePreferences } from '../types';

const STORAGE_KEY = 'reversi:preferences';

export const DEFAULT_PREFERENCES: GamePreferences = {
  playerName: 'Player_Web',
  color: 'black',
  aiType: 'egaroucid',
  aiLevel: 7,
  aiUseBook: true,
};

// 保存値は信用できないため、フィールドごとに検証してデフォルトで補完する
const parsePreferences = (raw: unknown): GamePreferences => {
  if (typeof raw !== 'object' || raw === null) return DEFAULT_PREFERENCES;
  const v = raw as Record<string, unknown>;
  return {
    playerName:
      typeof v.playerName === 'string' && v.playerName !== ''
        ? v.playerName
        : DEFAULT_PREFERENCES.playerName,
    color: v.color === 'white' ? 'white' : 'black',
    aiType: v.aiType === 'random' ? 'random' : 'egaroucid',
    aiLevel:
      typeof v.aiLevel === 'number' && Number.isInteger(v.aiLevel) && v.aiLevel >= 0 && v.aiLevel <= 20
        ? v.aiLevel
        : DEFAULT_PREFERENCES.aiLevel,
    aiUseBook: typeof v.aiUseBook === 'boolean' ? v.aiUseBook : DEFAULT_PREFERENCES.aiUseBook,
  };
};

const loadPreferences = (): GamePreferences => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored === null ? DEFAULT_PREFERENCES : parsePreferences(JSON.parse(stored));
  } catch {
    return DEFAULT_PREFERENCES;
  }
};

const savePreferences = (prefs: GamePreferences) => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
  } catch {
    // 保存できない環境ではメモリ上の設定だけで動作させる
  }
};

export const usePreferences = () => {
  const [prefs, setPrefs] = useState(loadPreferences);

  const update = useCallback((patch: Partial<GamePreferences>) => {
    setPrefs((prev) => {
      const next = { ...prev, ...patch };
      savePreferences(next);
      return next;
    });
  }, []);

  return { prefs, update };
};
```

補足: `aiLevel` の範囲 0〜20 は AI 設定モーダルの入力制約（`LobbyModals.tsx:228-229, 293-294` の `min="0" max="20"`）に合わせている。

- [ ] **Step 3: 未使用定数を削除する**

`frontend/src/config/constants.ts` の `DEFAULT_COOLDOWN_SEC` はどこからも参照されていない（`grep -rn "DEFAULT_COOLDOWN_SEC" frontend/src` の結果は定義行のみ）。2行目を削除し、ファイルを次の内容にする。

```ts
export const DEFAULT_TIME_MS = 60000; // 1分
```

- [ ] **Step 4: ビルドと lint を通す**

Run:
```bash
cd frontend && pnpm build && pnpm lint
```
Expected: どちらも成功。`usePreferences` はまだ未使用だが、エクスポートされた関数なので未使用警告は出ない。

- [ ] **Step 5: コミット**

```bash
git add frontend/src/types/index.ts frontend/src/hooks/usePreferences.ts frontend/src/config/constants.ts
git commit -m "add GamePreferences type and usePreferences hook"
```

---

### Task 2: `GameConfig` に `preferences` を持たせる

型を先に確定させる。`GameConfig` を組み立てる 4 箇所（`LobbyPage` 2 + `HistoryPage` 1 + `App` の `defaultConfig`）と、消費する `GamePage` を同時に追随させる。この時点では `LobbyPage` はまだ `useState` で設定を持っており、永続化は Task 3 で入る。

**Files:**
- Modify: `frontend/src/types/index.ts:86-98`
- Modify: `frontend/src/App.tsx:3, 27-32`
- Modify: `frontend/src/components/History/HistoryPage.tsx:1-8, 47-57`
- Modify: `frontend/src/components/Game/GamePage.tsx:40-50, 100-125, 170-180`
- Modify: `frontend/src/components/Lobby/LobbyPage.tsx:96-121, 231-241`

**Interfaces:**
- Consumes: `GamePreferences`, `DEFAULT_PREFERENCES`（Task 1）
- Produces: `GameConfig` が `preferences: GamePreferences` フィールドを持ち、`playerName` / `color` / `aiLevel` / `aiUseBook` を直下に持たなくなる

- [ ] **Step 1: `GameConfig` を書き換える**

`frontend/src/types/index.ts:86-98` の `GameConfig` を次に置き換える。`playerName` / `color` / `aiLevel` / `aiUseBook` は `preferences` の下に移り、ここには対局ごとに決まる値だけが残る。

```ts
// 1対局を開始するための確定パラメータ
export interface GameConfig {
  preferences: GamePreferences;
  mode: MatchMode;
  timeMs: number;
  targetClientId?: string;
  roomId?: string;
  replayMoves?: string[];
  replayBlackName?: string;
  replayWhiteName?: string;
}
```

- [ ] **Step 2: ビルドを実行して壊れた箇所を洗い出す**

Run:
```bash
cd frontend && pnpm build
```
Expected: FAIL。`App.tsx` / `HistoryPage.tsx` / `LobbyPage.tsx` の `GameConfig` 生成箇所と、`GamePage.tsx` の `config.playerName` などの参照でエラーが出る（Step 3〜6 で修正する）。

- [ ] **Step 3: `GamePage.tsx` を `config.preferences` 経由に変更する**

`frontend/src/components/Game/GamePage.tsx` で、`config` から直接読んでいた 4 フィールドを分割代入でまとめて取り出す。`isReplayMode` の定義（44行目付近）の直前に次を追加する。

```ts
  const { playerName, color, aiLevel, aiUseBook } = config.preferences;
```

そのうえで以下を置換する。

- 102-110行目の WebSocket パス組み立て（`config.color` → `color`、`config.aiLevel ?? 3` → `aiLevel`、`config.aiUseBook !== false` → `aiUseBook`）。`aiLevel` / `aiUseBook` が必須になったためフォールバックは不要:

```ts
    if (config.mode === 'vs-human') {
      path = `/ws/vs_human/join/${config.targetClientId}?color=${color}`;
    } else if (config.mode === 'vs-ai') {
      path = `/ws/ai/random?color=${color}`;
    } else if (config.mode === 'vs-ai-egaroucid') {
      path = `/ws/ai/egaroucid?level=${aiLevel}&use_book=${aiUseBook}&color=${color}`;
    } else if (config.mode === 'room') {
      path = `/client/room/${config.roomId || '1'}?color=${color}`;
```

- 122行目: `sendRaw(\`OPEN ${config.playerName}\`)` → `sendRaw(\`OPEN ${playerName}\`)`
- 173行目: `setBlackName(config.playerName)` → `setBlackName(playerName)`
- 178行目: `setWhiteName(config.playerName)` → `setWhiteName(playerName)`

`config.mode` / `config.targetClientId` / `config.roomId` / `config.replayMoves` などモード固有のフィールドは `config` 直下のまま変更しない。

- [ ] **Step 4: `App.tsx` の `defaultConfig` を追随させる**

`frontend/src/App.tsx:27-32` を次に置き換える。

```ts
  const defaultConfig: GameConfig = {
    preferences: DEFAULT_PREFERENCES,
    mode: 'vs-ai-egaroucid',
    timeMs: 86400000,
  };
```

import を追加する（3行目の型 import の下）。

```ts
import { DEFAULT_PREFERENCES } from './hooks/usePreferences';
```

- [ ] **Step 5: `HistoryPage.tsx` の棋譜再生を追随させる**

`frontend/src/components/History/HistoryPage.tsx:48-56` の `onStartGame` 呼び出しを次に置き換える。棋譜再生はプレイヤー名も AI 設定も使わないため、デフォルト値をそのまま渡す。

```ts
    onStartGame({
      preferences: DEFAULT_PREFERENCES,
      mode: 'kifu-replay',
      timeMs: 0,
      replayMoves: moves,
      replayBlackName: blackName,
      replayWhiteName: whiteName,
    });
```

import を追加する。

```ts
import { DEFAULT_PREFERENCES } from '../../hooks/usePreferences';
```

- [ ] **Step 6: `LobbyPage.tsx` の 3 つの `GameConfig` 生成を追随させる**

この時点では `playerName` / `aiType` / `aiLevel` / `aiUseBook` / `playerColor` / `vsHumanColor` はまだローカルの `useState`。`preferences` オブジェクトを組み立てて渡す形にする。Task 3 でここが `prefs` のスプレッドに置き換わる。

`handleConfirmStartVsHuman`（96-108行目）:

```ts
    onStartGame({
      preferences: { playerName, color: vsHumanColor, aiType, aiLevel, aiUseBook },
      mode: 'vs-human',
      targetClientId: target.client_id,
      timeMs: target.assigned_time_ms,
    });
```

`handleStartVsAi`（110-121行目）:

```ts
    onStartGame({
      preferences: { playerName, color: playerColor, aiType, aiLevel, aiUseBook },
      mode: aiType === 'random' ? 'vs-ai' : 'vs-ai-egaroucid',
      timeMs: 86400000,
    });
```

`handleReplayMatch`（231-241行目）:

```ts
  const handleReplayMatch = (moves: string[], blackName: string, whiteName: string) => {
    onStartGame({
      preferences: DEFAULT_PREFERENCES,
      mode: 'kifu-replay',
      timeMs: 0,
      replayMoves: moves,
      replayBlackName: blackName,
      replayWhiteName: whiteName,
    });
  };
```

`DEFAULT_PREFERENCES` の import を追加する（22行目の `DEFAULT_TIME_MS` の import の下）。

```ts
import { DEFAULT_PREFERENCES } from '../../hooks/usePreferences';
```

- [ ] **Step 7: ビルドと lint を通す**

Run:
```bash
cd frontend && pnpm build && pnpm lint
```
Expected: どちらも成功。

- [ ] **Step 8: ブラウザで動作確認する**

Run: `cd frontend && pnpm dev`（サーバ本体が必要なら別途 `cargo run`）。この時点では挙動は変更前と同じはず。確認項目:

1. AI（Egaroucid、レベルを 3 以外に設定）と対局を開始 → DevTools の Network → WS で `level=` が設定した値、`use_book=` が設定通りになっている
2. client と対局を開始でき、手番の選択が反映される
3. ロビーの「最近の対局」から棋譜再生が動く
4. 履歴ページ（`/history`）から棋譜再生が動く
5. `/game` に直接アクセスしてもクラッシュしない（`defaultConfig` が使われる）

- [ ] **Step 9: コミット**

```bash
git add frontend/src/types/index.ts frontend/src/App.tsx frontend/src/components/History/HistoryPage.tsx frontend/src/components/Game/GamePage.tsx frontend/src/components/Lobby/LobbyPage.tsx
git commit -m "hold GamePreferences in GameConfig"
```

---

### Task 3: `LobbyPage` を `usePreferences` に置き換える

このタスクで実際に設定が保存・復元されるようになる。

**Files:**
- Modify: `frontend/src/components/Lobby/LobbyPage.tsx`
- Modify: `frontend/src/components/Lobby/LobbyModals.tsx:41-44, 89-93, 346-370`

**Interfaces:**
- Consumes: `usePreferences`（Task 1）、`GameConfig.preferences`（Task 2）
- Produces: `LobbyModalsProps` から `vsHumanColor` / `onChangeVsHumanColor` が消え、手番選択モーダルは `playerColor` / `onChangePlayerColor` を使う

- [ ] **Step 1: `LobbyModals` から `vsHumanColor` 系 props を削除する**

`frontend/src/components/Lobby/LobbyModals.tsx` の `LobbyModalsProps`（41-44行目）から次の2行を削除する。

```ts
  vsHumanColor: PlayerColor;
  onChangeVsHumanColor: (val: PlayerColor) => void;
```

分割代入（89-93行目）からも `vsHumanColor,` と `onChangeVsHumanColor,` の2行を削除する。

- [ ] **Step 2: 手番選択モーダルを `playerColor` に切り替える**

`frontend/src/components/Lobby/LobbyModals.tsx` の Modal 5（346-370行目付近）で、`vsHumanColor` を `playerColor` に、`onChangeVsHumanColor` を `onChangePlayerColor` に置換する。置換後の `input-group` は次のようになる。

```tsx
              <div className="input-group">
                <label>あなたの手番を選んでください</label>
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
```

- [ ] **Step 3: `LobbyPage` の設定 state を `usePreferences` に置換する**

`frontend/src/components/Lobby/LobbyPage.tsx` で以下の 6 つの `useState` を削除する。

- 44行目 `playerName`
- 54行目 `vsHumanColor`
- 61行目 `aiType`
- 62行目 `aiLevel`
- 63行目 `aiUseBook`
- 64行目 `playerColor`

代わりに 42行目の `const navigate = useNavigate();` の直後に次を追加する。

```ts
  const { prefs, update } = usePreferences();
```

import を `DEFAULT_PREFERENCES` と同じ行にまとめる（Task 2 Step 6 で追加した import を置き換える）。

```ts
import { DEFAULT_PREFERENCES, usePreferences } from '../../hooks/usePreferences';
```

- [ ] **Step 4: 対局開始ハンドラを `prefs` 参照に書き換える**

Task 2 Step 6 で書いた `preferences` の組み立てを `prefs` に置き換える。

`handleConfirmStartVsHuman`（96-108行目）:

```ts
    onStartGame({
      preferences: prefs,
      mode: 'vs-human',
      targetClientId: target.client_id,
      timeMs: target.assigned_time_ms,
    });
```

`handleStartVsAi`（110-121行目）:

```ts
    onStartGame({
      preferences: prefs,
      mode: prefs.aiType === 'random' ? 'vs-ai' : 'vs-ai-egaroucid',
      timeMs: 86400000,
    });
```

`handleReplayMatch` は Task 2 Step 6 のまま（`DEFAULT_PREFERENCES` を使う）で変更しない。棋譜再生は保存済み設定と無関係なため。

- [ ] **Step 5: AI 追加リクエストを `prefs` 参照に書き換える**

`handleAddAiToRoom`（179-198行目）の `body` を次に置き換える。

```ts
        body: JSON.stringify({
          ai_type: prefs.aiType,
          level: prefs.aiLevel,
          use_book: prefs.aiUseBook,
        }),
```

- [ ] **Step 6: JSX の props を `prefs` / `update` に差し替える**

`VsHumanSection`（265-275行目）を次に置き換える。手番は選択のたびにリセットせず前回値を保つため、`setVsHumanColor('black')` は削除する。

```tsx
            <VsHumanSection
              waitingClients={lobbyData.waiting_clients}
              playerName={prefs.playerName}
              onChangePlayerName={(name) => update({ playerName: name })}
              copiedUrlType={copiedUrlType}
              onCopyUrl={copyClientProxyUrl}
              onSelectClient={setSelectedClientForMatch}
            />
```

`LobbyModals`（322-361行目）の該当 props を次に置き換える。`vsHumanColor` / `onChangeVsHumanColor` の2行は削除する。`aiLevel` は空入力時に `parseInt` が `NaN` を返すため `|| 0` で 0 に丸める（`0 || 0` は 0 なのでレベル 0 の入力も壊れない）。

```tsx
        aiType={prefs.aiType}
        onChangeAiType={(val) => update({ aiType: val })}
        aiLevel={prefs.aiLevel}
        onChangeAiLevel={(val) => update({ aiLevel: val || 0 })}
        aiUseBook={prefs.aiUseBook}
        onChangeAiUseBook={(val) => update({ aiUseBook: val })}
        playerColor={prefs.color}
        onChangePlayerColor={(val) => update({ color: val })}
```

- [ ] **Step 7: ビルドと lint を通す**

Run:
```bash
cd frontend && pnpm build && pnpm lint
```
Expected: どちらも成功。失敗する場合は `vsHumanColor` / `playerName` などの旧参照が残っている。

- [ ] **Step 8: ブラウザで動作確認する**

Run: `cd frontend && pnpm dev`。確認項目:

1. 「あなたのプレイヤー名」を `Alice` に変更 → リロード → `Alice` のまま復元される
2. DevTools の Application → Local Storage に `reversi:preferences` があり、`{"playerName":"Alice",...}` が入っている
3. 「AIと対戦」モーダルでレベルを 12、定石オフ、後手(白) に変更して閉じる → リロード → モーダルを開き直すと 12 / オフ / 白 が復元される
4. ルームの「AIを追加」モーダルを開くと、3 で設定したレベル・定石が反映されている
5. Local Storage の `reversi:preferences` の値を `{"playerName":123,"aiLevel":"abc"}` に手で書き換えてリロード → クラッシュせず、名前が `Player_Web`、レベルが 7 に戻る
6. AI と対局を開始でき、対局画面のプレイヤー名が保存した名前になっている
7. client と対局する際の手番選択が、前回選んだ色を保持している

- [ ] **Step 9: コミット**

```bash
git add frontend/src/components/Lobby/LobbyPage.tsx frontend/src/components/Lobby/LobbyModals.tsx
git commit -m "persist lobby preferences in localStorage"
```

---

## Self-Review 結果

**1. 要件カバレッジ**

| 要件 | 対応タスク |
| --- | --- |
| `playerName` を保存し毎回入力不要にする | Task 1（保存機構）+ Task 3 Step 3,6 |
| その他の設定の保存範囲を決める | 「永続化する設定 / しない設定」表、Task 3 |
| 設定と `GameConfig` を並立させない | Task 1 Step 1 + Task 2 Step 1 |
| 継承ではなく合成で表現する | Task 2 Step 1（`preferences` フィールド） |
| デッドコードの削除 | Task 1 Step 3 |
| localStorage が使えない環境で壊れない | Task 1 Step 2 の try/catch、Task 3 Step 8 の項目5 |
| 不正な保存値で壊れない | Task 1 Step 2 の `parsePreferences`、Task 3 Step 8 の項目5 |

**2. プレースホルダ検査:** TBD / TODO / 「適切なエラー処理を追加」の類は無し。全ての実装ステップに実際のコードを記載済み。

**3. 型の整合性:** `GamePreferences` のフィールド名（`playerName` / `color` / `aiType` / `aiLevel` / `aiUseBook`）は Task 1〜3 で一貫。`usePreferences` の戻り値名（`prefs` / `update`）は Task 3 の全ステップで一致。`DEFAULT_PREFERENCES` のエクスポート元（`hooks/usePreferences`）は Task 2・3 の 3 箇所の import で一致。`GameConfig.preferences` というフィールド名は Task 2・3 の全ステップで一致。

**4. 自己レビューで修正した点:**

- 当初 Task 1 で `GameConfig` も同時に変更する構成にしていたが、それだと Task 1 終了時点でビルドが壊れる。型変更を独立したタスクに分離し、各タスク終了時にビルドが通るようにした。
- 当初 `GameConfig extends GamePreferences` としていたが、継承を避けて `preferences` フィールドによる合成に変更した。これに伴い `GamePage` の参照が `config.playerName` → `config.preferences.playerName` に変わるため、Task 2 Step 3 で分割代入して呼び出し側を短く保つようにした。
- 合成にしたことで、棋譜再生時の「`color` だけ黒に上書き」が不要になった（`preferences` 丸ごと `DEFAULT_PREFERENCES` を渡せばよい）ため、`handleReplayMatch` から `color: 'black'` の上書きを削除した。
- `LobbyModals` の `aiLevel` 入力は空文字時に `parseInt` が `NaN` を返す。従来は state に `NaN` が入るだけだったが、永続化すると `JSON.stringify` で `null` になる。`parsePreferences` が次回読み込みでデフォルトに戻すので致命傷にはならないが、その場で表示が壊れるため Task 3 Step 6 で `|| 0` による補正を入れた。

## スコープ外（今回やらないこと）

- ルーム作成のデフォルト値（`newRoomTimeMs` / `newRoomMatchCount`）の永続化。
- テストフレームワークの導入。
