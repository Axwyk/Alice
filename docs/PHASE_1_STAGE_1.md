# Fase 1 — Etapa 1: contratos y fundamentos de alice-core

Fecha: **6 de noviembre de 2026**, America/Bogota.

Estado: **Etapa 1 completada**. La autorización utilizada se limita a contratos y reglas del Core. No se inició la Etapa 2 y no se da por completada la Fase 1.

## 1. Resumen

Se creó `crates/alice-core` como biblioteca Rust independiente. Compila y se prueba sin Tauri, Tokio, tokio-util, executor, HTTP, base de datos ni SDK de proveedores. La única dependencia resuelta es `futures-core 0.3.34`, sin características predeterminadas, utilizada únicamente por el trait `Stream`.

La biblioteca contiene valores, validaciones y puertos. No ejecuta tareas, hilos, streaming ni solicitudes de cancelación. No proporciona un servicio de conversación, coordinador, proveedor simulado, proveedor real ni repositorio. Las pruebas de cancelación usan exclusivamente un doble de señal de `std`, expresamente permitido en esta etapa.

El chat de escritorio y su servicio de demostración permanecen intactos. El nuevo crate todavía no es una dependencia del backend Tauri.

## 2. Inspección y preservación previas

Antes de escribir se inspeccionaron estructura y Git y se leyeron completos `README.md`, `docs/ARCHITECTURE.md`, `docs/CHAT_INTEGRATION.md` y `docs/PHASE_1_TECHNICAL_PLAN.md`, incluidas las precisiones de terminal independiente, límites finitos y retención. No existe `docs/PHASE_1_PLAN.md`; se utilizó el plan vigente indicado.

Git confirmó un repositorio independiente Alice2 en la rama `codex/phase-0-chat`. Los archivos existentes aparecían sin seguimiento, incluidos cambios visuales y recursos añadidos después del Paso C. No se reconstruyó el proyecto, no se restablecieron esos archivos ni se realizaron commits.

Se tomaron huellas SHA-256 de los 74 archivos existentes de código, documentación, manifiestos y recursos de Alice2, excluyendo dependencias, builds y metadatos Git. También se tomaron las de los 1607 archivos de Alice 1.0.

## 3. Archivos creados y modificados

```text
crates/alice-core/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── src/
│   ├── lib.rs
│   ├── message.rs
│   ├── conversation.rs
│   ├── operation.rs
│   ├── provider.rs
│   ├── storage.rs
│   └── error.rs
└── tests/
    ├── identifiers_messages.rs
    ├── conversations.rs
    ├── operations.rs
    ├── provider_contracts.rs
    ├── cancellation.rs
    └── support/mod.rs
```

Además se creó este reporte y se añadió una entrada al `CHANGELOG.md`, conservando sus entradas previas. Estos son los únicos cambios fuera del crate y su necesidad se explicó antes de realizarlos.

No se modificaron el plan, la arquitectura, README, React/TypeScript, assets, configuración, manifiestos o lockfiles del escritorio ni el servicio de demostración. No hubo una nueva decisión arquitectónica que exigiera modificar `ARCHITECTURE.md`; los detalles concretados aquí desarrollan sus fronteras existentes. El texto del plan permanece como referencia y no se transforma en autorización de etapas posteriores.

No se creó un workspace Cargo en la raíz ni se añadió una dependencia al manifiesto Tauri. Por eso los comandos seleccionan explícitamente el manifiesto del crate independiente. `target/` se excluye mediante el `.gitignore` local del crate; `Cargo.lock` se conserva para reproducibilidad.

## 4. Contratos y decisiones implementadas

### Identificadores y mensajes

- `ConversationId`, `MessageId` y `OperationId` son tipos distintos con contenido privado, constructor validado, `FromStr`, igualdad, orden y hash.
- Los IDs aceptan entre 1 y 128 bytes ASCII alfanuméricos, guion o guion bajo. Rechazan espacios, controles, rutas y cadenas vacías. Aceptan UUID textuales, pero no obligan a ese formato.
- Core no genera IDs ni garantiza unicidad global: esas responsabilidades corresponden a la aplicación futura. Se evitó `uuid` porque esta etapa solo necesita validar y transportar identificadores opacos; no requiere aleatoriedad ni generación.
- `Message` tiene ID, `Role::{System, User, Assistant}`, texto y marca temporal opcional `SystemTime`.
- `MessageLimits` exige límites positivos explícitos de caracteres Unicode escalares y bytes UTF-8. No hay constructor ilimitado ni valores de producción predeterminados.
- El texto no vacío se conserva exactamente, incluidos espacios y saltos. Un texto compuesto solo por espacios Unicode se rechaza. No se añade serialización ni formato de proveedor al dominio.

### Conversaciones

- `Conversation` es un agregado local de mensajes, no un servicio. Conserva ID, revisión, límites y mensajes con acceso de lectura.
- `append` exige la revisión esperada, impide IDs duplicados y aplica límites de cantidad de mensajes y bytes de contenido. Los errores no alteran el agregado y las revisiones nunca desbordan silenciosamente.
- Los mensajes de sistema solo preceden al diálogo; una respuesta de asistente requiere un usuario inmediatamente anterior. Se permiten entradas de usuario consecutivas; la política completa de turnos fallidos/reintentos sigue pendiente de la capa de aplicación.
- `restore` valida el snapshot, sus mensajes y una revisión que no puede ser inferior a su cantidad de mensajes. No realiza I/O.
- Los límites son parámetros obligatorios. Las cifras pequeñas usadas por las pruebas son fixtures, no presupuestos de producción ya aprobados. Globales, retención y reservas se implementarán únicamente en su etapa autorizada.

### Operaciones y eventos

- `Correlation` agrupa conversación, operación y mensaje de entrada. `OperationContext` contiene ID, deadline `Instant` y señal neutral. El tiempo se entrega explícitamente a las evaluaciones; Core no crea timers.
- `Operation` es un valor con máquina de estados, última secuencia y evento terminal opcional, sin registro global, ejecución o coordinación concurrente.
- `apply_event` valida correlación completa, secuencia exacta desde 1 y transiciones permitidas antes de cambiar el valor. Un evento inválido no produce cambios parciales.
- Se contemplan `Validating`, `Rejected`, `Accepted`, `Running`, `CancelRequested`, `Completed`, `Failed` y `Cancelled`. `Rejected` es anterior a admisión y no genera un terminal de operación aceptada.
- Se admiten cierre por error desde admisión y cancelación antes o después de empezar. Aceptada la transición a `CancelRequested`, una respuesta tardía no puede confirmarse como completada. Un terminal no se reescribe.
- `OperationEvent` valida secuencia no nula y tamaño de contenido con presupuesto explícito. `ProviderEvent` conserva correlación y payload neutral; no asigna la secuencia autoritativa de aplicación.
- El valor terminal no necesita un repositorio. La autoridad global, liberación de reservas, almacenamiento independiente de resultados, timeout efectivo, supervisión y retención siguen siendo responsabilidades futuras del coordinador; esta etapa no afirma haberlas implementado.

### Proveedores y capacidades

- `ModelRef` separa instancia y modelo sin enumerar marcas. El ID de modelo es opaco, limitado a 256 bytes y sin espacios o controles; admite separadores habituales como `/` y `:`.
- `AiRequest` incluye correlación, modelo, contexto validado y acotado, opciones y capacidades requeridas. El último mensaje debe ser el usuario identificado por la correlación.
- `GenerationOptions` requiere un máximo de salida positivo; no introduce parámetros específicos de un proveedor.
- `AiResponse` contiene mensaje de asistente distinto del de entrada, modelo seleccionado, motivo de finalización conocido y uso opcional. `validate_for` verifica correspondencia con solicitud y que el ID de respuesta no duplique mensajes del contexto.
- `Usage` diferencia desconocido (`None`) de cero. `FinishReason` diferencia final normal, límite de salida y rechazo; un fin de protocolo desconocido no puede representarse directamente como éxito.
- `AiProvider` es `Send + Sync`, compatible con `dyn`, con declaración local, consulta de capacidades, respuesta completa y contrato de stream. No hay ninguna implementación de `AiProvider`, tampoco simulada en pruebas.
- `BoxFuture` es un alias local de `Pin<Box<dyn Future<Output = T> + Send + 'a>>` con `std`. `ProviderStream` es propietario y usa solo el trait `futures_core::Stream`.
- `Support` representa `Supported`, `Unsupported` y `Unknown`. `CapabilityEvidence` conserva procedencia, alcance, observación y vigencia opcionales. Evidencia vencida o todavía no vigente se evalúa como desconocida.
- `AdapterCapabilities` y `ProviderCapabilities` distinguen declaración del adaptador y datos de un modelo/instancia concretos; `ModelCapabilities` es un alias del segundo. No se permite adjudicar evidencia de otro modelo.
- `TokenLimit` conserva evidencia propia para límites numéricos: la expiración devuelve desconocido, no cero. La comprobación `require` exige soporte tanto del adaptador como del modelo para las capacidades requeridas.
- No se implementaron descubrimiento, caché, selección, registro, inferencia ni streaming. Validar opciones contra todos los límites efectivos de un modelo y supervisar que el adaptador respete sus promesas corresponde a la siguiente capa autorizada.

### Cancelación

`CancellationSignal` ofrece `is_requested()` y `requested() -> BoxFuture<'_, ()>`. Es una señal de solicitud de solo lectura, monotónica, de un solo uso, sin reinicio; su espera debe resolver inmediatamente si ya se solicitó y no perder notificaciones.

La única implementación concreta de señal está en `tests/support/mod.rs`. Usa un mutex de `std` compartido por consulta, registro y solicitud, con esperas acotadas, actualización del waker y limpieza al descartar un futuro. No crea hilos, tareas ni executor. Se cubren los dos órdenes relevantes de solicitud y espera mediante polling controlado, sin sleeps.

Las pruebas validan ese doble y la semántica del contrato. No demuestran todavía un adaptador Tokio, interrupción de HTTP o aborto remoto. `CancelRequested` no es un terminal; `RemoteCancellation` distingue `NotApplicable`, `Unconfirmed` y `Confirmed`. El Core no inventa una confirmación remota a partir de la señal.

### Errores y almacenamiento

- Familias separadas: validación, operación, proveedor y almacenamiento. `CoreError` permite transportarlas como un resultado de dominio.
- Los códigos estables tienen prefijos `validation.`, `operation.`, `provider.` y `storage.`. Display y mensajes de UI son textos fijos, sin causas HTTP, claves, endpoints ni contenidos de entrada interpolados. `ProviderError` lleva ID de operación para correlación.
- `ConversationRepository` define `load` y `save`; `ExpectedRevision::{Absent, Exact}` expresa creación o actualización condicionada. La escritura atómica y rechazo de revisiones conflictivas son obligaciones del implementador futuro.
- El puerto no guarda los terminales de operación, no ofrece transacciones genéricas ni implementa almacenamiento en memoria o disco.

## 5. Dependencias e independencia verificadas

Herramientas observadas: `rustc 1.99.1`, Cargo `1.99.1`, toolchain `stable-x86_64-pc-windows-msvc`. El crate declara Rust 1.90 y edición 2024; el mínimo declarado no se probó con una instalación separada de Rust 1.90.

```text
alice-core v0.1.0
└── futures-core v0.3.34
```

`cargo tree --edges normal,build,dev` mostró únicamente ese grafo. `cargo metadata --offline --locked` confirmó un único miembro independiente y dos paquetes resueltos; la dependencia directa tiene `default-features = false` y ninguna feature adicional. `futures-core` declara una dependencia opcional `portable-atomic` en su manifiesto publicado, pero no está activada ni forma parte del grafo resuelto de este crate.

No hay dependencias de build o desarrollo en `alice-core`. No existen dependencias directas ni indirectas resueltas de Tauri, Tokio, tokio-util, Reqwest, SQLx, SQLite, SDK comerciales, async-trait, futures-util o executors. No se instaló ni actualizó una herramienta global. La compilación utilizó la copia ya disponible de `futures-core` en caché; los comandos de Cargo se ejecutaron offline.

## 6. Comprobaciones y resultados reales

Desde la raíz de Alice2:

```powershell
cargo fmt --manifest-path crates/alice-core/Cargo.toml -- --check
cargo test -p alice-core --manifest-path crates/alice-core/Cargo.toml --offline --locked
cargo check -p alice-core --manifest-path crates/alice-core/Cargo.toml --offline --locked
cargo tree -p alice-core --manifest-path crates/alice-core/Cargo.toml --edges normal,build,dev --offline --locked
cargo metadata --manifest-path crates/alice-core/Cargo.toml --format-version 1 --offline --locked
```

En la sesión se invocaron mediante Rustup con la toolchain MSVC verificada, sin editar PATH. El primer test generó el lockfile offline; la verificación final utilizó `--locked`.

| Comprobación | Resultado |
|---|---|
| Formato | Código de salida 0 |
| Pruebas | **42 aprobadas, 0 fallos**; código de salida 0 |
| `cargo check` | Código de salida 0 |
| Árbol normal/build/dev | Solo Core y futures-core; código de salida 0 |
| Metadatos Cargo | Resolución independiente y sin runtime |
| Artefactos ignorados por Git | `crates/alice-core/target/debug` excluido por el `.gitignore` local |

Distribución: 1 prueba interna de desbordamiento de secuencia; 8 de cancelación/futuros/contexto; 6 de conversaciones; 7 de identificadores/mensajes/errores; 10 de operaciones/eventos; 10 de contratos de proveedor/capacidades. Las pruebas externas ejercitan unidades de dominio por la API pública. Los tests solo son Rust estándar y no requieren red, servicios ni bases de datos.

La primera ejecución pasó 39 pruebas. Durante la revisión se añadieron evidencia/vigencia de límites numéricos y cobertura de rutas terminales y desbordamiento; por esos cambios se repitieron las verificaciones finales y pasaron 42. No hubo fallos ocultos de compilación o pruebas ni comprobaciones obligatorias que quedaran sin ejecutar.

## 7. Preservación de Alice 1.0 y escritorio

La comparación SHA-256 de Alice 1.0 dio **1607 archivos antes y después, 0 modificados, 0 añadidos y 0 eliminados**. Solo se leyó para obtener huellas. No se ejecutaron pruebas históricas ni se abrieron sus bases mediante un motor de datos.

Las huellas de todos los archivos preexistentes de Alice2 se conservaron salvo el cambio documental anunciado de `CHANGELOG.md`. El chat, sus cambios visuales, avatares, fuentes, iconos, IPC, demostración y manifiestos del escritorio se preservaron.

No se recompiló ni reabrió el escritorio en esta etapa: no se modificó su código ni se enlazó con Core, y la autorización pidió evitar verificaciones costosas innecesarias. La evidencia aquí es de conservación de implementación, no una nueva prueba de ejecución o producción del escritorio. Su build de producción completo se verificará cuando se autorice una etapa que lo afecte.

## 8. Límites y decisiones pendientes para Etapa 2

- El crate no admite solicitudes de chat ni administra conversaciones activas: ofrece valores y reglas para que el servicio posterior lo haga.
- No hay coordinador, reservas, buffers de ejecución, controlador de cancelación, retención, timeouts efectivos ni recuperación del proceso. Los eventos y el estado local no son una implementación de esas funciones.
- Ninguna prueba implementa proveedores simulados, repositorios ni servicios de etapas posteriores. La suite de dos proveedores intercambiables sigue pendiente para su etapa autorizada.
- Los límites de valores son explícitos y positivos, sin defaults ilimitados. Los presupuestos globales y provisionales de la aplicación deben definirse antes de habilitar sus recursos; este crate no los inventa como cifras de producción.
- `CancellationSignal` es un contrato: cada implementación futura debe demostrar las mismas garantías de notificación. La confirmación de aborto remoto requiere evidencia ajena a Core.
- La conversación inicial usa mensajes ordenados, no la orquestación completa de turnos y reintentos. Los mensajes fallidos, el contexto efectivo y la selección de personalidad/memoria no están implementados.
- Antes de Etapa 2 deben concretarse creación/unicidad de IDs, ensamblaje de turnos, políticas de contexto, validación efectiva de límites de modelo y las responsabilidades del servicio frente a los valores existentes.
- La autoridad terminal independiente del historial y las políticas de retención del plan siguen vigentes; se implementarán en sus componentes de aplicación, sin trasladar esas responsabilidades al repositorio.
- Windows MSVC es el entorno de compilación verificado. No se afirma haber probado Linux/macOS, otra versión de Rust, modelos, streaming remoto o persistencia.

## 9. Cierre

**Se implementó únicamente la Etapa 1.** Se conserva el escritorio existente, no se integraron proveedores ni se inició la Etapa 2. Este reporte no autoriza más trabajo. Se espera una nueva autorización explícita del usuario.
