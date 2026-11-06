export interface ChatRequest {
  requestId: string;
  text: string;
}

export interface ChatResponse {
  requestId: string;
  text: string;
}

export interface ChatTransport {
  send(request: ChatRequest): Promise<ChatResponse>;
}

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  text: string;
  createdAt: string;
  status: "pending" | "sent" | "failed";
}

export interface ChatFailure {
  code: string;
  message: string;
  requestId: string;
}
