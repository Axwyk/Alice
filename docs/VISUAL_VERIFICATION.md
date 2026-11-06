# Verificación del rediseño visual

## Actualización de retratos — 6 de noviembre de 2026

- Fuente transparente aportada por el usuario: docs/Hoja de expresiones de chica gótica anime.png, 1536 × 1024.
- Siete PNG RGBA de 220 × 264: seis rutas públicas y un recurso interno de ojos cerrados. La reserva enojada reutiliza seria.
- Se conservaron exactamente los cuatro canales de los píxeles extraídos; comparación automática sin diferencias. Detalles en avatar-source-verification.json.
- TypeScript y Vite correctos; ejecutable Tauri release recompilado con dependencias locales y los iconos nuevos.
- Once grupos de pruebas de interfaz correctos, sin excepciones JavaScript, con IPC sustituido solo en Playwright.
- Comprobación adicional del canal alfa en canvas: neutral con 23.683 píxeles transparentes, 34.397 visibles y alpha máximo 254, igual que la fuente.
- Parpadeo confirmado con ojos-cerrados.png; movimiento reducido impide el parpadeo. Los siete recursos cargan y el fondo del camafeo es #252731.
- Revisiones visuales de bienvenida y chat, en escritorio y móvil. La lógica del chat, estilos generales y transporte permanecen idénticos a la revisión anterior.
- Se aplicaron 21 archivos de recursos/presentación/documentación y se verificaron sus hashes; este informe se actualizó después de compilar.
- No se repitieron pruebas Rust porque no hubo cambios en Rust ni en la lógica de conversación; las cinco pruebas anteriores siguen registradas debajo. No se abrió una sesión nativa de chat nueva.


## Verificación inicial del 5 de noviembre de 2026

Fecha: 5 de noviembre de 2026, America/Bogota.

| Verificación | Resultado |
| --- | --- |
| TypeScript y build Vite en Alice2 | Correctos |
| Tauri release con dependencias locales | Correcto; ejecutable src-tauri/target/release/alice2.exe |
| Pruebas Rust existentes | 5 correctas, 0 fallos |
| Pruebas de interfaz en Microsoft Edge sin ventana | 11 grupos correctos, 0 fallos y 0 errores JavaScript |
| Recursos | Seis expresiones y las tres variantes tipográficas cargan correctamente |
| Adaptación | Sin desbordamiento horizontal a 320, 390, 640 y 1160 px; revisión visual adicional a 1600 px |
| Archivos de implementación | 29 archivos verificados mediante SHA-256 después de copiarlos, más este informe |
| Fronteras preservadas | useChat.ts, types.ts, chatTransport.ts y main.tsx idénticos a la copia inicial |

Comandos de compilación:

    npm run tauri -- build --no-bundle -- --offline
    cargo test --offline --manifest-path src-tauri/Cargo.toml

No se generó un instalador. El ejecutable de producción quedó compilado en el proyecto.

Los grupos funcionales comprueban saludo y recursos; bloqueo de vacío, Shift+Enter e IME; Enter y procesamiento; error tipado y recuperación; conservación del borrador nuevo durante un fallo; confirmación, cancelación, Escape y foco del reinicio; límite de 2000/2001 caracteres Unicode; movimiento reducido; diseño adaptable y título accesible; fallback del retrato; ausencia de excepciones.

Las pruebas de interfaz inyectan un sustituto IPC únicamente desde Playwright. El producto conserva su transporte Tauri real y no contiene un mock de navegador. Estas comprobaciones acreditan el comportamiento de React; no sustituyen una prueba interactiva de la ventana nativa. En esta entrega se verificaron la compilación nativa y los tests reales del servicio Rust, sin iniciar una nueva sesión de chat nativa.

Contraste calculado con luminancia relativa: texto principal/fondo 13,76:1; texto secundario/superficie 7,40:1; texto tenue/superficie elevada 5,56:1; azul/superficie 6,93:1; error/fondo 7,99:1; borde del compositor/superficie 3,16:1; foco/superficie 9,93:1. Los bordes decorativos no se usan como única indicación de estado. Se revisaron foco de teclado, orden de lectura y movimiento reducido; esto no es una certificación completa de accesibilidad.

No se modificaron los archivos ni datos del proyecto histórico Alice 1.0. Los cambios de código y las bases de datos que ya existían allí se conservan.

## Corrección del icono nativo — 6 de noviembre de 2026

La recompilación anterior conservó un recurso de icono antiguo porque el script de compilación no observaba los cambios en icons/. Se añadió cargo:rerun-if-changed=icons en src-tauri/build.rs y se recompiló el ejecutable release. No cambió la lógica de conversación.

Se verificaron directamente los recursos RT_GROUP_ICON/RT_ICON del archivo PE, sin usar la caché de iconos de Windows. Las seis representaciones incrustadas (16, 32, 48, 64, 128 y 256 px) coinciden exactamente con los bytes PNG del ICO actual. Evidencia en native-icon-verification.json.

El acceso directo existente Alice.lnk del escritorio conserva su destino y ahora asigna explícitamente src-tauri/icons/icon.ico. Se notificó a Windows el cambio de ese archivo y de ese acceso directo, sin reiniciar el Explorador ni vaciar la caché global.
## Icono e interfaz sin fondo — 6 de noviembre de 2026

Por solicitud del usuario, se retiraron los rellenos añadidos tanto en los iconos como en los contenedores del personaje. Los avatares originales ya tenían transparencia y no se modificaron.

Los iconos PNG/ICO/ICNS se regeneraron sobre RGBA(0,0,0,0), sin flatten, discos, marcos, sombras ni fondos opacos. Las cuatro esquinas de cada PNG y de cada tamaño empaquetado tienen alpha 0; native-icon-transparency.json contiene las estadísticas. Los seis recursos PNG incrustados en el ejecutable coinciden exactamente con el ICO transparente, según native-icon-verification.json.

En navegador se comprobaron los fondos CSS de avatar, retrato e indicador de procesamiento: todos rgba(0,0,0,0), bordes de 0 px y box-shadow none. La bienvenida no contiene marco ni halo. Se revisaron bienvenida, móvil, procesamiento y mensajes, con siete recursos cargados y sin excepciones. ui-avatar-transparency.json contiene los estilos calculados. El IPC solo se sustituyó durante la comprobación con Playwright.

TypeScript, Vite y la compilación Tauri release finalizaron correctamente. Se mantuvo la lógica del chat. El acceso directo del escritorio sigue apuntando al ICO y se notificó a Windows únicamente la actualización del archivo y del acceso directo.