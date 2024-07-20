#![no_std]
use gmeta::{InOut, Metadata, Out};
use gstd::{collections::BTreeMap, prelude::*, ActorId, MessageId};
use wordle_io::{Event as WordleEvent, WORD_LENGTH};

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
    Starting,
    Started,
    CheckingWord,
    WordChecked(bool),
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
    msg_ids: (SentMessageId, OriginalMessageId),
}

impl PlayerInfo {
    pub fn new(sent_msg_id: SentMessageId, original_msg_id: OriginalMessageId) -> Self {
        Self {
            game_status: GameStatus::Idle,
            attempts_count: 0,
            msg_ids: (sent_msg_id, original_msg_id),
        }
    }

    pub fn start_game(&mut self) {
        assert_eq!(self.game_status, GameStatus::Idle);
        self.game_status = GameStatus::InProgress;
    }

    pub fn sent_msg_id(&self) -> SentMessageId {
        self.msg_ids.0
    }

    pub fn original_msg_id(&self) -> OriginalMessageId {
        self.msg_ids.1
    }
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct State {
    pub target_program_id: ActorId,
    pub players: BTreeMap<ActorId, PlayerInfo>,
}

impl Into<GameStatus> for WordleEvent {
    fn into(self) -> GameStatus {
        match self {
            WordleEvent::GameStarted { .. } => GameStatus::Started,
            WordleEvent::WordChecked {
                correct_positions, ..
            } => {
                let guessed = correct_positions.len() == WORD_LENGTH;
                GameStatus::WordChecked(guessed)
            }
        }
    }
}
