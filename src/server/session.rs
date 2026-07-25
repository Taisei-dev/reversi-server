use crate::protocol::ServerCommand;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct SessionHandle {
    pub player_name: String,
    pub tx: UnboundedSender<ServerCommand>,
}

impl SessionHandle {
    pub fn new(player_name: impl Into<String>, tx: UnboundedSender<ServerCommand>) -> Self {
        Self {
            player_name: player_name.into(),
            tx,
        }
    }

    pub fn send(&self, cmd: ServerCommand) -> Result<(), String> {
        self.tx
            .send(cmd)
            .map_err(|e| format!("Failed to send command to session: {e}"))
    }

    pub fn is_closed(&self) -> bool {
        self.tx.is_closed()
    }
}

#[derive(Debug)]
pub struct ActivePlayerSession {
    pub handle: SessionHandle,
    pub rx: tokio::sync::mpsc::UnboundedReceiver<crate::protocol::ClientCommand>,
}

impl ActivePlayerSession {
    pub fn is_closed(&self) -> bool {
        self.handle.is_closed() || self.rx.is_closed()
    }
}
