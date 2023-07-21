use crate::server::{OpenConnection, ConnectionKind, ClientConnectionStage, ServerConnectionStage};

pub fn parse_message(
    connection: &mut OpenConnection,
    plaintext: &str,
) {
    // Break the incoming message into lines
    let mut lines = plaintext.split('\n');
    
    match connection.kind() {
        ConnectionKind::Unknown => {
            
        },
        ConnectionKind::GameClient(stage) => {
            match stage {
                ClientConnectionStage::WaitingOnServer => {

                },
            }
        },
        ConnectionKind::GameServer(stage) => {
            match stage {
                ServerConnectionStage::ReplaceMe => {
                    
                },
            }
        },
    }
}