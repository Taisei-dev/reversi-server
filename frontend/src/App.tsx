import React, { useState } from 'react';
import { BrowserRouter, Routes, Route, useNavigate } from 'react-router-dom';
import type { GameConfig } from './types';
import { DEFAULT_PREFERENCES } from './hooks/usePreferences';
import { LobbyPage } from './components/Lobby/LobbyPage';
import { GamePage } from './components/Game/GamePage';
import { HistoryPage } from './components/History/HistoryPage';
import './App.css';

const MainRoutes: React.FC = () => {
  const [activeConfig, setActiveConfig] = useState<GameConfig | null>(null);
  const navigate = useNavigate();

  const handleStartGame = (config: GameConfig) => {
    setActiveConfig(config);
    if (config.mode === 'kifu-replay') {
      navigate('/replay');
    } else {
      navigate('/game');
    }
  };

  const handleBackToLobby = () => {
    setActiveConfig(null);
    navigate('/');
  };

  const defaultConfig: GameConfig = {
    preferences: DEFAULT_PREFERENCES,
    mode: 'vs-ai-egaroucid',
    timeMs: 86400000,
  };

  return (
    <Routes>
      <Route
        path="/"
        element={<LobbyPage onStartGame={handleStartGame} />}
      />
      <Route
        path="/history"
        element={<HistoryPage onStartGame={handleStartGame} />}
      />
      <Route
        path="/game"
        element={
          <GamePage
            config={activeConfig || defaultConfig}
            onBackToLobby={handleBackToLobby}
          />
        }
      />
      <Route
        path="/replay"
        element={
          <GamePage
            config={activeConfig || defaultConfig}
            onBackToLobby={handleBackToLobby}
          />
        }
      />
    </Routes>
  );
};

export const App: React.FC = () => {
  return (
    <BrowserRouter>
      <div className="app-root">
        <MainRoutes />
      </div>
    </BrowserRouter>
  );
};

export default App;
