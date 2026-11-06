# Alice 2.0

Aplicación de escritorio con chat gráfico. **Versión 0.1.0, Fase 0 / Paso C**: las respuestas son de demostración, generadas en Rust mediante IPC real. No hay un modelo de IA conectado.

## Uso

- Escribe un mensaje y pulsa **Enter** o **Enviar**. **Shift + Enter** inserta una nueva línea.
- El historial permanece en memoria durante la sesión. No se guarda al cerrar la aplicación.
- El estado de procesamiento refleja una solicitud IPC pendiente. La demostración añade 1,2 segundos de latencia en Rust para poder observarlo.
- Usa **Comprobar un error** al inicio o envía `/error` para recibir un error tipado real de Rust. La prueba no desconecta la aplicación; puedes enviar después un mensaje normal.
- **Nueva conversación** pide confirmación antes de descartar el historial actual.
- El límite es de 2000 caracteres Unicode por mensaje, validado también en Rust.

## Desarrollo

Requisitos: Node/npm compatibles con Vite, Rust stable MSVC y C++ Build Tools/Windows SDK en Windows, además de WebView2. El manifiesto Rust declara 1.90 como mínimo de resolución; la versión verificada se registra en el informe. Consulta los [prerrequisitos oficiales de Tauri](https://v2.tauri.app/start/prerequisites/) para Linux y macOS.

Desde la raíz del proyecto, en PowerShell:

```powershell
npm ci
npm run tauri dev
```

En terminales que ya estaban abiertas al instalar Rust puede ser necesario reiniciar la terminal o el editor para recoger el PATH configurado por Rustup. No hace falta editar el PATH manualmente.

La interacción de chat debe probarse **dentro de Tauri**. `npm run dev` por sí solo sirve el frontend de Vite; no crea un backend IPC ni simula respuestas en el navegador.

## Comprobaciones

```powershell
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --locked --jobs 2
npm run tauri build -- --no-bundle
```

La última orden compila el ejecutable de producción sin generar instaladores. En Windows, la salida normal es `src-tauri/target/release/alice2.exe`. El ejecutable de producción no abre una consola para interactuar.

## Estructura principal

| Archivo | Responsabilidad |
|---|---|
| `src/App.tsx` | Ventana de chat, mensajes, compositor y estados visuales |
| `src/styles.css` | Tailwind y diseño inspirado en la paleta de Alice; sin fuentes remotas ni emojis |
| `src/features/chat/types.ts` | Contratos de transporte y estado del chat |
| `src/features/chat/useChat.ts` | Historial de sesión, operación pendiente, correlación y errores |
| `src/services/chatTransport.ts` | Única frontera frontend con `invoke` de Tauri |
| `src-tauri/src/commands.rs` | Comando IPC asíncrono de demostración |
| `src-tauri/src/core/demo_chat.rs` | Validación y respuesta determinista, sin dependencia de Tauri ni proveedores |
| `src-tauri/tauri.conf.json` | Ventana principal, build y políticas de contenido |
| `docs/ARCHITECTURE.md` | Independencia de proveedores y decisiones arquitectónicas permanentes |
| `docs/CHAT_INTEGRATION.md` | Fronteras actuales y futuras integraciones con IA/voz |

Los lockfiles de npm y Cargo pertenecen al proyecto. No se requieren Docker, Ollama, servicios externos ni herramientas npm globales para esta demostración.

## Alcance y portabilidad

Solo se implementa el chat de prueba. IA real, memoria/SQLite, personalidad funcional, agentes, herramientas, voz, avatar y tareas programadas quedan fuera de esta versión.

El código de aplicación no contiene rutas de esta máquina ni escribe datos en el directorio de instalación. Los prerrequisitos y almacenes de credenciales futuros serán adaptadores específicos de plataforma. Que el código esté preparado para Windows/Linux/macOS no equivale a haberlo probado en las tres plataformas; los resultados reales se registran en el informe del Paso C.

Alice 1.0 no se integra ni se importa. Este es un proyecto independiente.

El [informe del Paso C](docs/PHASE_0_STEP_C.md) registra versiones, pruebas reales, incidencias e integridad del proyecto histórico.
