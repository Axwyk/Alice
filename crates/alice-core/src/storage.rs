//! Conversation storage port only. No implementation, engine, path or I/O.
use crate::{
    BoxFuture,
    conversation::{Conversation, ConversationId, Revision},
    error::StorageError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpectedRevision {
    Absent,
    Exact(Revision),
}

/// Saves a single aggregate atomically against the expected persisted revision.
/// `Absent` is create-only; `Exact` requires an existing matching revision.
/// Implementations must reject replacement with a revision that does not advance.
/// Operation terminals remain owned by the future application coordinator, never
/// by this port. No cross-aggregate transaction or retry semantics are implied.
pub trait ConversationRepository: Send + Sync {
    fn load(&self, id: ConversationId)
    -> BoxFuture<'_, Result<Option<Conversation>, StorageError>>;
    fn save(
        &self,
        conversation: Conversation,
        expected: ExpectedRevision,
    ) -> BoxFuture<'_, Result<(), StorageError>>;
}
