//! Per-value operation rules. No scheduler, registry, coordinator or cancellation controller.
use crate::{
    BoxFuture,
    conversation::ConversationId,
    error::{
        CoreError, OperationCode, OperationError, ValidationCode, ValidationError, identifier,
    },
    message::MessageId,
    provider::{AiResponse, Usage},
};
use std::{num::NonZeroU64, sync::Arc, time::Instant};

identifier!(OperationId);

/// Read-only, one-shot cancellation REQUEST, not acknowledgement of remote abort.
/// `requested` must resolve immediately after a prior request, register wakeups
/// without lost signals and never revoke cancellation when its future is dropped.
/// Concrete controllers and runtime-specific implementations belong outside Core.
pub trait CancellationSignal: Send + Sync {
    fn is_requested(&self) -> bool;
    fn requested(&self) -> BoxFuture<'_, ()>;
}

#[derive(Clone)]
pub struct OperationContext {
    operation_id: OperationId,
    deadline: Instant,
    cancellation: Arc<dyn CancellationSignal>,
}
impl OperationContext {
    pub fn new(
        operation_id: OperationId,
        deadline: Instant,
        cancellation: Arc<dyn CancellationSignal>,
    ) -> Self {
        Self {
            operation_id,
            deadline,
            cancellation,
        }
    }
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    pub fn cancellation(&self) -> &dyn CancellationSignal {
        self.cancellation.as_ref()
    }
    /// Caller supplies time; Core starts no clock, timer or task.
    pub fn is_expired_at(&self, now: Instant) -> bool {
        now >= self.deadline
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Correlation {
    conversation_id: ConversationId,
    operation_id: OperationId,
    input_message_id: MessageId,
}
impl Correlation {
    pub fn new(
        conversation_id: ConversationId,
        operation_id: OperationId,
        input_message_id: MessageId,
    ) -> Self {
        Self {
            conversation_id,
            operation_id,
            input_message_id,
        }
    }
    pub fn conversation_id(&self) -> &ConversationId {
        &self.conversation_id
    }
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    pub fn input_message_id(&self) -> &MessageId {
        &self.input_message_id
    }
}

/// Supplied by infrastructure evidence, never inferred merely from a signal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteCancellation {
    NotApplicable,
    Unconfirmed,
    Confirmed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationState {
    Validating,
    Rejected,
    Accepted,
    Running,
    CancelRequested,
    Completed,
    Failed,
    Cancelled,
}
impl OperationState {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Rejected | Self::Completed | Self::Failed | Self::Cancelled
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationEventKind {
    Accepted,
    Started,
    TextDelta(String),
    UsageUpdated(Usage),
    CancelRequested,
    Completed(AiResponse),
    Failed(CoreError),
    Cancelled(RemoteCancellation),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationEvent {
    correlation: Correlation,
    sequence: NonZeroU64,
    kind: OperationEventKind,
}
impl OperationEvent {
    /// Event payload budget is mandatory even though no streaming is executed here.
    pub fn new(
        correlation: Correlation,
        sequence: u64,
        kind: OperationEventKind,
        max_payload_bytes: usize,
    ) -> Result<Self, ValidationError> {
        if max_payload_bytes == 0 {
            return Err(ValidationError::new(ValidationCode::InvalidLimit));
        }
        let sequence = NonZeroU64::new(sequence)
            .ok_or_else(|| ValidationError::new(ValidationCode::InvalidSequence))?;
        match &kind {
            OperationEventKind::Completed(response) => {
                if response.correlation() != &correlation {
                    return Err(ValidationError::new(ValidationCode::CorrelationMismatch));
                }
                if response.message().content().len() > max_payload_bytes {
                    return Err(ValidationError::new(ValidationCode::ContentLimit));
                }
            }
            OperationEventKind::TextDelta(text) => {
                if text.is_empty() {
                    return Err(ValidationError::new(ValidationCode::EmptyContent));
                }
                if text.len() > max_payload_bytes {
                    return Err(ValidationError::new(ValidationCode::ContentLimit));
                }
            }
            OperationEventKind::Failed(CoreError::Provider(error))
                if error.operation_id() != correlation.operation_id() =>
            {
                return Err(ValidationError::new(ValidationCode::CorrelationMismatch));
            }
            _ => {}
        }
        Ok(Self {
            correlation,
            sequence,
            kind,
        })
    }
    pub fn correlation(&self) -> &Correlation {
        &self.correlation
    }
    pub fn sequence(&self) -> u64 {
        self.sequence.get()
    }
    pub fn kind(&self) -> &OperationEventKind {
        &self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operation {
    correlation: Correlation,
    state: OperationState,
    last_sequence: u64,
    terminal: Option<OperationEvent>,
}
impl Operation {
    pub fn new(correlation: Correlation) -> Self {
        Self {
            correlation,
            state: OperationState::Validating,
            last_sequence: 0,
            terminal: None,
        }
    }
    pub fn correlation(&self) -> &Correlation {
        &self.correlation
    }
    pub fn state(&self) -> OperationState {
        self.state
    }
    pub fn last_sequence(&self) -> u64 {
        self.last_sequence
    }
    pub fn terminal_event(&self) -> Option<&OperationEvent> {
        self.terminal.as_ref()
    }
    /// Rejection precedes admission and therefore has no accepted-operation event.
    pub fn reject(&mut self) -> Result<(), OperationError> {
        if self.state != OperationState::Validating {
            return Err(OperationError::new(OperationCode::InvalidTransition));
        }
        self.state = OperationState::Rejected;
        Ok(())
    }
    /// Validate and update one value atomically. No application work is executed.
    pub fn apply_event(&mut self, event: &OperationEvent) -> Result<(), OperationError> {
        if event.correlation() != &self.correlation {
            return Err(OperationError::new(OperationCode::CorrelationMismatch));
        }
        let next_sequence = self
            .last_sequence
            .checked_add(1)
            .ok_or_else(|| OperationError::new(OperationCode::SequenceExhausted))?;
        if event.sequence() != next_sequence {
            return Err(OperationError::new(OperationCode::InvalidSequence));
        }
        use OperationEventKind as E;
        use OperationState as S;
        let next = match (self.state, event.kind()) {
            (S::Validating, E::Accepted) => S::Accepted,
            (S::Accepted, E::Started) => S::Running,
            (S::Running, E::TextDelta(_) | E::UsageUpdated(_)) => S::Running,
            (S::Accepted | S::Running, E::CancelRequested) => S::CancelRequested,
            (S::Running, E::Completed(_)) => S::Completed,
            (S::Accepted | S::Running, E::Failed(_)) => S::Failed,
            (S::CancelRequested, E::Cancelled(_)) => S::Cancelled,
            _ => return Err(OperationError::new(OperationCode::InvalidTransition)),
        };
        self.state = next;
        self.last_sequence = next_sequence;
        if next.is_terminal() {
            self.terminal = Some(event.clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_overflow_does_not_mutate_or_wrap() {
        let correlation = Correlation::new(
            "c".parse().unwrap(),
            "o".parse().unwrap(),
            "m".parse().unwrap(),
        );
        let mut operation = Operation::new(correlation.clone());
        operation.state = OperationState::Running;
        operation.last_sequence = u64::MAX;
        let before = operation.clone();
        let event =
            OperationEvent::new(correlation, 1, OperationEventKind::TextDelta("x".into()), 1)
                .unwrap();
        assert_eq!(
            operation.apply_event(&event).unwrap_err().code(),
            OperationCode::SequenceExhausted
        );
        assert_eq!(operation, before);
    }
}
