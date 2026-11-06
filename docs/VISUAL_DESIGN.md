# La casa de Alice

Rediseño de presentación para la aplicación de escritorio existente. La referencia del diseño inicial fue «Asistente Virtual 2D – Live2D Model Concept». Desde el 6 de noviembre de 2026, los retratos usan la hoja PNG transparente aportada por el usuario: «Hoja de expresiones de chica gótica anime.png», conservada en docs/. No se encontró la carpeta design-refs ni una captura adicional en las fuentes disponibles; la comparación se hizo contra la interfaz React existente.

## Dirección y revisión del plan

La identidad viene del rostro, el cabello crema, los ojos azul grisáceo y el vestido oscuro. El retrato transparente es el elemento protagonista; la tipografía y el encaje permanecen discretos.

Paleta central: crema #EFE3C2, azul grisáceo #9DB4C8, marino #1F2A44, carbón #45404A, negro #1C1C21 y fondo con matiz marino #1B1D26. Los valores derivados, tamaños, familias y tiempos compartidos se concentran en src/tokens.css.

Cormorant Garamond da voz a los títulos y al lema; Source Sans 3 resuelve los controles y la conversación. Ambas familias se incluyen localmente con sus licencias OFL en public/assets/fonts; la aplicación no solicita fuentes a servicios externos.

Se compararon dos composiciones:

    A. Reserva de cuerpo completo
    [ sesión ] [ bienvenida       | modelo ]
               [ acciones         |        ]
               [ mensaje                   ]

    B. Camafeo integrado (elegida)
    [ sesión ] [ saludo y título   (Alice) ]
               [ acciones de conversación ]
               [ mensaje                   ]
               [ lema                      ]

Se eligió B porque da un hogar al personaje desde ahora, sin un panel vacío que compita con la conversación. La bienvenida y los mensajes se alinean a la izquierda; solo el retrato y el lema se centran dentro de sus zonas. En móvil, el retrato se reduce y las acciones se apilan. En ventanas bajas, la bienvenida tiene desplazamiento independiente del campo de mensaje.

La revisión descartó marcos múltiples alrededor del avatar y animaciones repetidas en las tarjetas. Tras la corrección solicitada por el usuario, el retrato se presenta como PNG sin fondo, sin filetes, marco ni halo; la malla diagonal y los broches en los controles permanecen independientes del personaje. La microinteracción de los controles ocurre al usarlos; el movimiento automático se limita al saludo, los puntos de procesamiento y el parpadeo ocasional.

## Componentes y comportamiento

- src/App.tsx: composición, presentación de mensajes y derivación del estado visual.
- src/styles.css y src/tokens.css: composición adaptable, tipografía local, color, foco y movimiento reducido.
- src/components/AliceAvatar.tsx y AliceAvatar.css: única frontera visual del personaje, con seis expresiones públicas y siete recursos internos precargados, transición de 200 ms y fallback de broche si falta un archivo.
- src/components/Icons.tsx: iconos de interfaz; se retiró la marca de letra A.
- public/assets/avatar/: seis rutas de expresión y un recurso independiente de ojos cerrados, con transparencia original.
- public/assets/fonts/: familias locales y licencias.
- index.html: favicon del rostro y color de tema.
- src-tauri/tauri.conf.json y src-tauri/icons/: fondo nativo e iconos del rostro en PNG, ICO e ICNS.

useChat, chatTransport, los contratos y el código Rust de conversación se conservan. Enter, Shift+Enter, composición IME, validación de 2000 caracteres Unicode, historial en memoria, modo de prueba y recuperación de errores mantienen su flujo. La confirmación de reinicio cierra el diálogo antes de devolver el foco al campo.

| Estado | Expresión |
| --- | --- |
| Reposo / bienvenida | Neutral |
| Usuario escribiendo | Seria |
| Respuesta pendiente | Seria y puntos discretos |
| Respuesta completada | Feliz |
| Error de chat | Triste |
| Saludo inicial | Feliz, seguido de neutral |

La prioridad visual es error, procesamiento, escritura y respuesta lista. El saludo dura brevemente. En reposo se muestra el recurso privado ojos-cerrados durante 120 ms cada 11 segundos para aproximar un parpadeo; la expresión feliz permanece independiente. Es una sustitución de retrato, no animación de párpados. Movimiento reducido desactiva ese parpadeo, las animaciones y las transiciones. Los temporizadores se limpian al desmontar.

## Recursos actuales

Los avatares usan el canal alfa de la nueva hoja PNG. No hay un fondo blanco ni se eliminó automáticamente el fondo: se conserva la transparencia proporcionada por el usuario. Los lienzos se normalizan con margen y se muestran con object-fit: contain para preservar el cabello sobre la superficie de la aplicación, sin un fondo propio.

La API pública conserva neutral, feliz, seria, sorprendida, triste y enojada. La nueva hoja no contiene un busto enojado independiente: enojada.png reutiliza seria.png como reserva, sin mezclar el estilo chibi con los bustos. ojos-cerrados.png es un séptimo recurso interno para el parpadeo.

No se necesita ninguna exportación manual para usar esta versión. Para diferenciar enojada en el futuro bastará un busto transparente con la misma escala y alineación. Las coordenadas y el tamaño del lienzo se documentan en ASSETS.md; avatar-contact-sheet.png muestra el conjunto sobre fondo oscuro.

## Integración posterior de Live2D

Conservar la API AliceAvatar({ estado, size, expression, label }). Sustituir su capa interna de imágenes por un renderer de canvas y un adaptador que traduzca esos estados a expresiones y movimientos del modelo; mantener el mismo contenedor, semántica accesible y fallback PNG. El chat continuará enviando estados de presentación sin importar el SDK.

Cargar el modelo de forma diferida y compartir recursos. Usar Live2D primero en size="hero" o en el espacio data-character-slot="welcome"; mantener los retratos PNG en mensajes para evitar un contexto WebGL por burbuja. Liberar recursos al desmontar, tratar pérdida de contexto y conservar el modo de movimiento reducido. La sincronía labial necesitará eventos de audio reales cuando exista TTS; no depende de que una respuesta esté pendiente.

No se incorpora un SDK Live2D ni se modifica el núcleo en esta entrega.
