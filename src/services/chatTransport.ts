import { invoke } from "@tauri-apps/api/core";
import type { ChatResponse, ChatTransport } from "../features/chat/types";

// This boundary is the only frontend module that knows the IPC command.
// There is deliberately no browser mock or provider SDK fallback.
export const tauriChatTransport: ChatTransport = {
  async send(request) {
    const response = await invoke<ChatResponse>("send_demo_message", { request });
    if (response.requestId !== request.requestId || typeof response.text !== "string") {
      throw {
        code: "invalid_response",
        message: "La respuesta no corresponde a este mensaje. Intenta enviarlo de nuevo.",
        requestId: request.requestId,
      };
    }
    return response;
  },
};
