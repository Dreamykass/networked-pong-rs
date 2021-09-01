use crate::input::Input;
use crate::world::World;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Message {
    // message from server to client
    WorldUpdate(World),

    // message from client to server
    UserIdentifier(String),

    // message from client to server
    UserInput(Input),
}
