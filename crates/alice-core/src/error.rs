//! Stable categories and fixed, UI-safe messages. Raw diagnostics belong outside Core.
use std::{error::Error, fmt};

macro_rules! category {
    ($code:ident, $error:ident, $prefix:literal, {$($variant:ident => ($name:literal, $message:literal)),+ $(,)?}) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $code { $($variant),+ }
        impl $code {
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => concat!($prefix, ".", $name)),+ }
            }
            pub const fn safe_message(self) -> &'static str {
                match self { $(Self::$variant => $message),+ }
            }
        }
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $error { code: $code }
        impl $error {
            pub const fn new(code: $code) -> Self { Self { code } }
            pub const fn code(&self) -> $code { self.code }
            pub const fn safe_message(&self) -> &'static str { self.code.safe_message() }
        }
        impl fmt::Display for $error {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}: {}", self.code.as_str(), self.safe_message())
            }
        }
        impl Error for $error {}
    };
}

category!(ValidationCode, ValidationError, "validation", {
    InvalidIdentifier => ("invalid_identifier", "El identificador no es válido."),
    EmptyContent => ("empty_content", "El mensaje no puede estar vacío."),
    ContentLimit => ("content_limit", "El contenido supera el límite permitido."),
    InvalidLimit => ("invalid_limit", "El límite debe ser finito y mayor que cero."),
    DuplicateMessage => ("duplicate_message", "El identificador de mensaje ya existe."),
    InvalidRole => ("invalid_role", "El rol no es válido en esta posición."),
    InvalidRevision => ("invalid_revision", "La revisión no es válida."),
    CorrelationMismatch => ("correlation_mismatch", "Los identificadores no corresponden a la solicitud."),
    InvalidSequence => ("invalid_sequence", "La secuencia de eventos no es válida."),
    InvalidEvidence => ("invalid_evidence", "La evidencia de capacidades no es válida."),
    UnsupportedCapability => ("unsupported_capability", "La capacidad solicitada no está soportada."),
    UnknownCapability => ("unknown_capability", "La capacidad solicitada no está verificada.")
});
category!(OperationCode, OperationError, "operation", {
    InvalidTransition => ("invalid_transition", "La operación no permite esa transición."),
    CorrelationMismatch => ("correlation_mismatch", "El evento pertenece a otra operación."),
    InvalidSequence => ("invalid_sequence", "El evento está fuera de secuencia."),
    SequenceExhausted => ("sequence_exhausted", "Se agotó la secuencia de la operación."),
    RevisionConflict => ("revision_conflict", "La conversación cambió; actualiza su estado."),
    RevisionExhausted => ("revision_exhausted", "Se agotó la revisión de la conversación."),
    CapacityExceeded => ("capacity_exceeded", "Se alcanzó la capacidad permitida."),
    Timeout => ("timeout", "La operación superó el tiempo permitido."),
    Internal => ("internal", "No se pudo completar la operación.")
});
category!(ProviderCode, ProviderFault, "provider", {
    InvalidRequest => ("invalid_request", "La solicitud al proveedor no es válida."),
    UnsupportedCapability => ("unsupported_capability", "El proveedor no admite la capacidad requerida."),
    Authentication => ("authentication", "No se pudo autenticar con el proveedor."),
    Authorization => ("authorization", "El proveedor no autorizó la solicitud."),
    Unavailable => ("unavailable", "El proveedor no está disponible."),
    ModelUnavailable => ("model_unavailable", "El modelo no está disponible."),
    RateLimited => ("rate_limited", "Se alcanzó un límite del proveedor."),
    ContextExceeded => ("context_exceeded", "El contexto supera el límite del modelo."),
    Timeout => ("timeout", "El proveedor superó el tiempo permitido."),
    Protocol => ("protocol", "El proveedor no entregó una respuesta válida y completa."),
    Internal => ("internal", "No se pudo obtener la respuesta del proveedor.")
});
category!(StorageCode, StorageError, "storage", {
    NotFound => ("not_found", "La conversación no se encuentra disponible."),
    RevisionConflict => ("revision_conflict", "La revisión almacenada no coincide."),
    Unavailable => ("unavailable", "El almacenamiento no está disponible."),
    CapacityExceeded => ("capacity_exceeded", "El almacenamiento alcanzó su capacidad."),
    InvalidData => ("invalid_data", "Los datos almacenados no son válidos.")
});

use crate::operation::OperationId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderError {
    operation_id: OperationId,
    fault: ProviderFault,
}
impl ProviderError {
    pub fn new(operation_id: OperationId, code: ProviderCode) -> Self {
        Self {
            operation_id,
            fault: ProviderFault::new(code),
        }
    }
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    pub fn code(&self) -> ProviderCode {
        self.fault.code()
    }
    pub fn safe_message(&self) -> &'static str {
        self.fault.safe_message()
    }
}
impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fault.fmt(f)
    }
}
impl Error for ProviderError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreError {
    Validation(ValidationError),
    Operation(OperationError),
    Provider(ProviderError),
    Storage(StorageError),
}
impl CoreError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(e) => e.code().as_str(),
            Self::Operation(e) => e.code().as_str(),
            Self::Provider(e) => e.code().as_str(),
            Self::Storage(e) => e.code().as_str(),
        }
    }
    pub fn safe_message(&self) -> &'static str {
        match self {
            Self::Validation(e) => e.safe_message(),
            Self::Operation(e) => e.safe_message(),
            Self::Provider(e) => e.safe_message(),
            Self::Storage(e) => e.safe_message(),
        }
    }
}
impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code(), self.safe_message())
    }
}
impl Error for CoreError {}
macro_rules! conversion {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for CoreError {
            fn from(error: $ty) -> Self {
                Self::$variant(error)
            }
        }
    };
}
conversion!(ValidationError, Validation);
conversion!(OperationError, Operation);
conversion!(ProviderError, Provider);
conversion!(StorageError, Storage);

pub(crate) fn validate_identifier(value: &str) -> Result<(), ValidationError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(ValidationError::new(ValidationCode::InvalidIdentifier));
    }
    Ok(())
}

/// Define opaque, typed identifiers; generation and global uniqueness are external.
macro_rules! identifier {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, crate::error::ValidationError> {
                let value = value.into();
                crate::error::validate_identifier(&value)?;
                Ok(Self(value))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
        impl std::str::FromStr for $name {
            type Err = crate::error::ValidationError;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}
pub(crate) use identifier;
