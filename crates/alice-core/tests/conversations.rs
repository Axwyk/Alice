mod support;
use alice_core::{
    conversation::*,
    error::{OperationCode, ValidationCode},
    message::Role,
};

fn conversation() -> Conversation {
    Conversation::new(
        ConversationId::new("c").unwrap(),
        ConversationLimits::new(8, 1000).unwrap(),
    )
}
#[test]
fn append_advances_revision_and_preserves_identity() {
    let mut c = conversation();
    let rev = c
        .append(
            support::message("s", Role::System, "instrucción"),
            Revision::INITIAL,
        )
        .unwrap();
    assert_eq!(rev.get(), 1);
    c.append(support::message("u", Role::User, "hola"), rev)
        .unwrap();
    assert_eq!(c.id().as_str(), "c");
    assert_eq!(c.messages().len(), 2);
    assert_eq!(c.revision().get(), 2);
}
#[test]
fn stale_revision_does_not_mutate_conversation() {
    let mut c = conversation();
    c.append(support::message("u", Role::User, "hola"), Revision::INITIAL)
        .unwrap();
    let before = c.clone();
    assert_eq!(
        c.append(
            support::message("a", Role::Assistant, "sí"),
            Revision::INITIAL
        )
        .unwrap_err()
        .code(),
        OperationCode::RevisionConflict.as_str()
    );
    assert_eq!(c, before);
}
#[test]
fn duplicate_ids_and_invalid_roles_are_rejected_atomically() {
    let mut c = conversation();
    assert_eq!(
        c.append(
            support::message("a", Role::Assistant, "sí"),
            Revision::INITIAL
        )
        .unwrap_err()
        .code(),
        ValidationCode::InvalidRole.as_str()
    );
    c.append(support::message("u", Role::User, "hola"), Revision::INITIAL)
        .unwrap();
    let before = c.clone();
    assert_eq!(
        c.append(support::message("u", Role::Assistant, "sí"), c.revision())
            .unwrap_err()
            .code(),
        ValidationCode::DuplicateMessage.as_str()
    );
    assert_eq!(
        c.append(support::message("s", Role::System, "tarde"), c.revision())
            .unwrap_err()
            .code(),
        ValidationCode::InvalidRole.as_str()
    );
    assert_eq!(c, before);
}
#[test]
fn history_count_and_bytes_are_bounded() {
    for limits in [
        ConversationLimits::new(1, 100).unwrap(),
        ConversationLimits::new(8, 4).unwrap(),
    ] {
        let mut c = Conversation::new(ConversationId::new("c").unwrap(), limits);
        c.append(support::message("u", Role::User, "hola"), Revision::INITIAL)
            .unwrap();
        let before = c.clone();
        assert!(
            c.append(support::message("a", Role::Assistant, "sí"), c.revision())
                .is_err()
        );
        assert_eq!(c, before);
    }
    assert!(ConversationLimits::new(0, 10).is_err());
    assert!(ConversationLimits::new(10, 0).is_err());
}
#[test]
fn restored_snapshots_are_validated() {
    let messages = vec![
        support::message("u", Role::User, "hola"),
        support::message("a", Role::Assistant, "sí"),
    ];
    let limits = ConversationLimits::new(8, 1000).unwrap();
    assert!(
        Conversation::restore(
            ConversationId::new("c").unwrap(),
            Revision::new(2),
            messages.clone(),
            limits
        )
        .is_ok()
    );
    assert_eq!(
        Conversation::restore(
            ConversationId::new("c").unwrap(),
            Revision::new(1),
            messages,
            limits
        )
        .unwrap_err()
        .code(),
        ValidationCode::InvalidRevision
    );
    let duplicates = vec![
        support::message("u", Role::User, "hola"),
        support::message("u", Role::Assistant, "sí"),
    ];
    assert!(
        Conversation::restore(
            ConversationId::new("c").unwrap(),
            Revision::new(2),
            duplicates,
            limits
        )
        .is_err()
    );
}
#[test]
fn revision_exhaustion_is_an_error_not_wraparound() {
    let mut c = Conversation::restore(
        ConversationId::new("c").unwrap(),
        Revision::new(u64::MAX),
        vec![],
        ConversationLimits::new(8, 1000).unwrap(),
    )
    .unwrap();
    let before = c.clone();
    assert_eq!(
        c.append(support::message("u", Role::User, "hola"), c.revision())
            .unwrap_err()
            .code(),
        OperationCode::RevisionExhausted.as_str()
    );
    assert_eq!(c, before);
}
