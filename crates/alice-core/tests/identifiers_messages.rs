mod support;
use alice_core::{
    conversation::ConversationId,
    error::*,
    message::{Message, MessageId, MessageLimits, Role},
    operation::OperationId,
};
use std::time::SystemTime;

#[test]
fn identifiers_are_typed_opaque_and_round_trip() {
    let id: ConversationId = "conv_1-ABC".parse().unwrap();
    assert_eq!(id.as_str(), "conv_1-ABC");
    assert_eq!(id.to_string(), "conv_1-ABC");
    assert_eq!(id, ConversationId::new("conv_1-ABC").unwrap());
    assert!(MessageId::new("x".repeat(128)).is_ok());
    assert!(OperationId::new("123e4567-e89b-12d3-a456-426614174000").is_ok());
}
#[test]
fn all_identifier_types_reject_invalid_values_without_echoing_them() {
    for bad in ["", " ", "id with space", "a/b", "a\nb", "á", "secret=token"] {
        for error in [
            ConversationId::new(bad).unwrap_err(),
            MessageId::new(bad).unwrap_err(),
            OperationId::new(bad).unwrap_err(),
        ] {
            assert_eq!(error.code(), ValidationCode::InvalidIdentifier);
            assert_eq!(error.safe_message(), "El identificador no es válido.");
            assert!(!error.to_string().contains("secret="));
        }
    }
    assert!(OperationId::new("x".repeat(129)).is_err());
}
#[test]
fn message_preserves_text_role_timestamp_and_unicode() {
    let message = support::message("m", Role::User, "  ñ\nsegunda línea  ");
    assert_eq!(message.content(), "  ñ\nsegunda línea  ");
    assert_eq!(message.role(), Role::User);
    assert_eq!(message.created_at(), Some(SystemTime::UNIX_EPOCH));
}
#[test]
fn message_rejects_empty_or_whitespace_content() {
    for text in ["", " \n\t", "\u{2003}"] {
        let error = Message::new(
            MessageId::new("m").unwrap(),
            Role::User,
            text,
            None,
            MessageLimits::new(10, 40).unwrap(),
        )
        .unwrap_err();
        assert_eq!(error.code(), ValidationCode::EmptyContent);
    }
}
#[test]
fn character_and_byte_budgets_are_both_enforced() {
    let make = |text: String, chars, bytes| {
        Message::new(
            MessageId::new("m").unwrap(),
            Role::User,
            text,
            None,
            MessageLimits::new(chars, bytes).unwrap(),
        )
    };
    assert!(make("ñ".repeat(2000), 2000, 8000).is_ok());
    assert_eq!(
        make("ñ".repeat(2001), 2000, 8000).unwrap_err().code(),
        ValidationCode::ContentLimit
    );
    assert!(make("𝄞".into(), 1, 4).is_ok());
    assert_eq!(
        make("𝄞".into(), 1, 3).unwrap_err().code(),
        ValidationCode::ContentLimit
    );
}
#[test]
fn limits_cannot_be_zero() {
    assert!(MessageLimits::new(0, 1).is_err());
    assert!(MessageLimits::new(1, 0).is_err());
}
#[test]
fn error_families_have_stable_codes_and_fixed_safe_messages() {
    let errors: Vec<CoreError> = vec![
        ValidationError::new(ValidationCode::EmptyContent).into(),
        OperationError::new(OperationCode::Timeout).into(),
        ProviderError::new(
            support::correlation().operation_id().clone(),
            ProviderCode::Authentication,
        )
        .into(),
        StorageError::new(StorageCode::Unavailable).into(),
    ];
    let codes = [
        "validation.empty_content",
        "operation.timeout",
        "provider.authentication",
        "storage.unavailable",
    ];
    for (error, code) in errors.iter().zip(codes) {
        assert_eq!(error.code(), code);
        assert!(error.to_string().contains(error.safe_message()));
        assert!(!error.safe_message().is_empty());
        let _: &dyn std::error::Error = error;
    }
}
