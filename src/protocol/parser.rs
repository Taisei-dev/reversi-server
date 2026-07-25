use super::proto::ClientCommand;
use crate::game::Move;

pub fn parse_client_command(line: &str) -> Result<ClientCommand, String> {
    let line = line.trim();
    if line.is_empty() {
        return Err("Empty line".into());
    }

    let mut parts = line.split_whitespace();
    let cmd = parts.next().ok_or("Missing command")?;

    match cmd.to_uppercase().as_str() {
        "OPEN" => {
            let name = parts.next().unwrap_or("Anon").to_string();
            Ok(ClientCommand::Open { player_name: name })
        }
        "MOVE" => {
            let move_str = parts.next().ok_or("Missing move argument")?;
            let m = Move::from_str(move_str)
                .ok_or_else(|| format!("Invalid move argument: {move_str}"))?;
            Ok(ClientCommand::Move(m))
        }
        other => Err(format!("Unknown command: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_open() {
        let cmd = parse_client_command("OPEN Player1\n").unwrap();
        assert_eq!(
            cmd,
            ClientCommand::Open {
                player_name: "Player1".to_string()
            }
        );
    }

    #[test]
    fn test_parse_move() {
        let cmd = parse_client_command("MOVE D3").unwrap();
        assert_eq!(cmd, ClientCommand::Move(Move::Coord { x: 4, y: 3 }));
    }
}
