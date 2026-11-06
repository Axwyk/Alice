mod support;

#[test]
fn admitted_cancellation_and_running_failure_follow_the_remaining_terminal_paths() {
    let mut accepted = Operation::new(support::correlation());
    accepted
        .apply_event(&event(1, OperationEventKind::Accepted))
        .unwrap();
    accepted
        .apply_event(&event(2, OperationEventKind::CancelRequested))
        .unwrap();
    accepted
        .apply_event(&event(
            3,
            OperationEventKind::Cancelled(RemoteCancellation::NotApplicable),
        ))
        .unwrap();
    assert_eq!(accepted.state(), OperationState::Cancelled);
    let mut failed = running();
    failed
        .apply_event(&event(
            3,
            OperationEventKind::Failed(OperationError::new(OperationCode::Timeout).into()),
        ))
        .unwrap();
    assert_eq!(failed.state(), OperationState::Failed);
    assert_eq!(failed.last_sequence(), 3);
}
use alice_core::{error::*, message::Role, operation::*, provider::*};
fn event(n: u64, kind: OperationEventKind) -> OperationEvent {
    OperationEvent::new(support::correlation(), n, kind, 8000).unwrap()
}
fn running() -> Operation {
    let mut op = Operation::new(support::correlation());
    op.apply_event(&event(1, OperationEventKind::Accepted))
        .unwrap();
    op.apply_event(&event(2, OperationEventKind::Started))
        .unwrap();
    op
}
#[test]
fn accepted_operation_can_complete_once_with_correlated_events() {
    let mut op = running();
    op.apply_event(&event(3, OperationEventKind::TextDelta("res".into())))
        .unwrap();
    op.apply_event(&event(
        4,
        OperationEventKind::Completed(support::response()),
    ))
    .unwrap();
    assert_eq!(op.state(), OperationState::Completed);
    assert_eq!(op.last_sequence(), 4);
    let before = op.clone();
    assert!(
        op.apply_event(&event(
            5,
            OperationEventKind::Failed(StorageError::new(StorageCode::Unavailable).into())
        ))
        .is_err()
    );
    assert_eq!(op, before);
    assert!(matches!(
        op.terminal_event().unwrap().kind(),
        OperationEventKind::Completed(_)
    ));
}
#[test]
fn cancellation_request_is_not_confirmation_and_blocks_late_completion() {
    let mut op = running();
    op.apply_event(&event(3, OperationEventKind::CancelRequested))
        .unwrap();
    assert_eq!(op.state(), OperationState::CancelRequested);
    assert!(!op.state().is_terminal());
    assert!(op.terminal_event().is_none());
    let before = op.clone();
    assert_eq!(
        op.apply_event(&event(
            4,
            OperationEventKind::Completed(support::response())
        ))
        .unwrap_err()
        .code(),
        OperationCode::InvalidTransition
    );
    assert_eq!(op, before);
    op.apply_event(&event(
        4,
        OperationEventKind::Cancelled(RemoteCancellation::Unconfirmed),
    ))
    .unwrap();
    assert_eq!(op.state(), OperationState::Cancelled);
    assert!(matches!(
        op.terminal_event().unwrap().kind(),
        OperationEventKind::Cancelled(RemoteCancellation::Unconfirmed)
    ));
}
#[test]
fn rejected_operation_was_never_admitted() {
    let mut op = Operation::new(support::correlation());
    op.reject().unwrap();
    assert_eq!(op.state(), OperationState::Rejected);
    assert_eq!(op.last_sequence(), 0);
    assert!(op.terminal_event().is_none());
    assert!(op.reject().is_err());
    assert!(
        op.apply_event(&event(1, OperationEventKind::Accepted))
            .is_err()
    );
}
#[test]
fn failure_can_close_after_admission_without_starting_or_using_storage() {
    let mut op = Operation::new(support::correlation());
    op.apply_event(&event(1, OperationEventKind::Accepted))
        .unwrap();
    op.apply_event(&event(
        2,
        OperationEventKind::Failed(StorageError::new(StorageCode::Unavailable).into()),
    ))
    .unwrap();
    assert_eq!(op.state(), OperationState::Failed);
    assert_eq!(op.terminal_event().unwrap().sequence(), 2);
    assert!(op.reject().is_err());
}
#[test]
fn invalid_state_edges_do_not_mutate_the_value() {
    let cases = [
        (
            Operation::new(support::correlation()),
            event(1, OperationEventKind::Started),
        ),
        (running(), event(3, OperationEventKind::Started)),
        (running(), event(3, OperationEventKind::Accepted)),
        (
            running(),
            event(
                3,
                OperationEventKind::Cancelled(RemoteCancellation::NotApplicable),
            ),
        ),
    ];
    for (mut op, e) in cases {
        let before = op.clone();
        assert_eq!(
            op.apply_event(&e).unwrap_err().code(),
            OperationCode::InvalidTransition
        );
        assert_eq!(op, before);
    }
}
#[test]
fn duplicate_or_skipped_sequence_does_not_change_the_operation() {
    let mut op = running();
    let before = op.clone();
    for seq in [1, 2, 4, u64::MAX] {
        assert_eq!(
            op.apply_event(&event(seq, OperationEventKind::TextDelta("x".into())))
                .unwrap_err()
                .code(),
            OperationCode::InvalidSequence
        );
        assert_eq!(op, before);
    }
}
#[test]
fn foreign_conversation_operation_or_message_is_rejected() {
    for other in [
        Correlation::new(
            "other".parse().unwrap(),
            "operation-1".parse().unwrap(),
            "input-1".parse().unwrap(),
        ),
        Correlation::new(
            "conversation-1".parse().unwrap(),
            "other".parse().unwrap(),
            "input-1".parse().unwrap(),
        ),
        Correlation::new(
            "conversation-1".parse().unwrap(),
            "operation-1".parse().unwrap(),
            "other".parse().unwrap(),
        ),
    ] {
        let mut op = running();
        let before = op.clone();
        let e =
            OperationEvent::new(other, 3, OperationEventKind::TextDelta("x".into()), 8).unwrap();
        assert_eq!(
            op.apply_event(&e).unwrap_err().code(),
            OperationCode::CorrelationMismatch
        );
        assert_eq!(op, before);
    }
}
#[test]
fn event_payloads_sequences_and_nested_correlation_are_validated() {
    assert!(
        OperationEvent::new(support::correlation(), 0, OperationEventKind::Accepted, 8).is_err()
    );
    assert!(
        OperationEvent::new(support::correlation(), 1, OperationEventKind::Accepted, 0).is_err()
    );
    assert!(
        OperationEvent::new(
            support::correlation(),
            1,
            OperationEventKind::TextDelta("".into()),
            8
        )
        .is_err()
    );
    assert!(
        OperationEvent::new(
            support::correlation(),
            1,
            OperationEventKind::TextDelta("123".into()),
            2
        )
        .is_err()
    );
    let foreign_error = ProviderError::new("other".parse().unwrap(), ProviderCode::Protocol);
    assert!(
        OperationEvent::new(
            support::correlation(),
            1,
            OperationEventKind::Failed(foreign_error.clone().into()),
            8
        )
        .is_err()
    );
    assert!(
        ProviderEvent::new(
            support::correlation(),
            ProviderEventKind::Failed(foreign_error),
            8
        )
        .is_err()
    );
    let foreign_response = AiResponse::new(
        Correlation::new(
            "other".parse().unwrap(),
            "operation-1".parse().unwrap(),
            "input-1".parse().unwrap(),
        ),
        support::model(),
        support::message("answer", Role::Assistant, "sí"),
        FinishReason::Stop,
        Usage {
            input_tokens: None,
            output_tokens: None,
        },
    )
    .unwrap();
    assert!(
        ProviderEvent::new(
            support::correlation(),
            ProviderEventKind::Completed(foreign_response),
            8
        )
        .is_err()
    );
}
#[test]
fn provider_event_contract_does_not_assign_application_sequence() {
    let e = ProviderEvent::new(
        support::correlation(),
        ProviderEventKind::TextDelta("texto".into()),
        8,
    )
    .unwrap();
    assert_eq!(e.correlation(), &support::correlation());
    assert!(matches!(e.kind(), ProviderEventKind::TextDelta(s) if s == "texto"));
}
