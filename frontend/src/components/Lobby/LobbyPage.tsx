import React, { useState, useEffect } from 'react';
import type {
  LobbyData,
  HistoryData,
  GameConfig,
  PlayerColor,
  TournamentMatchDetail,
  WaitingClientInfo,
  RoomInfo,
} from '../../types';
import { RefreshCw } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

import { ProxyGuideBanner } from './ProxyGuideBanner';
import { FreematchSection } from './FreematchSection';
import { RoomListSection } from './RoomListSection';
import { VsHumanSection } from './VsHumanSection';
import { VsAiSection } from './VsAiSection';
import { ActiveMatchesSection } from './ActiveMatchesSection';
import { RecentHistorySection } from './RecentHistorySection';
import { LobbyModals } from './LobbyModals';
import { DEFAULT_TIME_MS } from '../../config/constants';
import { DEFAULT_PREFERENCES } from '../../hooks/usePreferences';

interface LobbyPageProps {
  onStartGame: (config: GameConfig) => void;
}

export const LobbyPage: React.FC<LobbyPageProps> = ({ onStartGame }) => {
  const [lobbyData, setLobbyData] = useState<LobbyData>({
    open_rooms: [],
    waiting_clients: [],
    freematch_clients: [],
    active_matches: [],
  });
  const [historyData, setHistoryData] = useState<HistoryData>({
    recent_matches: [],
    tournament_results: [],
  });
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [copiedUrlType, setCopiedUrlType] = useState<string | null>(null);

  const navigate = useNavigate();

  const [playerName, setPlayerName] = useState('Player_Web');
  const [showCreateRoomModal, setShowCreateRoomModal] = useState(false);
  const [editingRoom, setEditingRoom] = useState<RoomInfo | null>(null);
  const [editMatchCount, setEditMatchCount] = useState(1);
  const [editTimeMs, setEditTimeMs] = useState(1000);

  const [showAddAiModal, setShowAddAiModal] = useState<string | null>(null);
  const [showStartAiModal, setShowStartAiModal] = useState(false);

  const [selectedClientForMatch, setSelectedClientForMatch] = useState<WaitingClientInfo | null>(null);
  const [vsHumanColor, setVsHumanColor] = useState<PlayerColor>('black');
  const [selectedCellMatches, setSelectedCellMatches] = useState<TournamentMatchDetail[] | null>(null);
  const [expandedTournaments, setExpandedTournaments] = useState<{ [key: number]: boolean }>({});
  const [newRoomId, setNewRoomId] = useState('1');
  const [newRoomTimeMs, setNewRoomTimeMs] = useState(DEFAULT_TIME_MS);
  const [newRoomMatchCount, setNewRoomMatchCount] = useState(1);

  const [aiType, setAiType] = useState<'random' | 'egaroucid'>('egaroucid');
  const [aiLevel, setAiLevel] = useState(7);
  const [aiUseBook, setAiUseBook] = useState(true);
  const [playerColor, setPlayerColor] = useState<PlayerColor>('black');

  const fetchLobbyAndHistory = async () => {
    setIsRefreshing(true);
    try {
      const [resLobby, resHist] = await Promise.all([
        fetch('/api/lobby'),
        fetch('/api/history?limit=3'),
      ]);
      if (resLobby.ok) setLobbyData(await resLobby.json());
      if (resHist.ok) setHistoryData(await resHist.json());
    } catch (e) {
      console.error('Failed to fetch lobby data', e);
    } finally {
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    fetchLobbyAndHistory();
    const interval = setInterval(fetchLobbyAndHistory, 2500);
    return () => clearInterval(interval);
  }, []);

  const copyClientProxyUrl = (path: string, label: string) => {
    const host = window.location.host;
    navigator.clipboard.writeText(`${host}${path}`).then(() => {
      setCopiedUrlType(label);
      setTimeout(() => setCopiedUrlType(null), 2000);
    });
  };

  const handleConfirmStartVsHuman = (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedClientForMatch) return;
    const target = selectedClientForMatch;
    setSelectedClientForMatch(null);
    onStartGame({
      preferences: { playerName, color: vsHumanColor, aiType, aiLevel, aiUseBook },
      mode: 'vs-human',
      targetClientId: target.client_id,
      timeMs: target.assigned_time_ms,
    });
  };

  const handleStartVsAi = (e: React.FormEvent) => {
    e.preventDefault();
    setShowStartAiModal(false);
    onStartGame({
      preferences: { playerName, color: playerColor, aiType, aiLevel, aiUseBook },
      mode: aiType === 'random' ? 'vs-ai' : 'vs-ai-egaroucid',
      timeMs: 86400000,
    });
  };

  const handleCreateRoom = async (e: React.FormEvent) => {
    e.preventDefault();
    setShowCreateRoomModal(false);
    try {
      const res = await fetch('/api/rooms', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          room_id: newRoomId,
          time_ms: newRoomTimeMs,
          match_count: newRoomMatchCount,
        }),
      });
      if (res.ok) fetchLobbyAndHistory();
    } catch (e) {
      console.error('Failed to create room', e);
    }
  };

  const handleDeleteRoom = async (roomId: string) => {
    try {
      const res = await fetch(`/api/rooms/${roomId}/delete`, {
        method: 'POST',
      });
      if (res.ok) fetchLobbyAndHistory();
    } catch (e) {
      console.error('Failed to delete room', e);
    }
  };

  const handleOpenEditRoomModal = (room: RoomInfo) => {
    setEditingRoom(room);
    setEditMatchCount(room.match_count);
    setEditTimeMs(room.assigned_time_ms);
  };

  const handleEditRoomSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingRoom) return;
    const targetRoomId = editingRoom.room_id;
    setEditingRoom(null);
    try {
      const res = await fetch(`/api/rooms/${targetRoomId}/update`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          match_count: editMatchCount,
          time_ms: editTimeMs,
        }),
      });
      if (res.ok) fetchLobbyAndHistory();
    } catch (e) {
      console.error('Failed to update room', e);
    }
  };

  const handleAddAiToRoom = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!showAddAiModal) return;
    const targetRoomId = showAddAiModal;
    setShowAddAiModal(null);
    try {
      const res = await fetch(`/api/rooms/${targetRoomId}/add_ai`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          ai_type: aiType,
          level: aiLevel,
          use_book: aiUseBook,
        }),
      });
      if (res.ok) fetchLobbyAndHistory();
    } catch (e) {
      console.error('Failed to add AI', e);
    }
  };

  const handleRemoveAiFromRoom = async (roomId: string, aiName: string) => {
    try {
      const res = await fetch(`/api/rooms/${roomId}/remove_ai`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ai_name: aiName }),
      });
      if (res.ok) fetchLobbyAndHistory();
    } catch (e) {
      console.error('Failed to remove AI', e);
    }
  };

  const handleStartTournament = async (roomId: string) => {
    try {
      const res = await fetch(`/api/rooms/${roomId}/start`, {
        method: 'POST',
      });
      if (res.ok) fetchLobbyAndHistory();
    } catch (e) {
      console.error('Failed to start tournament', e);
    }
  };

  const toggleTournamentMatrix = (idx: number) => {
    setExpandedTournaments((prev) => ({
      ...prev,
      [idx]: !prev[idx],
    }));
  };

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

  return (
    <div className="lobby-container">
      <header className="app-header">
        <div className="logo">
          <span>⚫⚪ Reversi Server Lobby</span>
        </div>
        <button
          className="btn btn-outline btn-small"
          onClick={fetchLobbyAndHistory}
          disabled={isRefreshing}
        >
          <RefreshCw size={14} className={isRefreshing ? 'animate-spin' : ''} />
          <span>更新</span>
        </button>
      </header>

      <ProxyGuideBanner />

      <main className="lobby-main">
        <div className="lobby-three-column-layout">
          {/* Column 1: 1vs1 対局 */}
          <div className="lobby-column">
            <VsHumanSection
              waitingClients={lobbyData.waiting_clients}
              playerName={playerName}
              onChangePlayerName={setPlayerName}
              copiedUrlType={copiedUrlType}
              onCopyUrl={copyClientProxyUrl}
              onSelectClient={(client) => {
                setSelectedClientForMatch(client);
                setVsHumanColor('black');
              }}
            />

            <VsAiSection
              copiedUrlType={copiedUrlType}
              onCopyUrl={copyClientProxyUrl}
              onOpenStartAiModal={() => setShowStartAiModal(true)}
            />
          </div>

          {/* Column 2: 自動マッチ & ルーム一覧 */}
          <div className="lobby-column">
            <FreematchSection
              freematchClients={lobbyData.freematch_clients}
              copiedUrlType={copiedUrlType}
              onCopyUrl={copyClientProxyUrl}
            />

            <RoomListSection
              openRooms={lobbyData.open_rooms}
              copiedUrlType={copiedUrlType}
              onCopyUrl={copyClientProxyUrl}
              onOpenCreateRoomModal={() => setShowCreateRoomModal(true)}
              onOpenEditRoomModal={handleOpenEditRoomModal}
              onDeleteRoom={handleDeleteRoom}
              onOpenAddAiModal={(roomId) => setShowAddAiModal(roomId)}
              onRemoveAi={handleRemoveAiFromRoom}
              onStartTournament={handleStartTournament}
            />
          </div>

          {/* Column 3: リアルタイム進行 & 観戦・結果 */}
          <div className="lobby-column">
            <ActiveMatchesSection activeMatches={lobbyData.active_matches} />

            <RecentHistorySection
              recentMatches={historyData.recent_matches}
              recentTournaments={historyData.tournament_results}
              expandedTournaments={expandedTournaments}
              onToggleMatrix={toggleTournamentMatrix}
              onSelectCellMatches={setSelectedCellMatches}
              onReplayMatch={handleReplayMatch}
              onNavigateHistory={() => navigate('/history')}
            />
          </div>
        </div>
      </main>

      <LobbyModals
        showCreateRoomModal={showCreateRoomModal}
        onCloseCreateRoomModal={() => setShowCreateRoomModal(false)}
        onCreateRoomSubmit={handleCreateRoom}
        newRoomId={newRoomId}
        onChangeNewRoomId={setNewRoomId}
        newRoomMatchCount={newRoomMatchCount}
        onChangeNewRoomMatchCount={setNewRoomMatchCount}
        newRoomTimeMs={newRoomTimeMs}
        onChangeNewRoomTimeMs={setNewRoomTimeMs}
        editingRoom={editingRoom}
        onCloseEditRoomModal={() => setEditingRoom(null)}
        onEditRoomSubmit={handleEditRoomSubmit}
        editMatchCount={editMatchCount}
        onChangeEditMatchCount={setEditMatchCount}
        editTimeMs={editTimeMs}
        onChangeEditTimeMs={setEditTimeMs}
        showAddAiModal={showAddAiModal}
        onCloseAddAiModal={() => setShowAddAiModal(null)}
        onAddAiSubmit={handleAddAiToRoom}
        showStartAiModal={showStartAiModal}
        onCloseStartAiModal={() => setShowStartAiModal(false)}
        onStartAiSubmit={handleStartVsAi}
        aiType={aiType}
        onChangeAiType={setAiType}
        aiLevel={aiLevel}
        onChangeAiLevel={setAiLevel}
        aiUseBook={aiUseBook}
        onChangeAiUseBook={setAiUseBook}
        playerColor={playerColor}
        onChangePlayerColor={setPlayerColor}
        selectedClientForMatch={selectedClientForMatch}
        onCloseVsHumanModal={() => setSelectedClientForMatch(null)}
        onConfirmStartVsHuman={handleConfirmStartVsHuman}
        vsHumanColor={vsHumanColor}
        onChangeVsHumanColor={setVsHumanColor}
        selectedCellMatches={selectedCellMatches}
        onCloseCellMatchesModal={() => setSelectedCellMatches(null)}
        onReplayMatch={handleReplayMatch}
      />
    </div>
  );
};
