# Actualización de rostros de Alice

Fuente: `docs/Hoja de expresiones de chica gótica anime.png`, PNG sRGB RGBA de **1536 × 1024**, 8 bits por canal. Los archivos nuevos están en `public/assets/avatar/`.

## Recortes y alineación

Los siete PNG de salida miden **220 × 264 (5:6)**. Cada busto conserva su resolución original: no hay escalado, reconstrucción, cambio de color ni generación. Se copian sus cuatro canales directamente a un canvas transparente. Los desplazamientos dejan margen alrededor del cabello y aproximan los ojos a la misma altura sin deformar cada pose.

Coordenadas medidas desde la esquina superior izquierda de la fuente. `dx` y `dy` indican dónde comienza el recorte dentro del PNG final.

| Archivo | x | y | ancho | alto | dx | dy |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| neutral.png | 903 | 5 | 198 | 208 | 11 | 15 |
| seria.png | 1107 | 5 | 195 | 208 | 12 | 9 |
| ojos-cerrados.png | 1305 | 5 | 207 | 208 | 6 | 20 |
| feliz.png | 919 | 218 | 195 | 191 | 12 | 30 |
| sorprendida.png | 1112 | 218 | 193 | 191 | 13 | 25 |
| triste.png | 1305 | 218 | 208 | 191 | 6 | 32 |

En `feliz.png`, el rectángulo de selección también abarca una esquina del puño de la ilustración grande vecina. Se excluyeron únicamente los píxeles de esa esquina con coordenadas fuente `x < 929` e `y > 360`. Se revisó el borde ampliado para preservar las hebras laterales del busto feliz. El resto del recorte conserva exactamente sus RGBA originales.

`ojos-cerrados.png` usa el busto superior derecho: sonrisa suave, párpados cerrados y mano junto al rostro. Es un recurso interno de parpadeo; la pose completa se conserva. `feliz.png` usa la sonrisa de boca abierta de la segunda fila. `triste.png` usa el busto de ojos cerrados y boca descendente de la segunda fila derecha.

**No hay un busto enojado independiente en esta hoja.** `enojada.png` es una copia byte por byte de `seria.png` como reserva de compatibilidad para la API pública. Esto no implica que el busto serio represente una expresión de ira diferenciada. La expresión molesta disponible pertenece al conjunto chibi y no se mezcló con los bustos.

## Transparencia y revisión

La fuente sí contiene alpha, aunque algunos visores muestran el beige de sus canales RGB ocultos. Contiene 525.725 píxeles con alpha 0 y 1.047.139 con alpha entre 1 y 254; no contiene alpha 255. La mayor parte del personaje está entre 251 y 253. Se preservó esta transparencia original sin aumentar la opacidad ni extraer un fondo nuevo.

`avatar-contact-sheet.png` muestra los siete archivos sobre `#20212b`, con sus nombres. Se revisaron cabello, márgenes, vecinos y expresiones. Los bustos se ven limpios sobre oscuro y no presentan el papel beige de la vista previa.

`avatar-source-verification.json` registra la comparación de cada salida con la fuente: **cero diferencias de canal RGBA** en todos los píxeles copiados y equivalencia exacta de `enojada.png` con `seria.png`.

## Iconos

`src-tauri/icons/` contiene:

- `32x32.png`: 32 × 32.
- `128x128.png`: 128 × 128.
- `128x128@2x.png`: 256 × 256.
- `icon.png`: 512 × 512.
- `icon.ico`: representaciones PNG de 16, 32, 48, 64, 128 y 256.
- `icon.icns`: representaciones PNG de 128, 256, 512 y 1024.

Los iconos se generan sobre un lienzo RGBA totalmente transparente, sin relleno negro, blanco, círculo, marco ni sombra. El rostro conserva el canal alfa original; el escalado interpola ese canal junto al color. Todos los tamaños PNG y los payloads PNG de ICO/ICNS conservan áreas con alpha 0. El maestro cuadrado ajusta el busto al lienzo sin cortar el cabello.


## Presentación sin fondo

Los avatares de la interfaz se muestran directamente, sin fondos, bordes, sombras, marcos de camafeo ni halo. Los indicadores de procesamiento también son transparentes. El tema de la aplicación permanece independiente de los PNG: las áreas vacías del personaje muestran la superficie que haya detrás.
