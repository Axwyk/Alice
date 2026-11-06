//! Validated text messages. Limits are explicit and finite; there is no unlimited default.
use crate::error::{ValidationCode, ValidationError, identifier};
use std::{num::NonZeroUsize, time::SystemTime};

identifier!(MessageId);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MessageLimits {
    max_chars: NonZeroUsize,
    max_bytes: NonZeroUsize,
}
impl MessageLimits {
    pub fn new(max_chars: usize, max_bytes: usize) -> Result<Self, ValidationError> {
        Ok(Self {
            max_chars: NonZeroUsize::new(max_chars)
                .ok_or_else(|| ValidationError::new(ValidationCode::InvalidLimit))?,
            max_bytes: NonZeroUsize::new(max_bytes)
                .ok_or_else(|| ValidationError::new(ValidationCode::InvalidLimit))?,
        })
    }
    pub fn max_chars(self) -> usize {
        self.max_chars.get()
    }
    pub fn max_bytes(self) -> usize {
        self.max_bytes.get()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    id: MessageId,
    role: Role,
    content: String,
    created_at: Option<SystemTime>,
}
impl Message {
    pub fn new(
        id: MessageId,
        role: Role,
        content: impl Into<String>,
        created_at: Option<SystemTime>,
        limits: MessageLimits,
    ) -> Result<Self, ValidationError> {
        let content = content.into();
        if content.len() > limits.max_bytes() || content.chars().count() > limits.max_chars() {
            return Err(ValidationError::new(ValidationCode::ContentLimit));
        }
        if content.trim().is_empty() {
            return Err(ValidationError::new(ValidationCode::EmptyContent));
        }
        Ok(Self {
            id,
            role,
            content,
            created_at,
        })
    }
    pub fn id(&self) -> &MessageId {
        &self.id
    }
    pub fn role(&self) -> Role {
        self.role
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }
}
