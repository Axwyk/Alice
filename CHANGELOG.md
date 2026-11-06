# Changelog

## Fase 1 — Etapa 1: contratos de Core — 2026-11-06

- Crate independiente `crates/alice-core` con identificadores tipados, mensajes y conversaciones validados, revisiones y reglas puras de operaciones/eventos.
- Contratos neutrales de proveedor, capacidades con evidencia/vigencia, errores seguros y puerto de almacenamiento sin implementación.
- Futuros estándar y señal de cancelación independiente de runtime; única dependencia `futures-core` para `Stream`.
- 42 pruebas offline aprobadas y verificación Cargo de ausencia de Tauri, Tokio y tokio-util en el grafo resuelto.
- Reporte técnico en `docs/PHASE_1_STAGE_1.md`. Sin conexión del nuevo crate al escritorio ni funcionalidades de etapas posteriores.

## Icono y avatares sin fondo — 2026-11-06

- Iconos PNG, ICO e ICNS con transparencia real, sin relleno oscuro.
- Avatares de interfaz sin fondos, marcos, halos ni sombras, incluidos los estados de procesamiento.
- Se conservan los píxeles y el canal alfa de los recortes del personaje.


## Corrección del icono nativo — 2026-11-06

- La compilación detecta cambios en icons/ y vuelve a incrustar el ICO actualizado en el ejecutable de Windows y su ventana.
- El acceso directo existente de Alice en el escritorio utiliza el ICO actual del proyecto.


## Retratos transparentes — 2026-11-06

- Nueva hoja de expresiones con transparencia original conservada en los avatares y el rostro de los iconos nativos.
- Camafeo oscuro, encuadre sin recortar el cabello y recurso independiente de ojos cerrados para el parpadeo.
- La ruta reservada enojada reutiliza el retrato serio hasta disponer de un busto enojado independiente.
- Se conserva la API pública de AliceAvatar y la lógica de conversación.

## Rediseño visual — 2026-11-05

- Identidad de Alice con seis retratos extraídos de la hoja original, camafeo de bienvenida e iconos nativos del rostro.
- Componente AliceAvatar con estados, crossfade, saludo, parpadeo y fallback; respeta movimiento reducido.
- Tokens centralizados, Cormorant Garamond y Source Sans 3 locales, encaje discreto y motivos del vestido.
- Composición adaptable, mensajes con avatar, foco visible, contraste AA y lema de Alice.
- Se conserva el flujo de chat e IPC existente. La confirmación de reinicio devuelve el foco después de cerrar el diálogo.
- Documentación de recursos y de la futura sustitución del renderer por Live2D en docs/VISUAL_DESIGN.md.

## 0.1.0 — 2026-11-05

### Añadido

- Base independiente de escritorio con Tauri 2, React, TypeScript, Vite y Tailwind CSS.
- Chat oscuro inspirado en carbón, marfil y azul grisáceo de la referencia de Alice, sin emojis.
- Mensajes diferenciados, historial de sesión, envío por Enter/botón y salto de línea mediante Shift + Enter.
- Intercambio IPC real con respuestas de demostración en Rust, validación Unicode y errores tipados.
- Estado de procesamiento, recuperación visual de errores y confirmación antes de limpiar la conversación.
- Contratos y fronteras para futuras integraciones con IA y voz; arquitectura independiente de proveedores.
- Lockfiles, exclusión de builds/dependencias/secretos y configuración de contenido/ventana.
- Informe del Paso C con TypeScript/build correctos, cinco pruebas Rust aprobadas, comprobación nativa del chat e IPC y ejecutable de producción verificado en Windows.

En el Paso C original no se incluían proveedores de IA, memoria persistente, voz ni avatar.
