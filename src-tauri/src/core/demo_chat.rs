use serde::{Deserialize, Serialize};

pub const MAX_MESSAGE_CHARS: usize = 2000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub request_id: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub request_id: String,
    pub text: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    EmptyMessage,
    MessageTooLong,
    DemoFailure,
    InternalError,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatError {
    pub request_id: String,
    pub code: ErrorCode,
    pub message: String,
}

impl ChatError {
    fn new(request_id: String, code: ErrorCode, message: &str) -> Self {
        Self {
            request_id,
            code,
            message: message.to_owned(),
        }
    }

    pub fn internal(request_id: String) -> Self {
        Self::new(
            request_id,
            ErrorCode::InternalError,
            "No se pudo completar la respuesta. Intenta enviar el mensaje de nuevo.",
        )
    }
}

pub fn reply(request: ChatRequest) -> Result<ChatResponse, ChatError> {
    let id = request.request_id;
    if id.trim().is_empty() || id.len() > 128 {
        return Err(ChatError::new(
            id,
            ErrorCode::InvalidRequest,
            "La solicitud no tiene un identificador válido.",
        ));
    }
    let text = request.text.trim();
    if text.is_empty() {
        return Err(ChatError::new(
            id,
            ErrorCode::EmptyMessage,
            "Escribe un mensaje antes de enviarlo.",
        ));
    }
    if text.chars().count() > MAX_MESSAGE_CHARS {
        return Err(ChatError::new(
            id,
            ErrorCode::MessageTooLong,
            "El mensaje supera el límite de 2000 caracteres.",
        ));
    }
    if text == "/error" {
        return Err(ChatError::new(
            id,
            ErrorCode::DemoFailure,
            "Error de prueba recibido. El chat sigue disponible; puedes enviar otro mensaje.",
        ));
    }
    Ok(ChatResponse {
        request_id: id,
        text: format!(
            "Te recibo. Tu mensaje llegó correctamente:\n\n«{text}»\n\nEsta respuesta de prueba se generó en Rust. Todavía no hay un modelo de IA conectado."
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(text: &str) -> ChatRequest {
        ChatRequest {
            request_id: "test-request".into(),
            text: text.into(),
        }
    }

    #[test]
    fn preserves_correlation_and_multiline_content() {
        let response = reply(request("  primera línea\nsegunda línea  ")).unwrap();
        assert_eq!(response.request_id, "test-request");
        assert!(response.text.contains("«primera línea\nsegunda línea»"));
    }

    #[test]
    fn rejects_whitespace_without_a_successful_reply() {
        assert_eq!(
            reply(request(" \n\t ")).unwrap_err().code,
            ErrorCode::EmptyMessage
        );
    }

    #[test]
    fn limits_characters_not_utf8_bytes() {
        assert!(reply(request(&"ñ".repeat(MAX_MESSAGE_CHARS))).is_ok());
        assert_eq!(
            reply(request(&"ñ".repeat(MAX_MESSAGE_CHARS + 1)))
                .unwrap_err()
                .code,
            ErrorCode::MessageTooLong
        );
    }

    #[test]
    fn demonstration_failure_is_a_typed_error() {
        let error = reply(request("/error")).unwrap_err();
        assert_eq!(error.code, ErrorCode::DemoFailure);
        let json = serde_json::to_value(error).unwrap();
        assert_eq!(json["code"], "demo_failure");
        assert_eq!(json["requestId"], "test-request");
    }

    #[test]
    fn rejects_invalid_correlation_id() {
        let error = reply(ChatRequest {
            request_id: String::new(),
            text: "hola".into(),
        })
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidRequest);
    }
}
