# Chat: implementación actual y puntos de integración

## Implementado en el Paso C

```text
Texto / Enter / botón
  → useChat.sendMessage(text)
  → ChatTransport.send({ requestId, text })
  → invoke("send_demo_message", { request })
  → comando Tauri asíncrono
  → servicio puro Rust core::demo_chat::reply
  → respuesta o error serializable
  → actualización del historial y estados de React
```

El transporte devuelve una promesa real de Tauri. No hay una respuesta falsa en JavaScript para aparentar éxito cuando se abre el frontend en un navegador. El ID de correlación se valida antes de aceptar una respuesta. Los errores se muestran fuera de los mensajes de Alice y el mensaje del usuario queda marcado como no respondido.

El servicio Rust no tiene acceso a red, archivos, memoria persistente, credenciales o proveedores. Sus tests validan texto vacío, límite Unicode, correlación, contenido multilínea y error de demostración. El comando añade una latencia de 1,2 segundos en un worker para hacer observable el procesamiento, sin bloquear el hilo principal. Esa latencia no pertenece al servicio puro ni deberá trasladarse a una inferencia real.

El historial vive exclusivamente en React durante esta sesión. El backend responde a cada solicitud de forma independiente. No se afirma memoria conversacional ni razonamiento de IA. Nueva conversación se desactiva mientras hay una solicitud pendiente; un identificador de operación impide aplicar respuestas de operaciones ya descartadas. No se implementa todavía aborto del trabajo Rust.

## Integración posterior con IA

`ChatTransport` es la frontera **de transporte de la interfaz**, no la AI Provider Interface. En la fase de IA, el comando Rust podrá delegar en un servicio de conversación que use el puerto de proveedores documentado en `ARCHITECTURE.md`. El chat seguirá consumiendo los contratos de aplicación y no importará SDK ni credenciales.

El cambio deberá conservar request IDs y errores tipados. La introducción de streaming añadirá eventos del contrato de aplicación con secuencia y un único terminal; la UI no interpretará payloads de proveedores. La cancelación tendrá semántica explícita desde interfaz hasta adaptador. No se modela una cancelación ficticia en esta versión.

La selección de contexto, personalidad, memoria, agente y permisos de herramientas seguirá separada de los adaptadores. La configuración de proveedor, modelo y endpoint se resolverá en Rust. Ollama será el primer adaptador futuro; no se ha añadido su dependencia.

## Integración posterior con voz

No hay motores, permisos de micrófono, grabación ni componentes vacíos de voz.

- **STT:** una captura explícita producirá texto revisable y utilizará la misma acción `sendMessage(text)` del chat. La captura no pertenecerá al componente textarea ni al transporte de proveedor.
- **TTS:** consumirá respuestas de Alice confirmadas por la capa de conversación, con IDs para evitar reproducción duplicada; no leerá errores como si fueran respuestas normales.
- **Interrupción:** la reproducción tendrá su propia operación cancelable. Detener audio y cancelar inferencia serán decisiones distintas y visibles.
- **Estados:** escucha, transcripción y reproducción se añadirán cuando existan eventos reales que los respalden. El único estado de trabajo implementado ahora es la solicitud IPC pendiente.

## Portabilidad y almacenamiento

Las rutas de desarrollo y los comandos usados por esta máquina se guardan en evidencias externas, no en la lógica de aplicación. El código y los assets son relativos al proyecto. Tauri administra la ventana nativa y el transporte por plataforma.

No hay persistencia en el Paso C. Cuando se autorice, configuración/datos deberán usar los directorios de aplicación del usuario obtenidos mediante la API de plataforma de Tauri; Core recibirá un contrato de almacenamiento y no calculará rutas del sistema. No se escribirá en el directorio de instalación. Los secretos utilizarán el almacén protegido disponible por plataforma y no el frontend ni una base en texto plano.

El build actual solo podrá acreditar Windows cuando se verifique. Linux requiere sus bibliotecas del sistema y macOS sus herramientas de desarrollo; sus compilaciones y pruebas quedan pendientes. Docker no es necesario.
