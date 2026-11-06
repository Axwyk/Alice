# Fase 0 — Paso C: informe de implementación y verificación

Fecha: 5 de noviembre de 2026. Aplicación: Alice 2.0, versión 0.1.0.

## Estado

**Paso C completado en Windows x64.** La aplicación abre como ventana gráfica, intercambia mensajes con Rust mediante IPC y compila en producción. Las comprobaciones obligatorias terminaron correctamente. Este documento no autoriza fases posteriores.

## Estructura creada

Proyecto independiente de Alice 1.0, sin importar su código ni sus datos:

```text
Alice2/
  .gitignore
  .vscode/extensions.json
  README.md
  CHANGELOG.md
  package.json / package-lock.json
  index.html / vite.config.ts / tsconfig*.json
  src/
    main.tsx / App.tsx / styles.css
    components/Icons.tsx
    features/chat/types.ts / useChat.ts
    services/chatTransport.ts
  src-tauri/
    Cargo.toml / Cargo.lock / build.rs
    tauri.conf.json
    capabilities/default.json
    icons/
    src/main.rs / lib.rs / commands.rs
    src/core/mod.rs / demo_chat.rs
  docs/
    ARCHITECTURE.md
    CHAT_INTEGRATION.md
    PHASE_0_STEP_C.md
```

`node_modules`, `dist`, los esquemas generados y `src-tauri/target` son artefactos excluidos de Git. Los dos lockfiles se conservan. Se retiraron los logotipos y el CSS de ejemplo que no utiliza la interfaz.

## Herramientas y versiones comprobadas

| Componente | Versión observada |
|---|---|
| Node.js | 24.15.0 |
| npm | 12.2.0 |
| Rustup | 1.29.3 |
| rustc | 1.99.1, commit 58cf8f9c3 |
| Cargo | 1.99.1 |
| Toolchain activa | stable-x86_64-pc-windows-msvc, predeterminada |
| Target instalado | x86_64-pc-windows-msvc, único target |
| Tauri CLI y API JavaScript | 2.12.4 |
| Tauri Rust | 2.12.5 |
| tauri-build | 2.7.4 |

Las compilaciones Rust de este paso utilizan y validan el entorno MSVC del Paso A. No se instalaron toolchains GNU ni targets adicionales. El manifiesto declara Rust 1.90 como mínimo de resolución de dependencias; la versión realmente probada es 1.99.1.

## Dependencias directas instaladas

| npm | Versión resuelta |
|---|---|
| react / react-dom | 19.3.1 |
| @types/react / @types/react-dom | 19.3.0 |
| typescript | 6.0.3 |
| vite | 8.3.6 |
| @vitejs/plugin-react | 6.2.1 |
| tailwindcss / @tailwindcss/vite | 4.3.4 |
| @tauri-apps/api / @tauri-apps/cli | 2.12.4 |

Cargo declara exclusivamente `tauri`, `serde`, `serde_json` y, como dependencia de compilación, `tauri-build`. Sus versiones transitivas quedan fijadas en `Cargo.lock`. No se añadió el plugin opener de la plantilla. No se instalaron librerías de proveedores de IA, voz, SQLite ni SQLx, ni herramientas npm globales. La instalación npm finalizó correctamente y notificó cero vulnerabilidades en ese momento; no constituye una garantía permanente.

## Interfaz y responsabilidades

La interfaz oscura toma de la referencia visual carbón `#1b1b23`, marfil `#efe5d6` y azul grisáceo `#9eb5c6`. Usa fuentes del sistema e iconos SVG, sin emojis, imágenes de avatar ni fuentes remotas.

Incluye una barra lateral de sesión, bienvenida con dos acciones funcionales, historial con mensajes del usuario a la derecha y respuestas de Alice a la izquierda, marcas horarias, compositor multilínea, botón Enviar, Enter para enviar y Shift + Enter para insertar un salto. En ventanas estrechas la cabecera reemplaza la barra lateral. El historial vive en memoria; no hay almacenamiento de conversaciones.

`App.tsx` presenta estos estados; `useChat.ts` administra la operación pendiente, los mensajes, la correlación y los errores. Un bloqueo evita envíos simultáneos. El borrador se recupera tras un error sin sobrescribir otro texto que el usuario haya empezado a escribir. Nueva conversación solicita confirmación antes de vaciar la sesión.

`chatTransport.ts` es la única frontera de `invoke` desde React. Envía un DTO con ID de solicitud y texto a `send_demo_message`. El comando asíncrono Rust delega en una tarea bloqueante y llama al servicio puro de demostración. Este servicio valida el contenido y devuelve un eco determinista con el mismo ID. La espera de 1,2 segundos existe únicamente para observar el estado pendiente de la demostración.

`/error` provoca un error tipado real desde Rust: aparece un aviso separado y el mensaje queda marcado sin respuesta. No se añade una respuesta ficticia de Alice. El servicio limita los mensajes a 2000 caracteres Unicode, rechaza contenido vacío e IDs inválidos. No ejecuta el contenido recibido como código ni como comandos del sistema.

## Configuración Tauri

- Identificador `dev.axwyk.alice2`; ventana principal `main`, título Alice, tema oscuro y tamaño inicial 1160 × 800; mínimo de contenido 640 × 520.
- Vite utiliza exclusivamente el loopback de desarrollo, puerto 1420 con puerto estricto. Producción incluye `dist` en el ejecutable.
- Capability de la ventana: `core:default`. Sin plugins de shell, archivos, apertura externa ni clientes de IA.
- CSP de producción restringida a recursos propios e IPC. El permiso de estilos inline permite el estado visual de React; la CSP de desarrollo añade el loopback/WebSocket que requiere Vite.
- El comando `send_demo_message` es la única función de negocio registrada por la aplicación.
- Producción utiliza el subsistema gráfico de Windows; el chat no depende de una consola.

## Pruebas y resultados

| Comprobación ejecutada | Resultado |
|---|---|
| `npm run check` | Correcto; TypeScript sin errores |
| `npm run build` | Correcto; 21 módulos, Vite 8.3.6 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Correcto |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked --jobs 2` | Correcto; 5 pruebas, 0 fallos; proceso termina con código 0 |
| `npm run tauri dev` | Correcto; compila y abre una ventana nativa Alice |
| React dentro de Tauri | Comprobado en ventana y árbol de accesibilidad |
| Estado inicial vacío | Bienvenida, acciones y envío deshabilitado sin contenido comprobados en producción |
| Envío Enter / botón Enviar | Ambos comprobados con respuesta Rust visible |
| Shift + Enter / mensajes multilínea | Salto sin enviar y posterior eco multilínea comprobados |
| Historial de sesión | Conserva mensajes anteriores y aumenta hasta 7 mensajes en la sesión observada |
| Procesamiento | Estado accesible “Alice está procesando”, botón Enviando y envío deshabilitado durante IPC |
| Error tipado y recuperación | `/error` muestra aviso y “Sin respuesta”; un mensaje posterior recibe respuesta normal |
| Adaptación al tamaño | Comprobada en 1160, 965 y 640 píxeles de contenido, incluida altura mínima de 520; a 640 se oculta la barra lateral y el compositor permanece accesible |
| Producción `npm run tauri build -- --no-bundle` | Correcto; perfil optimizado terminado con código 0 en 6 min 12 s; ejecutable de 9 232 896 bytes |
| Apertura e IPC del ejecutable de producción | Correcto; abre ventana, carga `http://tauri.localhost/` y devuelve el eco de “Prueba del ejecutable de producción” desde Rust |
| Git independiente / exclusiones | Correcto; rama `codex/phase-0-chat`, sin commits ni remoto; archivos nuevos sin seguimiento |
| Integridad de Alice 1.0 | 1607 archivos antes/después; SHA-256: 0 modificados, 0 añadidos, 0 eliminados |

Las cinco pruebas Rust cubren correlación y contenido multilínea, mensajes vacíos, límite Unicode, error serializado y rechazo de IDs inválidos. La interfaz se comprobó mediante la ventana de Windows, sin sustituir el transporte por un mock de navegador. El caso `/error` es un resultado esperado de prueba, no un fallo de la verificación.

La salida de producción es `src-tauri/target/release/alice2.exe`. Se lanzó directamente y se verificó el estado pendiente seguido de la respuesta visible. La ventana de desarrollo usa el loopback de Vite; la de producción fue identificada por su ejecutable `release` y su origen interno de Tauri. Se conservaron abiertas ambas sesiones de prueba para no descartar el historial de la ventana de desarrollo, que también había recibido interacción manual.

## Arquitectura y siguientes integraciones

Las decisiones permanentes están en [ARCHITECTURE.md](ARCHITECTURE.md). Alice Core dependerá de contratos neutrales de Alice; los adaptadores concretos dependerán de esos contratos. El arranque inyectará el proveedor. Core no importará Ollama, SDK comerciales, DTO HTTP ni Tauri. Proveedor, modelo, agente, personalidad, memoria y herramientas quedan separados. Streaming, cancelación, capacidades y errores están definidos como contratos de diseño, todavía sin implementación.

El transporte del chat actual es una frontera de UI/aplicación; no se confunde con la futura AI Provider Interface. La respuesta de demostración se sustituirá en el servicio de aplicación sin reconstruir la presentación. Ollama será el primer adaptador en una fase autorizada, no una dependencia estructural.

[CHAT_INTEGRATION.md](CHAT_INTEGRATION.md) documenta STT como entrada a la misma acción de envío, TTS como consumidor de respuestas confirmadas e interrupción de reproducción separada de cancelación de inferencia. No se añadieron componentes vacíos ni motores de voz.

La aplicación evita rutas de esta máquina y persistencia en el directorio de instalación. Los datos futuros utilizarán directorios de aplicación del usuario y los secretos, almacenes protegidos del sistema operativo detrás de una abstracción. Windows está probado; Linux y macOS son objetivos arquitectónicos pendientes de compilación y pruebas en sus propios entornos. Docker y servicios de IA externos no son necesarios para esta demostración.

## Comandos de preparación y ejecución

Creación original, desde el directorio padre del proyecto:

```powershell
npm exec --yes --package=create-tauri-app@4.7.5 -- create-tauri-app Alice2 --manager npm --template react-ts --identifier dev.axwyk.alice2 --tauri-version 2 --yes
```

Desde la raíz creada, se configuraron las dependencias descritas y se ejecutó `npm install`. Para reproducir las versiones fijadas y ejecutar:

```powershell
npm ci
npm run tauri dev
npm run tauri build -- --no-bundle
```

El listado completo de comprobaciones está en [README.md](../README.md). En esta sesión de Codex, cuyo PATH era anterior a Rustup, se utilizó `rustup run stable-x86_64-pc-windows-msvc` para lanzar Cargo y npm con la toolchain verificada. Rustup se invocó desde su ubicación instalada; no se alteró el PATH persistente. Una terminal nueva recoge el PATH del instalador.

## Incidencias y límites

1. El primer comando del generador confundió la versión interna del binario (4.8.0) con la del paquete npm. npm devolvió ETARGET antes de crear la carpeta. Se consultó la versión publicada del wrapper, 4.7.5, y la creación posterior terminó correctamente. No se sobrescribió un proyecto existente.
2. La plantilla advirtió que no encontraba Rust por el PATH heredado de Codex. Las herramientas sí estaban instaladas y se verificaron por su ruta. Se usó el contexto de Rustup, sin reinstalar ni editar manualmente PATH.
3. Antes de las verificaciones se ajustó el mínimo declarado de Rust a 1.90 y se actualizaron las resoluciones Cargo para mantener Tauri Rust en la familia actual compatible con CLI/API 2.12. No hubo una comprobación obligatoria fallida por ello.
4. Una consulta auxiliar de procesos mediante CIM fue denegada por el entorno. No fue necesaria para iniciar la aplicación; no se modificaron permisos del sistema. La compilación y la ventana se verificaron con sus propias salidas y la interfaz nativa.
5. La automatización de ventanas tuvo que refrescar sus observaciones tras cambios de tamaño e interacción manual. El foco accesible de WebView2 aparece a veces como documento, aunque el campo esté activo; se utilizó el control editable observado para introducir el texto. No se interpretaron estas limitaciones del inspector como fallos del chat.
6. No se generaron instaladores MSI/NSIS ni se distribuyó o instaló Alice. La compilación solicitada es el ejecutable de producción. No se afirma haber probado Linux/macOS, persistencia, inferencia, streaming ni voz.

## Preservación y próximo paso

Alice 1.0 se trató únicamente como fuente de lectura para la comprobación de integridad. No se ejecutaron sus pruebas, no se abrieron sus bases con motores de base de datos y no se importaron datos. La comparación SHA-256 incluye sus bases de datos y archivos locales existentes. Su estado Git previo se conserva.

El siguiente paso recomendado es revisar y aprobar este entregable antes de definir la fase de integración del primer proveedor sobre los contratos documentados. No se continúa automáticamente.
