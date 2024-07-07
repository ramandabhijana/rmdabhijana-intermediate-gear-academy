#![no_std]
use gmeta::{InOut, Metadata, Out};
use gstd::{collections::BTreeMap, prelude::*, ActorId, MessageId};

pub struct SessionMetadata;
impl Metadata for SessionMetadata {
    type Init = ();
    type Handle = InOut<Action, Event>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = Out<State>;
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum Action {
    StartGame { user: ActorId },
    CheckWord { user: ActorId, word: String },
    CheckGameStatus,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum GameStatus {
    Win,
    Lose,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum Event {
    GameStarted {
        user: ActorId,
    },
    WordChecked {
        user: ActorId,
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
    },
    GameOver {
        user: ActorId,
        status: GameStatus,
    },
    MessageSent,
}

type SentMessageId = MessageId;
type OriginalMessageId = MessageId;

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct PlayerInfo {
    game_status: Option<GameStatus>,
    attempts_count: u32,
    msg_ids: (SentMessageId, OriginalMessageId),
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct State {
    pub target_program_id: ActorId,
    pub players: BTreeMap<ActorId, PlayerInfo>,
}
