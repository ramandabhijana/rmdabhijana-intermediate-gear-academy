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
    CheckGameStatus { user: ActorId },
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub enum GameOverStatus {
    Win,
    Lose,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub enum GameStatus {
    Idle,
    InProgress,
    Completed(GameOverStatus),
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
        status: GameOverStatus,
    },
    MessageSent,
}

type SentMessageId = MessageId;
type OriginalMessageId = MessageId;

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct PlayerInfo {
    pub game_status: GameStatus,
    pub attempts_count: u32,
    pub msg_ids: (SentMessageId, OriginalMessageId),
}

impl PlayerInfo {
    pub fn new(msg_ids: (SentMessageId, OriginalMessageId)) -> Self {
        Self {
            game_status: GameStatus::Idle,
            attempts_count: 0,
            msg_ids,
        }
    }

    pub fn start_game(&mut self) {
        assert_eq!(self.game_status, GameStatus::Idle);
        self.game_status = GameStatus::InProgress;
    }
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct State {
    pub target_program_id: ActorId,
    pub players: BTreeMap<ActorId, PlayerInfo>,
}
