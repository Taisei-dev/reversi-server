import React, { useState, useEffect, useRef } from 'react';
import type { GameConfig, BoardState } from '../../types';
import { ArrowLeft, Copy, Check } from 'lucide-react';

interface GamePageProps {
  config: GameConfig;
  onBackToLobby: () => void;
}

export const GamePage: React.FC<GamePageProps> = ({ config, onBackToLobby }) => {
  const [boardHistory, setBoardHistory] = useState<BoardState[]>([]);
  const [kifuMoves, setKifuMoves] = useState<string[]>([]);
  const [currentViewStep, setCurrentViewStep] = useState(0);

  const [myColor, setMyColor] = useState<number | null>(null); // 1: Black, 2: White
  const [currentTurn, setCurrentTurn] = useState<number>(1); // 1: Black, 2: White
  const [isGameActive, setIsGameActive] = useState(false);
  const [isWaitingAck, setIsWaitingAck] = useState(false);

  const [statusMsg, setStatusMsg] = useState('接続待機中...');
  const [logs, setLogs] = useState<{ time: string; msg: string; type: string }[]>([]);

  const [blackName, setBlackName] = useState('Black');
  const [whiteName, setWhiteName] = useState('White');

  const [isCopied, setIsCopied] = useState(false);

  const [endModal, setEndModal] = useState<{
    show: boolean;
    result: string;
    yourStones: number;
    oppStones: number;
    reason: string;
  }>({
    show: false,
    result: '',
    yourStones: 0,
    oppStones: 0,
    reason: '',
  });

  const wsRef = useRef<WebSocket | null>(null);
  const logConsoleRef = useRef<HTMLDivElement | null>(null);
  const isReplayMode = config.mode === 'kifu-replay';
  const { playerName, color, aiLevel, aiUseBook } = config.preferences;

  const addLog = (msg: string, type: 'sys' | 'in' | 'out' | 'err' = 'sys') => {
    const time = new Date().toLocaleTimeString();
    setLogs((prev) => [...prev, { time, msg, type }]);
  };

  useEffect(() => {
    if (logConsoleRef.current) {
      logConsoleRef.current.scrollTop = logConsoleRef.current.scrollHeight;
    }
  }, [logs]);

  const createInitialBoard = (): BoardState => {
    const b = Array(8).fill(null).map(() => Array(8).fill(0));
    b[3][3] = 2; // 白
    b[4][4] = 2; // 白
    b[3][4] = 1; // 黒
    b[4][3] = 1; // 黒
    return b;
  };

  const cloneBoard = (b: BoardState): BoardState => b.map((row) => [...row]);

  useEffect(() => {
    const initBoard = createInitialBoard();

    // 棋譜再生モード
    if (isReplayMode && config.replayMoves) {
      setBlackName(config.replayBlackName || 'Black');
      setWhiteName(config.replayWhiteName || 'White');
      setStatusMsg('棋譜再生モード');
      setIsGameActive(false);

      let history: BoardState[] = [initBoard];
      let cur = initBoard;
      let turn = 1;

      config.replayMoves.forEach((m) => {
        cur = applyMoveToBoard(cur, m, turn);
        history.push(cur);
        turn = turn === 1 ? 2 : 1;
      });

      setBoardHistory(history);
      setKifuMoves(config.replayMoves);
      setCurrentViewStep(history.length - 1);
      return;
    }

    setBoardHistory([initBoard]);
    setKifuMoves([]);
    setCurrentViewStep(0);

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;

    let path = '';
    if (config.mode === 'vs-human') {
      path = `/ws/vs_human/join/${config.targetClientId}?color=${color}`;
    } else if (config.mode === 'vs-ai') {
      path = `/ws/ai/random?color=${color}`;
    } else if (config.mode === 'vs-ai-egaroucid') {
      path = `/ws/ai/egaroucid?level=${aiLevel}&use_book=${aiUseBook}&color=${color}`;
    } else if (config.mode === 'room') {
      path = `/client/room/${config.roomId || '1'}?color=${color}`;
    }

    const url = `${protocol}//${host}${path}`;
    addLog(`サーバー接続試行: ${url}`, 'sys');

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setStatusMsg('接続完了 (対戦待機)');
      addLog('WebSocket 接続成功', 'sys');
      sendRaw(`OPEN ${playerName}`);
    };

    ws.onmessage = (event) => {
      const text = event.data.trim();
      addLog(`← ${text}`, 'in');
      handleServerMessage(text);
    };

    ws.onclose = () => {
      setStatusMsg('切断されました');
      setIsGameActive(false);
      addLog('WebSocket 切断', 'sys');
    };

    ws.onerror = () => {
      addLog('WebSocket エラー', 'err');
    };

    return () => {
      ws.close();
    };
  }, []);

  const sendRaw = (cmd: string) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      addLog(`→ ${cmd}`, 'out');
      wsRef.current.send(cmd + '\n');
    }
  };

  const handleServerMessage = (msgText: string) => {
    const parts = msgText.split(' ');
    const cmd = parts[0].toUpperCase();

    if (cmd === 'START') {
      const colorStr = parts[1];
      const oppName = parts[2];

      const assignedColor = colorStr === 'BLACK' ? 1 : 2;
      setMyColor(assignedColor);
      setCurrentTurn(1); // 1 (Black) から開始
      setIsGameActive(true);
      setIsWaitingAck(false);

      const initBoard = createInitialBoard();
      setBoardHistory([initBoard]);
      setKifuMoves([]);
      setCurrentViewStep(0);

      if (assignedColor === 1) {
        setBlackName(playerName);
        setWhiteName(oppName);
        setStatusMsg('あなたの番です (黒/先手)');
      } else {
        setBlackName(oppName);
        setWhiteName(playerName);
        setStatusMsg('相手の思考中... (白/後手)');
      }

      addLog(`対戦開始! あなたは ${colorStr} です。対戦相手: ${oppName}`, 'sys');
    } else if (cmd === 'MOVE') {
      const posStr = parts[1];
      setCurrentTurn((prevTurn) => {
        recordMove(posStr, prevTurn);
        const nextTurn = prevTurn === 1 ? 2 : 1;
        setIsWaitingAck(false);

        setMyColor((myCol) => {
          if (myCol === nextTurn) {
            setStatusMsg('あなたの番です');
          } else {
            setStatusMsg('相手の思考中...');
          }
          return myCol;
        });

        return nextTurn;
      });
    } else if (cmd === 'ACK') {
      setIsWaitingAck(false);
    } else if (cmd === 'END') {
      const result = parts[1];
      const yourStones = parseInt(parts[2], 10) || 0;
      const oppStones = parseInt(parts[3], 10) || 0;
      const reason = parts[4] || 'OK';

      setIsGameActive(false);
      setIsWaitingAck(false);
      setStatusMsg(`対戦終了: ${result}`);
      setEndModal({
        show: true,
        result,
        yourStones,
        oppStones,
        reason,
      });
    }
  };

  const recordMove = (posStr: string, turnColor: number) => {
    setKifuMoves((prev) => [...prev, posStr]);

    setBoardHistory((prevHistory) => {
      const latestBoard = prevHistory[prevHistory.length - 1];
      const newBoard = applyMoveToBoard(latestBoard, posStr, turnColor);
      const nextHistory = [...prevHistory, newBoard];

      setCurrentViewStep((prevStep) => {
        if (prevStep === prevHistory.length - 1 || prevStep === 0) {
          return nextHistory.length - 1;
        }
        return prevStep;
      });

      return nextHistory;
    });
  };

  const applyMoveToBoard = (board: BoardState, posStr: string, color: number): BoardState => {
    const newBoard = cloneBoard(board);
    posStr = posStr.toUpperCase();
    if (posStr === 'PASS' || posStr === 'GIVEUP' || posStr.length !== 2) return newBoard;

    const x = posStr.charCodeAt(0) - 65;
    const y = posStr.charCodeAt(1) - 49;
    const opp = color === 1 ? 2 : 1;
    const dirs = [[-1,-1],[0,-1],[1,-1],[-1,0],[1,0],[-1,1],[0,1],[1,1]];

    let flippables: { x: number; y: number }[] = [];
    for (const [dx, dy] of dirs) {
      let line: { x: number; y: number }[] = [];
      let cx = x + dx;
      let cy = y + dy;
      while (cx >= 0 && cx < 8 && cy >= 0 && cy < 8 && newBoard[cy][cx] === opp) {
        line.push({ x: cx, y: cy });
        cx += dx;
        cy += dy;
      }
      if (cx >= 0 && cx < 8 && cy >= 0 && cy < 8 && newBoard[cy][cx] === color) {
        flippables.push(...line);
      }
    }

    newBoard[y][x] = color;
    for (const p of flippables) {
      newBoard[p.y][p.x] = color;
    }
    return newBoard;
  };

  const getValidMoves = (board: BoardState, color: number) => {
    const opp = color === 1 ? 2 : 1;
    const dirs = [[-1,-1],[0,-1],[1,-1],[-1,0],[1,0],[-1,1],[0,1],[1,1]];
    const moves: { x: number; y: number }[] = [];

    for (let y = 0; y < 8; y++) {
      for (let x = 0; x < 8; x++) {
        if (board[y][x] !== 0) continue;
        let canFlip = false;
        for (const [dx, dy] of dirs) {
          let cx = x + dx;
          let cy = y + dy;
          let count = 0;
          while (cx >= 0 && cx < 8 && cy >= 0 && cy < 8 && board[cy][cx] === opp) {
            count++;
            cx += dx;
            cy += dy;
          }
          if (count > 0 && cx >= 0 && cx < 8 && cy >= 0 && cy < 8 && board[cy][cx] === color) {
            canFlip = true;
            break;
          }
        }
        if (canFlip) moves.push({ x, y });
      }
    }
    return moves;
  };

  const onCellClick = (x: number, y: number) => {
    if (!isGameActive || isWaitingAck || myColor !== currentTurn) return;

    const latestBoard = boardHistory[boardHistory.length - 1];
    const validMoves = getValidMoves(latestBoard, myColor);

    if (!validMoves.some((m) => m.x === x && m.y === y)) {
      addLog('そこには置けません', 'err');
      return;
    }

    const colChar = String.fromCharCode(65 + x);
    const rowChar = String.fromCharCode(49 + y);
    const moveStr = `${colChar}${rowChar}`;

    recordMove(moveStr, myColor);
    setIsWaitingAck(true);
    setCurrentTurn(myColor === 1 ? 2 : 1);
    setStatusMsg('相手の思考中...');
    sendRaw(`MOVE ${moveStr}`);
  };

  const copyKifu = () => {
    const validMoves = kifuMoves.filter(
      (m) => m.toUpperCase() !== 'PASS' && m.toUpperCase() !== 'GIVEUP'
    );
    if (validMoves.length === 0) return;
    const formattedKifu = validMoves.map((m) => m.toLowerCase()).join('');
    navigator.clipboard.writeText(formattedKifu).then(() => {
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    });
  };

  const currentBoard = boardHistory[currentViewStep] || createInitialBoard();
  const isLatestStep = currentViewStep === boardHistory.length - 1;
  const validMovesForCell = isLatestStep && isGameActive && !isWaitingAck && myColor === currentTurn
    ? getValidMoves(currentBoard, myColor)
    : [];

  const isMustPass = isLatestStep && isGameActive && !isWaitingAck && myColor === currentTurn && validMovesForCell.length === 0;

  const handleSendPass = () => {
    if (!isGameActive || isWaitingAck || myColor !== currentTurn) return;
    recordMove('PASS', myColor);
    setIsWaitingAck(true);
    setCurrentTurn(myColor === 1 ? 2 : 1);
    setStatusMsg('相手の思考中...');
    sendRaw('MOVE PASS');
  };

  let lastMovePos: { x: number; y: number } | null = null;
  if (currentViewStep > 0 && kifuMoves[currentViewStep - 1]) {
    const moveStr = kifuMoves[currentViewStep - 1].toUpperCase();
    if (moveStr !== 'PASS' && moveStr !== 'GIVEUP' && moveStr.length === 2) {
      lastMovePos = {
        x: moveStr.charCodeAt(0) - 65,
        y: moveStr.charCodeAt(1) - 49,
      };
    }
  }

  let blackStones = 0;
  let whiteStones = 0;
  for (let y = 0; y < 8; y++) {
    for (let x = 0; x < 8; x++) {
      if (currentBoard[y][x] === 1) blackStones++;
      if (currentBoard[y][x] === 2) whiteStones++;
    }
  }

  const isBlackActive = !isReplayMode && isGameActive && currentTurn === 1;
  const isWhiteActive = !isReplayMode && isGameActive && currentTurn === 2;

  return (
    <div className="game-container">
      <header className="app-header">
        <button className="btn btn-outline btn-small" onClick={onBackToLobby}>
          <ArrowLeft size={14} />
          <span>ロビーへ戻る</span>
        </button>
      </header>

      <main className={`game-main game-layout-split ${isReplayMode ? 'replay-full-width' : ''}`}>
        <div className="game-board-section">
          
          <div className="match-status-panel-vertical">
            <div className="status-header-text">{statusMsg}</div>
            
            <div className={`player-row-pill black ${isBlackActive ? 'active-turn' : ''}`}>
              <div className="p-left">
                <span className="disc-dot-large black"></span>
                <span className="p-name">{blackName}</span>
                {!isReplayMode && myColor === 1 && <span className="you-badge">(You)</span>}
              </div>
              <span className="p-score-huge">{blackStones}</span>
            </div>

            <div className={`player-row-pill white ${isWhiteActive ? 'active-turn' : ''}`}>
              <div className="p-left">
                <span className="disc-dot-large white"></span>
                <span className="p-name">{whiteName}</span>
                {!isReplayMode && myColor === 2 && <span className="you-badge">(You)</span>}
              </div>
              <span className="p-score-huge">{whiteStones}</span>
            </div>
          </div>

          <div className="board-wrapper">
            <div className="board-labels cols">
              {['A','B','C','D','E','F','G','H'].map((c) => (
                <span key={c}>{c}</span>
              ))}
            </div>
            <div className="board-middle">
              <div className="board-labels rows">
                {[1,2,3,4,5,6,7,8].map((r) => (
                  <span key={r}>{r}</span>
                ))}
              </div>
              <div className="board-grid">
                {currentBoard.map((row, y) =>
                  row.map((val, x) => {
                    const isValid = validMovesForCell.some((m) => m.x === x && m.y === y);
                    const isLastMove = lastMovePos?.x === x && lastMovePos?.y === y;
                    return (
                      <div
                        key={`${x}-${y}`}
                        className={`cell ${isValid ? 'valid-move' : ''}`}
                        onClick={() => onCellClick(x, y)}
                      >
                        {val === 1 && (
                          <div className={`disc black ${isLastMove ? 'last-move-disc' : ''}`}>
                            {isLastMove && <span className="last-move-dot" />}
                          </div>
                        )}
                        {val === 2 && (
                          <div className={`disc white ${isLastMove ? 'last-move-disc' : ''}`}>
                            {isLastMove && <span className="last-move-dot" />}
                          </div>
                        )}
                      </div>
                    );
                  })
                )}
              </div>
            </div>
          </div>

          <div className="playback-controls">
            <div className="kifu-controls">
              <button className="btn btn-small btn-outline" onClick={copyKifu}>
                {isCopied ? <Check size={14} /> : <Copy size={14} />}
                <span>{isCopied ? 'コピー完了 (d3c5...)' : '棋譜をコピー'}</span>
              </button>
              <span className="step-indicator">
                {currentViewStep} / {Math.max(0, boardHistory.length - 1)} 手
              </span>
              {!isLatestStep && (
                <button
                  className="btn btn-small btn-outline"
                  onClick={() => setCurrentViewStep(boardHistory.length - 1)}
                >
                  <span>最新手へ</span>
                </button>
              )}
            </div>
            <div className="seekbar-wrapper">
              <input
                type="range"
                min={0}
                max={Math.max(0, boardHistory.length - 1)}
                value={currentViewStep}
                onChange={(e) => setCurrentViewStep(parseInt(e.target.value, 10))}
              />
            </div>
          </div>
        </div>

        {!isReplayMode && (
          <aside className="game-sidebar-log">
            <div className="card log-card">
              <h2>通信ログ</h2>
              <div className="log-console-sidebar" ref={logConsoleRef}>
                {logs.map((l, i) => (
                  <div key={i} className={`log-entry ${l.type}`}>
                    [{l.time}] {l.msg}
                  </div>
                ))}
              </div>
            </div>
          </aside>
        )}
      </main>

      {isMustPass && (
        <div className="modal-overlay">
          <div className="modal-card pass-modal-card">
            <h3>パスが必要です</h3>
            <p style={{ margin: '1rem 0', color: 'var(--text-secondary)' }}>
              現在、あなた（{myColor === 1 ? '黒/先手' : '白/後手'}）の打てる有効手がありません。
            </p>
            <button className="btn btn-primary btn-large" onClick={handleSendPass} style={{ width: '100%', marginTop: '0.5rem' }}>
              パスを送信する (MOVE PASS)
            </button>
          </div>
        </div>
      )}

      {endModal.show && (
        <div className="modal-overlay">
          <div className="modal-card">
            <h3>対局終了</h3>
            <div className="modal-result-text">{endModal.result}</div>
            <div className="modal-scores">
              黒 {myColor === 1 ? endModal.yourStones : endModal.oppStones} - 白{' '}
              {myColor === 2 ? endModal.yourStones : endModal.oppStones}
            </div>
            <div className="modal-reason">Reason: {endModal.reason}</div>
            <button className="btn btn-primary" onClick={() => setEndModal((p) => ({ ...p, show: false }))}>
              閉じる
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
