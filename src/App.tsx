import { useEffect, useRef, useState } from "react";
import type { FormEvent, KeyboardEvent } from "react";
import { AliceAvatar } from "./components/AliceAvatar";
import type { AliceAvatarState } from "./components/AliceAvatar";
import { Icon } from "./components/Icons";
import { useChat } from "./features/chat/useChat";
import type { ChatMessage } from "./features/chat/types";
import { tauriChatTransport } from "./services/chatTransport";

const MAX_CHARS = 2000;
const timeFormat = new Intl.DateTimeFormat("es-CO", { hour: "2-digit", minute: "2-digit" });
const presenceLabels: Record<AliceAvatarState, string> = {
  reposo: "Alice está aquí",
  saludo: "Me alegra verte",
  escribiendo: "Te escucho",
  pensando: "Preparando tu respuesta",
  lista: "Lista para seguir",
  error: "Podemos intentarlo de nuevo",
};

function Brooch({ className = "" }: { className?: string }) {
  return <svg className={className} width="28" height="16" viewBox="0 0 28 16" fill="none" aria-hidden="true"><path d="M12 8C8 2 2 2 2 5s6 7 10 3Zm4 0c4-6 10-6 10-3s-6 7-10 3ZM12 10l-3 4m7-4 3 4" stroke="currentColor" strokeWidth="1" /><path d="m14 4 3 4-3 4-3-4 3-4Z" fill="currentColor" /></svg>;
}

function Message({ message }: { message: ChatMessage }) {
  const user = message.role === "user";
  return (
    <article className={`message ${user ? "message--user" : "message--alice"}`} aria-label={`Mensaje de ${user ? "usuario" : "Alice"}`}>
      {!user && <AliceAvatar size="small" estado="lista" className="message-portrait" />}
      <div className="message-body">
        <div className="message-meta"><span className="message-author">{user ? "Tú" : "Alice"}</span><time dateTime={message.createdAt}>{timeFormat.format(new Date(message.createdAt))}</time></div>
        <div className="message-text">{message.text}</div>
        {message.status === "failed" && <span className="message-failed">Sin respuesta</span>}
      </div>
    </article>
  );
}

export default function App() {
  const { messages, isProcessing, error, sendMessage, resetConversation, dismissError } = useChat(tauriChatTransport);
  const [draft, setDraft] = useState("");
  const [showReset, setShowReset] = useState(false);
  const [greeting, setGreeting] = useState(true);
  const textarea = useRef<HTMLTextAreaElement>(null);
  const feedEnd = useRef<HTMLDivElement>(null);
  const resetDialog = useRef<HTMLDialogElement>(null);
  const characters = Array.from(draft).length;
  const overLimit = characters > MAX_CHARS;
  const firstMessage = messages.find(message => message.role === "user")?.text.split("\n")[0];
  const conversationTitle = firstMessage ? Array.from(firstMessage).slice(0, 35).join("") : "Conversación nueva";
  const lastMessage = messages[messages.length - 1];
  const avatarState: AliceAvatarState = error ? "error" : isProcessing ? "pensando" : draft.trim() ? "escribiendo" : lastMessage?.role === "assistant" ? "lista" : greeting ? "saludo" : "reposo";

  useEffect(() => {
    if (messages.length !== 0) return;
    setGreeting(true);
    const timer = window.setTimeout(() => setGreeting(false), 1800);
    return () => window.clearTimeout(timer);
  }, [messages.length]);

  useEffect(() => {
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    feedEnd.current?.scrollIntoView({ behavior: reduceMotion ? "auto" : "smooth", block: "end" });
  }, [messages, isProcessing, error]);

  useEffect(() => {
    const dialog = resetDialog.current;
    if (showReset && dialog && !dialog.open) dialog.showModal();
    if (!showReset && dialog?.open) dialog.close();
  }, [showReset]);

  async function submitText(value: string) {
    const text = value.trim();
    if (!text || isProcessing || Array.from(text).length > MAX_CHARS) return;
    setDraft("");
    const success = await sendMessage(text);
    if (!success) setDraft(current => current || text);
    textarea.current?.focus();
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void submitText(draft);
  }

  function handleKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
      event.preventDefault();
      void submitText(draft);
    }
  }

  function startNew() {
    if (isProcessing) return;
    if (messages.length || draft.trim()) setShowReset(true);
    else textarea.current?.focus();
  }

  function confirmNew() {
    resetDialog.current?.close();
    resetConversation();
    setDraft("");
    setShowReset(false);
    textarea.current?.focus();
  }

  return (
    <div className="app-shell">
      <aside className="sidebar" aria-label="Sesión de Alice">
        <div className="brand"><AliceAvatar estado={avatarState} size="brand" /><div><span className="brand-name">alice</span><span className="brand-caption">Un espacio contigo</span></div></div>
        <button className="new-chat" onClick={startNew} disabled={isProcessing}><Icon name="plus" />Nueva conversación</button>
        <div className="session-section">
          <h2>Esta sesión</h2>
          <div className="session-item" aria-current="page"><Icon name="message" /><div><span className="session-title" title={conversationTitle}>{conversationTitle}</span><span className="session-count">{messages.length ? `${messages.length} ${messages.length === 1 ? "mensaje" : "mensajes"}` : "Un comienzo en blanco"}</span></div><span className="session-dot" /></div>
        </div>
        <div className="sidebar-bottom">
          <div className="ornament-divider"><Brooch /></div>
          <div className="demo-label"><span className="status-dot" />Modo de prueba</div>
          <p>Respuestas locales de demostración.<br />Sin un modelo de IA conectado.</p>
          <span className="version-label">Alice 2.0 <span>Fase inicial</span></span>
        </div>
      </aside>

      <main className="chat-main">
        <header className="chat-header">
          <div className="header-title"><div className="mobile-brand"><AliceAvatar estado={avatarState} size="small" /><span>Alice</span></div><h1>Conversación</h1><span className="header-caption">Un lugar para tus ideas</span></div>
          <div className="header-right"><span className="session-notice"><Icon name="session" />Solo esta sesión</span><button className="mobile-new icon-button" onClick={startNew} disabled={isProcessing} aria-label="Nueva conversación"><Icon name="plus" /></button></div>
        </header>

        <section className={`conversation ${messages.length ? "has-messages" : "is-empty"}`} aria-label="Conversación con Alice">
          {messages.length === 0 ? (
            <div className="welcome">
              <div className="welcome-copy">
                <p className="welcome-greeting">Hola, soy Alice.</p>
                <h2>Todo empieza<br />con un mensaje.</h2>
                <p className="welcome-description">Comparte una idea o empieza con un hola.<br className="desktop-break" /> Estoy aquí para acompañarte.</p>
                <div className="welcome-presence" role="status"><span className="status-dot" /><span>{presenceLabels[avatarState]}</span></div>
              </div>
              <figure className="character-portrait" aria-label="Retrato de Alice" data-character-slot="welcome">
                <AliceAvatar estado={avatarState} size="hero" />
              </figure>
              <div className="welcome-divider"><span />¿Por dónde empezamos?<span /></div>
              <div className="suggestions">
                <button className="suggestion-primary" onClick={() => void submitText("Hola, Alice. ¿Me recibes?")}><span className="suggestion-icon"><Icon name="message" /></span><span><strong>Empezar a conversar</strong><span>Un primer hola es suficiente</span></span></button>
                <button onClick={() => void submitText("/error")}><span className="suggestion-icon"><Icon name="alert" /></span><span><strong>Comprobar un error</strong><span>Prueba la recuperación del chat</span></span></button>
              </div>
            </div>
          ) : (
            <div className="message-feed" role="log" aria-live="polite" aria-relevant="additions text" aria-label="Historial de mensajes">
              <div className="day-separator"><span>Hoy</span></div>
              {messages.map(message => <Message key={message.id} message={message} />)}
              {isProcessing && <div className="processing" role="status"><AliceAvatar size="small" estado="pensando" /><span>Alice está preparando tu respuesta</span></div>}
              {error && <div className="chat-error" role="alert"><AliceAvatar size="small" estado="error" /><div><strong>No se completó la respuesta</strong><p>{error.message}</p></div><button className="icon-button" onClick={dismissError} aria-label="Descartar aviso de error"><Icon name="close" /></button></div>}
              <div ref={feedEnd} />
            </div>
          )}
        </section>

        <div className="composer-area">
          <form onSubmit={submit} className={`composer ${overLimit ? "composer--invalid" : ""}`} aria-label="Enviar un mensaje">
            <label className="sr-only" htmlFor="message-input">Mensaje para Alice</label>
            <textarea ref={textarea} id="message-input" autoFocus rows={2} value={draft} onChange={event => setDraft(event.target.value)} onKeyDown={handleKeyDown} placeholder="Escribe a Alice…" aria-describedby="composer-hint composer-limit" aria-invalid={overLimit} spellCheck />
            <div className="composer-toolbar"><span className="composer-mode"><Brooch />Conversación de prueba</span><div className="composer-actions"><span id="composer-limit" className={`character-count ${overLimit ? "over-limit" : ""}`} aria-live={overLimit ? "polite" : "off"}>{characters > 1600 ? `${characters} / ${MAX_CHARS}` : ""}</span><button type="submit" className="send-button" disabled={isProcessing || !draft.trim() || overLimit} aria-label="Enviar mensaje">{isProcessing ? "Enviando" : "Enviar"}<Icon name="send" /></button></div></div>
          </form>
          <div className="composer-footnote"><span id="composer-hint"><kbd>Enter</kbd> para enviar <span className="keyboard-separator" /><kbd>Shift + Enter</kbd> para una nueva línea</span><span className="ephemeral-note">Sin guardado permanente</span></div>
          <p className="alice-motto">Tu asistente virtual, siempre contigo.</p>
        </div>
      </main>

      <dialog ref={resetDialog} className="reset-dialog" aria-labelledby="reset-title" aria-describedby="reset-description" onCancel={event => { event.preventDefault(); setShowReset(false); }}>
        <Brooch className="dialog-brooch" /><h2 id="reset-title">¿Empezamos de nuevo?</h2><p id="reset-description">Los mensajes de esta conversación se borrarán. No hay guardado permanente en esta versión.</p><div><button className="secondary-button" onClick={() => setShowReset(false)} autoFocus>Conservar conversación</button><button className="primary-button" onClick={confirmNew}>Empezar de nuevo</button></div>
      </dialog>
    </div>
  );
}
