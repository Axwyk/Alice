//! A bounded conversation value, not a conversation service or repository.
use crate::{
    error::{
        CoreError, OperationCode, OperationError, ValidationCode, ValidationError, identifier,
    },
    message::{Message, Role},
};
use std::{collections::HashSet, num::NonZeroUsize};

identifier!(ConversationId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(u64);
impl Revision {
    pub const INITIAL: Self = Self(0);
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConversationLimits {
    max_messages: NonZeroUsize,
    max_content_bytes: NonZeroUsize,
}
impl ConversationLimits {
    pub fn new(max_messages: usize, max_content_bytes: usize) -> Result<Self, ValidationError> {
        Ok(Self {
            max_messages: NonZeroUsize::new(max_messages)
                .ok_or_else(|| ValidationError::new(ValidationCode::InvalidLimit))?,
            max_content_bytes: NonZeroUsize::new(max_content_bytes)
                .ok_or_else(|| ValidationError::new(ValidationCode::InvalidLimit))?,
        })
    }
    pub fn max_messages(self) -> usize {
        self.max_messages.get()
    }
    pub fn max_content_bytes(self) -> usize {
        self.max_content_bytes.get()
    }
}

/// Validate snapshots without accepting foreign message objects or mutating state.
pub(crate) fn validate_messages(
    messages: &[Message],
    limits: ConversationLimits,
) -> Result<usize, ValidationError> {
    if messages.len() > limits.max_messages() {
        return Err(ValidationError::new(ValidationCode::ContentLimit));
    }
    let mut ids = HashSet::new();
    let mut bytes = 0usize;
    let mut dialogue_started = false;
    let mut previous = None;
    for message in messages {
        if !ids.insert(message.id()) {
            return Err(ValidationError::new(ValidationCode::DuplicateMessage));
        }
        if (message.role() == Role::System && dialogue_started)
            || (message.role() == Role::Assistant && previous != Some(Role::User))
        {
            return Err(ValidationError::new(ValidationCode::InvalidRole));
        }
        dialogue_started |= message.role() != Role::System;
        previous = Some(message.role());
        bytes = bytes
            .checked_add(message.content().len())
            .filter(|n| *n <= limits.max_content_bytes())
            .ok_or_else(|| ValidationError::new(ValidationCode::ContentLimit))?;
    }
    Ok(bytes)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conversation {
    id: ConversationId,
    revision: Revision,
    messages: Vec<Message>,
    limits: ConversationLimits,
}
impl Conversation {
    pub fn new(id: ConversationId, limits: ConversationLimits) -> Self {
        Self {
            id,
            revision: Revision::INITIAL,
            messages: Vec::new(),
            limits,
        }
    }
    pub fn restore(
        id: ConversationId,
        revision: Revision,
        messages: Vec<Message>,
        limits: ConversationLimits,
    ) -> Result<Self, ValidationError> {
        validate_messages(&messages, limits)?;
        if u64::try_from(messages.len()).map_or(true, |len| revision.get() < len) {
            return Err(ValidationError::new(ValidationCode::InvalidRevision));
        }
        Ok(Self {
            id,
            revision,
            messages,
            limits,
        })
    }
    pub fn id(&self) -> &ConversationId {
        &self.id
    }
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
    /// Optimistic revision check. Failure leaves this value completely unchanged.
    pub fn append(&mut self, message: Message, expected: Revision) -> Result<Revision, CoreError> {
        if expected != self.revision {
            return Err(OperationError::new(OperationCode::RevisionConflict).into());
        }
        let next = self
            .revision
            .get()
            .checked_add(1)
            .ok_or_else(|| OperationError::new(OperationCode::RevisionExhausted))?;
        if self.messages.len() >= self.limits.max_messages() {
            return Err(ValidationError::new(ValidationCode::ContentLimit).into());
        }
        // Validate the proposed snapshot before committing; this temporary is bounded.
        let mut proposed = self.messages.clone();
        proposed.push(message);
        validate_messages(&proposed, self.limits)?;
        self.messages = proposed;
        self.revision = Revision(next);
        Ok(self.revision)
    }
}
