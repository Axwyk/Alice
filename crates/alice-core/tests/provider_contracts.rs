mod support;
use alice_core::{
    conversation::ConversationLimits, error::ValidationCode, message::Role, operation::Correlation,
    provider::*, storage::ConversationRepository,
};
use std::{num::NonZeroU32, sync::Arc};

#[test]
fn numerical_limits_keep_evidence_and_expire_without_becoming_zero() {
    let evidence = CapabilityEvidence::new(
        CapabilitySource::ProviderMetadata,
        CapabilityScope::Model(support::model()),
        Some(support::time(10)),
        Some(support::time(20)),
    )
    .unwrap();
    let limit = TokenLimit::new(NonZeroU32::new(64).unwrap(), evidence).unwrap();
    assert_eq!(limit.value_at(support::time(10)).unwrap().get(), 64);
    assert_eq!(limit.value_at(support::time(20)), None);
    let c = capability(
        Support::Supported,
        CapabilityScope::Model(support::model()),
        CapabilitySource::Configuration,
    );
    let caps = ProviderCapabilities::new(
        support::model(),
        c.clone(),
        c.clone(),
        c.clone(),
        Some(limit),
        None,
    )
    .unwrap();
    assert_eq!(
        caps.max_context_tokens().unwrap().evidence().source(),
        CapabilitySource::ProviderMetadata
    );
    let foreign = TokenLimit::new(
        NonZeroU32::new(64).unwrap(),
        CapabilityEvidence::new(
            CapabilitySource::Configuration,
            CapabilityScope::Model(ModelRef::new("other", "model").unwrap()),
            None,
            None,
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        ProviderCapabilities::new(
            support::model(),
            c.clone(),
            c.clone(),
            c,
            Some(foreign),
            None
        )
        .is_err()
    );
    assert!(
        TokenLimit::new(
            NonZeroU32::new(64).unwrap(),
            CapabilityEvidence::new(
                CapabilitySource::Unknown,
                CapabilityScope::Model(support::model()),
                None,
                None
            )
            .unwrap()
        )
        .is_err()
    );
}

fn capability(state: Support, scope: CapabilityScope, source: CapabilitySource) -> Capability {
    Capability::new(
        state,
        CapabilityEvidence::new(source, scope, None, None).unwrap(),
    )
    .unwrap()
}
fn adapter(state: Support) -> AdapterCapabilities {
    let c = capability(
        state,
        CapabilityScope::Adapter,
        CapabilitySource::AdapterDeclaration,
    );
    AdapterCapabilities::new(c.clone(), c.clone(), c).unwrap()
}
fn model_caps(state: Support) -> ProviderCapabilities {
    let c = capability(
        state,
        CapabilityScope::Model(support::model()),
        CapabilitySource::Configuration,
    );
    ProviderCapabilities::new(support::model(), c.clone(), c.clone(), c, None, None).unwrap()
}
#[test]
fn contracts_are_send_sync_and_dyn_compatible_without_any_provider_implementation() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<Arc<dyn AiProvider>>();
    send_sync::<Arc<dyn ConversationRepository>>();
    send_sync::<AiRequest>();
    send_sync::<AiResponse>();
    send_sync::<ProviderCapabilities>();
    // Type checking only: no fake provider, stream or repository is instantiated.
    let _: Option<ProviderStream> = None;
    let _: Option<Arc<dyn AiProvider>> = None;
}
#[test]
fn model_reference_is_opaque_but_bounded() {
    assert_eq!(support::model().model_id(), "namespace/model:1");
    assert_eq!(support::model().instance_id(), "instance-1");
    for bad in ["", "with space", "line\nbreak"] {
        assert!(ModelRef::new("instance", bad).is_err());
    }
    assert!(ModelRef::new("instance", "x".repeat(257)).is_err());
    assert!(ModelRef::new("bad instance", "model").is_err());
}
#[test]
fn unknown_is_not_zero_or_supported() {
    let unknown = Capability::new(
        Support::Unknown,
        CapabilityEvidence::new(
            CapabilitySource::Unknown,
            CapabilityScope::Adapter,
            None,
            None,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(unknown.support_at(support::time(1)), Support::Unknown);
    assert_eq!(model_caps(Support::Supported).max_context_tokens(), None);
    assert_ne!(
        Usage {
            input_tokens: None,
            output_tokens: None
        },
        Usage {
            input_tokens: Some(0),
            output_tokens: Some(0)
        }
    );
    assert!(Capability::new(Support::Supported, unknown.evidence().clone()).is_err());
}
#[test]
fn evidence_records_source_scope_and_expiry_without_discovery() {
    let evidence = CapabilityEvidence::new(
        CapabilitySource::ProviderMetadata,
        CapabilityScope::Model(support::model()),
        Some(support::time(10)),
        Some(support::time(20)),
    )
    .unwrap();
    assert_eq!(evidence.source(), CapabilitySource::ProviderMetadata);
    assert_eq!(evidence.observed_at(), Some(support::time(10)));
    assert_eq!(evidence.valid_until(), Some(support::time(20)));
    let c = Capability::new(Support::Supported, evidence).unwrap();
    assert_eq!(c.support_at(support::time(9)), Support::Unknown);
    assert_eq!(c.support_at(support::time(10)), Support::Supported);
    assert_eq!(c.support_at(support::time(20)), Support::Unknown);
}
#[test]
fn invalid_evidence_cannot_claim_verified_support() {
    for (source, start, end) in [
        (CapabilitySource::Unknown, Some(support::time(1)), None),
        (
            CapabilitySource::ProviderMetadata,
            None,
            Some(support::time(2)),
        ),
        (
            CapabilitySource::RuntimeVerification,
            Some(support::time(2)),
            Some(support::time(2)),
        ),
    ] {
        assert_eq!(
            CapabilityEvidence::new(source, CapabilityScope::Adapter, start, end)
                .unwrap_err()
                .code(),
            ValidationCode::InvalidEvidence
        );
    }
}
#[test]
fn model_and_adapter_capability_scopes_cannot_be_confused() {
    let c = capability(
        Support::Supported,
        CapabilityScope::Adapter,
        CapabilitySource::AdapterDeclaration,
    );
    assert!(
        ProviderCapabilities::new(support::model(), c.clone(), c.clone(), c, None, None).is_err()
    );
    let c = capability(
        Support::Supported,
        CapabilityScope::Model(ModelRef::new("other", "model").unwrap()),
        CapabilitySource::RuntimeVerification,
    );
    assert!(
        ProviderCapabilities::new(
            support::model(),
            c.clone(),
            c.clone(),
            c.clone(),
            None,
            None
        )
        .is_err()
    );
    assert!(AdapterCapabilities::new(c.clone(), c.clone(), c).is_err());
}
#[test]
fn required_capabilities_intersect_adapter_and_model_support() {
    let required = RequiredCapabilities {
        streaming: true,
        cancellation: true,
    };
    assert!(
        model_caps(Support::Supported)
            .require(&adapter(Support::Supported), required, support::time(1))
            .is_ok()
    );
    assert_eq!(
        model_caps(Support::Supported)
            .require(&adapter(Support::Unsupported), required, support::time(1))
            .unwrap_err()
            .code(),
        ValidationCode::UnsupportedCapability
    );
    assert_eq!(
        model_caps(Support::Unknown)
            .require(&adapter(Support::Supported), required, support::time(1))
            .unwrap_err()
            .code(),
        ValidationCode::UnknownCapability
    );
}
#[test]
fn request_validates_last_input_identity_role_and_finite_context() {
    let build = |messages| {
        AiRequest::new(
            support::correlation(),
            support::model(),
            messages,
            ConversationLimits::new(4, 100).unwrap(),
            GenerationOptions {
                max_output_tokens: NonZeroU32::new(8).unwrap(),
            },
            RequiredCapabilities {
                streaming: false,
                cancellation: false,
            },
        )
    };
    assert!(build(vec![support::message("input-1", Role::User, "hola")]).is_ok());
    assert_eq!(
        build(vec![]).unwrap_err().code(),
        ValidationCode::EmptyContent
    );
    assert_eq!(
        build(vec![support::message("wrong", Role::User, "hola")])
            .unwrap_err()
            .code(),
        ValidationCode::CorrelationMismatch
    );
    assert!(build(vec![support::message("input-1", Role::System, "hola")]).is_err());
    assert!(
        build(vec![support::message(
            "input-1",
            Role::User,
            &"x".repeat(101)
        )])
        .is_err()
    );
}
#[test]
fn response_must_be_a_distinct_assistant_message_for_the_request() {
    assert!(
        support::response()
            .validate_for(&support::request())
            .is_ok()
    );
    let build = |message| {
        AiResponse::new(
            support::correlation(),
            support::model(),
            message,
            FinishReason::Stop,
            Usage {
                input_tokens: None,
                output_tokens: None,
            },
        )
    };
    assert_eq!(
        build(support::message("answer", Role::User, "no"))
            .unwrap_err()
            .code(),
        ValidationCode::InvalidRole
    );
    assert_eq!(
        build(support::message("input-1", Role::Assistant, "no"))
            .unwrap_err()
            .code(),
        ValidationCode::DuplicateMessage
    );
    let foreign = AiResponse::new(
        Correlation::new(
            "c".parse().unwrap(),
            "o".parse().unwrap(),
            "i".parse().unwrap(),
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
    assert!(foreign.validate_for(&support::request()).is_err());
}
