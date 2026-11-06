//! Neutral provider contracts. No providers, discovery, cache or network clients.
use crate::{
    BoxFuture,
    conversation::{ConversationLimits, validate_messages},
    error::{ProviderError, ValidationCode, ValidationError, validate_identifier},
    message::{Message, Role},
    operation::{Correlation, OperationContext, RemoteCancellation},
};
use std::{num::NonZeroU32, pin::Pin, time::SystemTime};

/// Owned stream contract only; EOF without a terminal is not successful completion.
pub type ProviderStream = Pin<Box<dyn futures_core::Stream<Item = ProviderEvent> + Send + 'static>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelRef {
    instance_id: String,
    model_id: String,
}
impl ModelRef {
    pub fn new(
        instance_id: impl Into<String>,
        model_id: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let instance_id = instance_id.into();
        let model_id = model_id.into();
        validate_identifier(&instance_id)?;
        if model_id.is_empty()
            || model_id.len() > 256
            || model_id
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(ValidationError::new(ValidationCode::InvalidIdentifier));
        }
        Ok(Self {
            instance_id,
            model_id,
        })
    }
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
    pub fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Support {
    Supported,
    Unsupported,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilitySource {
    AdapterDeclaration,
    Configuration,
    ProviderMetadata,
    RuntimeVerification,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityScope {
    Adapter,
    Model(ModelRef),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityEvidence {
    source: CapabilitySource,
    scope: CapabilityScope,
    observed_at: Option<SystemTime>,
    valid_until: Option<SystemTime>,
}
impl CapabilityEvidence {
    pub fn new(
        source: CapabilitySource,
        scope: CapabilityScope,
        observed_at: Option<SystemTime>,
        valid_until: Option<SystemTime>,
    ) -> Result<Self, ValidationError> {
        if (source == CapabilitySource::Unknown && (observed_at.is_some() || valid_until.is_some()))
            || (valid_until.is_some() && observed_at.is_none())
            || matches!((observed_at, valid_until), (Some(start), Some(end)) if end <= start)
        {
            return Err(ValidationError::new(ValidationCode::InvalidEvidence));
        }
        Ok(Self {
            source,
            scope,
            observed_at,
            valid_until,
        })
    }
    pub fn source(&self) -> CapabilitySource {
        self.source
    }
    pub fn scope(&self) -> &CapabilityScope {
        &self.scope
    }
    pub fn observed_at(&self) -> Option<SystemTime> {
        self.observed_at
    }
    pub fn valid_until(&self) -> Option<SystemTime> {
        self.valid_until
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capability {
    support: Support,
    evidence: CapabilityEvidence,
}
impl Capability {
    pub fn new(support: Support, evidence: CapabilityEvidence) -> Result<Self, ValidationError> {
        if support != Support::Unknown && evidence.source == CapabilitySource::Unknown {
            return Err(ValidationError::new(ValidationCode::InvalidEvidence));
        }
        Ok(Self { support, evidence })
    }
    pub fn declared_support(&self) -> Support {
        self.support
    }
    pub fn evidence(&self) -> &CapabilityEvidence {
        &self.evidence
    }
    /// Pure evaluation. This method neither refreshes metadata nor reads the clock.
    pub fn support_at(&self, now: SystemTime) -> Support {
        if self.evidence.observed_at.is_some_and(|start| now < start)
            || self.evidence.valid_until.is_some_and(|end| now >= end)
        {
            Support::Unknown
        } else {
            self.support
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterCapabilities {
    text: Capability,
    streaming: Capability,
    cancellation: Capability,
}
impl AdapterCapabilities {
    pub fn new(
        text: Capability,
        streaming: Capability,
        cancellation: Capability,
    ) -> Result<Self, ValidationError> {
        for capability in [&text, &streaming, &cancellation] {
            if capability.evidence.scope != CapabilityScope::Adapter
                || !matches!(
                    capability.evidence.source,
                    CapabilitySource::AdapterDeclaration | CapabilitySource::Unknown
                )
            {
                return Err(ValidationError::new(ValidationCode::InvalidEvidence));
            }
        }
        Ok(Self {
            text,
            streaming,
            cancellation,
        })
    }
    pub fn text(&self) -> &Capability {
        &self.text
    }
    pub fn streaming(&self) -> &Capability {
        &self.streaming
    }
    pub fn cancellation(&self) -> &Capability {
        &self.cancellation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderCapabilities {
    model: ModelRef,
    text: Capability,
    streaming: Capability,
    cancellation: Capability,
    max_context_tokens: Option<TokenLimit>,
    max_output_tokens: Option<TokenLimit>,
}

/// A known numerical limit retains its own model-scoped source and validity.
/// Missing or expired evidence is not silently interpreted as a zero limit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenLimit {
    value: NonZeroU32,
    evidence: CapabilityEvidence,
}
impl TokenLimit {
    pub fn new(value: NonZeroU32, evidence: CapabilityEvidence) -> Result<Self, ValidationError> {
        if !matches!(evidence.scope, CapabilityScope::Model(_))
            || matches!(
                evidence.source,
                CapabilitySource::Unknown | CapabilitySource::AdapterDeclaration
            )
        {
            return Err(ValidationError::new(ValidationCode::InvalidEvidence));
        }
        Ok(Self { value, evidence })
    }
    pub fn evidence(&self) -> &CapabilityEvidence {
        &self.evidence
    }
    pub fn value_at(&self, now: SystemTime) -> Option<NonZeroU32> {
        if self.evidence.observed_at.is_some_and(|start| now < start)
            || self.evidence.valid_until.is_some_and(|end| now >= end)
        {
            None
        } else {
            Some(self.value)
        }
    }
}
/// Model-specific capabilities retain the same neutral representation.
pub type ModelCapabilities = ProviderCapabilities;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequiredCapabilities {
    pub streaming: bool,
    pub cancellation: bool,
}
impl ProviderCapabilities {
    pub fn new(
        model: ModelRef,
        text: Capability,
        streaming: Capability,
        cancellation: Capability,
        max_context_tokens: Option<TokenLimit>,
        max_output_tokens: Option<TokenLimit>,
    ) -> Result<Self, ValidationError> {
        for capability in [&text, &streaming, &cancellation] {
            if capability.evidence.scope != CapabilityScope::Model(model.clone())
                || capability.evidence.source == CapabilitySource::AdapterDeclaration
            {
                return Err(ValidationError::new(ValidationCode::InvalidEvidence));
            }
        }
        for limit in [&max_context_tokens, &max_output_tokens]
            .into_iter()
            .flatten()
        {
            if limit.evidence.scope != CapabilityScope::Model(model.clone()) {
                return Err(ValidationError::new(ValidationCode::InvalidEvidence));
            }
        }
        Ok(Self {
            model,
            text,
            streaming,
            cancellation,
            max_context_tokens,
            max_output_tokens,
        })
    }
    pub fn model(&self) -> &ModelRef {
        &self.model
    }
    pub fn text(&self) -> &Capability {
        &self.text
    }
    pub fn streaming(&self) -> &Capability {
        &self.streaming
    }
    pub fn cancellation(&self) -> &Capability {
        &self.cancellation
    }
    pub fn max_context_tokens(&self) -> Option<&TokenLimit> {
        self.max_context_tokens.as_ref()
    }
    pub fn max_output_tokens(&self) -> Option<&TokenLimit> {
        self.max_output_tokens.as_ref()
    }
    pub fn require(
        &self,
        adapter: &AdapterCapabilities,
        required: RequiredCapabilities,
        now: SystemTime,
    ) -> Result<(), ValidationError> {
        for (model, adapter, needed) in [
            (&self.text, &adapter.text, true),
            (&self.streaming, &adapter.streaming, required.streaming),
            (
                &self.cancellation,
                &adapter.cancellation,
                required.cancellation,
            ),
        ] {
            if !needed {
                continue;
            }
            let states = [model.support_at(now), adapter.support_at(now)];
            if states.contains(&Support::Unsupported) {
                return Err(ValidationError::new(ValidationCode::UnsupportedCapability));
            }
            if states.contains(&Support::Unknown) {
                return Err(ValidationError::new(ValidationCode::UnknownCapability));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenerationOptions {
    pub max_output_tokens: NonZeroU32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiRequest {
    correlation: Correlation,
    model: ModelRef,
    messages: Vec<Message>,
    options: GenerationOptions,
    required: RequiredCapabilities,
}
impl AiRequest {
    pub fn new(
        correlation: Correlation,
        model: ModelRef,
        messages: Vec<Message>,
        limits: ConversationLimits,
        options: GenerationOptions,
        required: RequiredCapabilities,
    ) -> Result<Self, ValidationError> {
        validate_messages(&messages, limits)?;
        let last = messages
            .last()
            .ok_or_else(|| ValidationError::new(ValidationCode::EmptyContent))?;
        if last.role() != Role::User {
            return Err(ValidationError::new(ValidationCode::InvalidRole));
        }
        if last.id() != correlation.input_message_id() {
            return Err(ValidationError::new(ValidationCode::CorrelationMismatch));
        }
        Ok(Self {
            correlation,
            model,
            messages,
            options,
            required,
        })
    }
    pub fn correlation(&self) -> &Correlation {
        &self.correlation
    }
    pub fn model(&self) -> &ModelRef {
        &self.model
    }
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
    pub fn options(&self) -> GenerationOptions {
        self.options
    }
    pub fn required(&self) -> RequiredCapabilities {
        self.required
    }
    pub fn validate_context(&self, context: &OperationContext) -> Result<(), ValidationError> {
        if self.correlation.operation_id() != context.operation_id() {
            return Err(ValidationError::new(ValidationCode::CorrelationMismatch));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
    OutputLimit,
    Refusal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiResponse {
    correlation: Correlation,
    model: ModelRef,
    message: Message,
    finish_reason: FinishReason,
    usage: Usage,
}
impl AiResponse {
    pub fn new(
        correlation: Correlation,
        model: ModelRef,
        message: Message,
        finish_reason: FinishReason,
        usage: Usage,
    ) -> Result<Self, ValidationError> {
        if message.role() != Role::Assistant {
            return Err(ValidationError::new(ValidationCode::InvalidRole));
        }
        if message.id() == correlation.input_message_id() {
            return Err(ValidationError::new(ValidationCode::DuplicateMessage));
        }
        Ok(Self {
            correlation,
            model,
            message,
            finish_reason,
            usage,
        })
    }
    pub fn correlation(&self) -> &Correlation {
        &self.correlation
    }
    pub fn model(&self) -> &ModelRef {
        &self.model
    }
    pub fn message(&self) -> &Message {
        &self.message
    }
    pub fn finish_reason(&self) -> FinishReason {
        self.finish_reason
    }
    pub fn usage(&self) -> Usage {
        self.usage
    }
    pub fn validate_for(&self, request: &AiRequest) -> Result<(), ValidationError> {
        if self.correlation != request.correlation
            || self.model != request.model
            || request.messages.iter().any(|m| m.id() == self.message.id())
        {
            return Err(ValidationError::new(ValidationCode::CorrelationMismatch));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderEventKind {
    Started,
    TextDelta(String),
    UsageUpdated(Usage),
    Completed(AiResponse),
    Failed(ProviderError),
    Cancelled(RemoteCancellation),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderEvent {
    correlation: Correlation,
    kind: ProviderEventKind,
}
impl ProviderEvent {
    pub fn new(
        correlation: Correlation,
        kind: ProviderEventKind,
        max_payload_bytes: usize,
    ) -> Result<Self, ValidationError> {
        // The same payload invariants apply on both sides of application mapping.
        use crate::operation::{OperationEvent, OperationEventKind as E};
        let payload = match &kind {
            ProviderEventKind::Started => E::Started,
            ProviderEventKind::TextDelta(text) => E::TextDelta(text.clone()),
            ProviderEventKind::UsageUpdated(usage) => E::UsageUpdated(*usage),
            ProviderEventKind::Completed(response) => E::Completed(response.clone()),
            ProviderEventKind::Failed(error) => E::Failed(error.clone().into()),
            ProviderEventKind::Cancelled(remote) => E::Cancelled(*remote),
        };
        OperationEvent::new(correlation.clone(), 1, payload, max_payload_bytes)?;
        Ok(Self { correlation, kind })
    }
    pub fn correlation(&self) -> &Correlation {
        &self.correlation
    }
    pub fn kind(&self) -> &ProviderEventKind {
        &self.kind
    }
}

/// Adapters translate protocols; they do not manage conversations or run tools.
/// Implementations must observe context cancellation/deadline, preserve correlation
/// and produce one stream terminal. The future application must supervise these
/// promises: the trait alone cannot enforce I/O behavior or remote termination.
pub trait AiProvider: Send + Sync {
    /// Local declaration only: never network discovery.
    fn declared_capabilities(&self) -> AdapterCapabilities;
    /// May require discovery; caller owns authorization, reuse and invalidation.
    fn capabilities(
        &self,
        model: ModelRef,
        context: OperationContext,
    ) -> BoxFuture<'_, Result<ProviderCapabilities, ProviderError>>;
    fn complete(
        &self,
        request: AiRequest,
        context: OperationContext,
    ) -> BoxFuture<'_, Result<AiResponse, ProviderError>>;
    fn stream(
        &self,
        request: AiRequest,
        context: OperationContext,
    ) -> BoxFuture<'_, Result<ProviderStream, ProviderError>>;
}
