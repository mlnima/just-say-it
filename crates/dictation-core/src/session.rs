use crate::defaults;
use serde::Serialize;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Idle,
    Preparing,
    Recording,
    Transcribing,
    Inserting,
    PendingDelivery,
    Paused,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Delivery {
    Submitted,
    Rejected,
    Partial,
    Unknown,
}

pub struct Recovery {
    pub text: String,
    pub outcome: Delivery,
    pub created: Instant,
}

#[derive(Default)]
pub struct Session {
    pub phase: Phase,
    pub id: u64,
    next_id: u64,
    attempted: bool,
    pub recovery: Option<Recovery>,
}

impl Session {
    pub fn start(&mut self) -> Result<u64, String> {
        self.expire();
        if !matches!(self.phase, Phase::Idle | Phase::Error) {
            return Err("Finish or discard the current session before starting another.".into());
        }
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or("Session counter exhausted; restart the app.")?;
        self.id = self.next_id;
        self.attempted = false;
        self.recovery = None;
        self.phase = Phase::Preparing;
        Ok(self.id)
    }

    pub fn authorize_delivery(&mut self, id: u64) -> bool {
        if self.id != id || self.attempted || self.phase != Phase::Transcribing {
            return false;
        }
        self.attempted = true;
        self.phase = Phase::Inserting;
        true
    }

    pub fn delivered(&mut self, text: String, outcome: Delivery) {
        self.phase = if outcome == Delivery::Submitted {
            Phase::Idle
        } else {
            Phase::PendingDelivery
        };
        self.recovery = Some(Recovery {
            text,
            outcome,
            created: Instant::now(),
        });
        self.id = 0;
    }

    pub fn cancel(&mut self, paused: bool) {
        self.id = 0;
        self.recovery = None;
        self.phase = if paused { Phase::Paused } else { Phase::Idle };
    }

    pub fn expire(&mut self) {
        if self
            .recovery
            .as_ref()
            .is_some_and(|r| r.created.elapsed() >= Duration::from_secs(defaults::RECOVERY_SECONDS))
        {
            self.recovery = None;
            if self.phase == Phase::PendingDelivery {
                self.phase = Phase::Idle;
            }
        }
    }
}

pub fn safe_inline_text(text: &str) -> bool {
    !text.is_empty()
        && text.chars().count() <= defaults::MAX_TRANSCRIPT_CHARS
        && !text.chars().any(char::is_control)
}
