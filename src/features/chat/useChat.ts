import { useEffect, useRef, useState } from "react";
import type { ChatFailure, ChatMessage, ChatTransport } from "./types";

function normalizeFailure(error: unknown, requestId: string): ChatFailure {
  if (
    typeof error === "object" && error !== null &&
    "code" in error && typeof error.code === "string" &&
    "message" in error && typeof error.message === "string"
  ) {
    return { code: error.code, message: error.message, requestId };
  }
  return {
    code: "transport_error",
    message: "No se pudo contactar con Alice. Abre la aplicación de escritorio y vuelve a intentarlo.",
    requestId,
  };
}

export function useChat(transport: ChatTransport) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<ChatFailure | null>(null);
  const busy = useRef(false);
  const operation = useRef(0);

  useEffect(() => () => { operation.current += 1; }, []);

  async function sendMessage(rawText: string): Promise<boolean> {
    const text = rawText.trim();
    if (!text || busy.current) return false;
    const requestId = crypto.randomUUID();
    const currentOperation = ++operation.current;
    busy.current = true;
    setIsProcessing(true);
    setError(null);
    setMessages(previous => [...previous, {
      id: requestId,
      role: "user",
      text,
      createdAt: new Date().toISOString(),
      status: "pending",
    }]);

    try {
      const response = await transport.send({ requestId, text });
      if (operation.current !== currentOperation) return false;
      setMessages(previous => [
        ...previous.map(message => message.id === requestId ? { ...message, status: "sent" as const } : message),
        { id: `${requestId}-reply`, role: "assistant", text: response.text, createdAt: new Date().toISOString(), status: "sent" },
      ]);
      return true;
    } catch (failure: unknown) {
      if (operation.current !== currentOperation) return false;
      setMessages(previous => previous.map(message => message.id === requestId ? { ...message, status: "failed" as const } : message));
      setError(normalizeFailure(failure, requestId));
      return false;
    } finally {
      if (operation.current === currentOperation) {
        busy.current = false;
        setIsProcessing(false);
      }
    }
  }

  function resetConversation() {
    operation.current += 1;
    busy.current = false;
    setIsProcessing(false);
    setMessages([]);
    setError(null);
  }

  return { messages, isProcessing, error, sendMessage, resetConversation, dismissError: () => setError(null) };
}
