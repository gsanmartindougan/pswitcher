# Especificación de interfaz

**Revisión:** interfaz 0.2 · **Estado:** propuesta de interfaz; implementación autorizada por instrucción del usuario · **Fuente:** [pswitcher.figcraft](pswitcher.figcraft), páginas 03 Aplicación y 04 Flujos.  
**Destino:** escritorio Linux, implementación con Rust/Iced. **Idioma de las muestras:** español, supuesto reversible. Identidad y flujos cubiertos por la autorización de implementación del usuario.

La ventana implementada tiene **430 × 610 px**, fija, como el proyecto Qui; navegación y controles se distribuyen en una columna compacta.

## Alcance

Elegir y conservar una versión global en la aplicación para PHP, Node.js y Composer; instalar versiones; configurar el directorio base; integrar de forma opcional las versiones seleccionadas con Bash. No se diseña todavía selección por proyecto. Los nombres de versión visibles son contenido de ejemplo, no un catálogo consultado.

## Frames y recorridos

| Frame | Tamaño | Recorrido |
|---|---:|---|
| 03 Aplicación · Gestión global | 1280 × 820 | Resumen de las tres versiones activas, acción de instalación y directorio actual. |
| 04 Flujos · Gestión global | 1280 × 820 | Ajustar selección de PHP, Node.js y Composer; guardar la selección global. |
| 04 Flujos · Cambiar versión global | 1280 × 820 | Abrir selector desde una herramienta, filtrar versiones instaladas, elegir una y guardar. Escape cierra y devuelve el foco a la acción de origen. |
| 04 Flujos · Instalar + elegir carpeta | 1280 × 820 | Elegir herramienta y versión; comprobar ruta base; examinar carpeta; cancelar o usar ruta y confirmar instalación. |
| 04 Flujos · Vacío / carga / error | 1440 × 820 | Sin versiones: CTA de instalación. En curso: estado legible y cancelar. Error: detalles y reintento. Muestras estáticas, no operación real. |
| 04 Flujos · Preferencias Bash desactivada | 1280 × 820 | Mostrar estado optativo apagado y acción para empezar la activación. |
| 04 Flujos · Confirmación Bash | 1280 × 820 | Explicar el cambio concreto a `~/.bashrc`; cancelar o confirmar expresamente. |
| 04 Flujos · Bash activa / revertir | 1280 × 820 | Mostrar los comandos globales y permitir desactivar, retirando solo el bloque gestionado. |

### Reglas de interacción

- Cada fila de herramienta muestra nombre, versión seleccionada globalmente y acción “Cambiar”. El diálogo identifica ámbito y herramienta antes de guardar la selección.
- La instalación separa herramienta, versión disponible y carpeta base. La ruta se puede editar; “Examinar” abre el selector nativo de directorios Linux. `~/Tools` es ejemplo, no valor obligatorio.
- Con cero versiones, presentar instalación como siguiente paso. Durante instalación, anunciar herramienta/versión y ofrecer cancelar. Ante error, conservar la selección, mostrar texto y permitir ver detalle o reintentar. Los mensajes exactos y los límites de cancelación dependen del instalador.
- La selección global queda en la aplicación. Por sí sola no modifica terminales. La integración Bash está desactivada por defecto.
- Al activar Bash, mostrar confirmación previa: añade un bloque delimitado propio a `~/.bashrc` y conecta `php`, `node` y `composer` con las versiones globales. Conserva el resto del archivo. Solo terminales Bash nuevas cargan el cambio.
- “Desactivar y revertir” retira solo el bloque delimitado por la aplicación; conserva líneas ajenas. Terminales ya abiertas no cambian. Si no se puede modificar o retirar el bloque sin dañar contenido ajeno, no sobrescribir; mostrar error y preservar el archivo. La recuperación ante ediciones externas queda por definir en implementación.
- No se abre ni modifica ningún archivo real del equipo en esta fase. El selector nativo, activación, reversión y estados de error son anotaciones de diseño.

## Adaptación y teclado

Frames de referencia 1280 × 820 para la ventana de aplicación y 1440 × 820 para el tablero de estados. Escritorio Linux solamente; no se fuerza un diseño móvil. Como regla propuesta, desde 1024 unidades lógicas mantener navegación lateral de 224 px y dejar crecer el contenido; por debajo de 1024, convertir la navegación en menú plegable con etiquetas y apilar paneles y campos. Evitar desbordamiento horizontal, truncar rutas solo con acceso a la ruta completa y admitir aumento de escala del escritorio.

Todos los controles se operan por teclado. Mostrar foco con token `color.border.focus`. Al abrir un diálogo, poner foco en su primer campo o título cuando proceda; Escape cancela si no hay operación en curso y devuelve foco al control de origen. No depender de hover. Controles con altura mínima de 40 px. Etiquetas de estado usan texto además de color.

## Movimiento

| Elemento | Disparador | Cambio | Duración / curva | Interrupción | Movimiento reducido |
|---|---|---|---|---|---|
| Diálogo | Abrir o cerrar | Opacidad 0→1 y 1→0; sin desplazamiento | 140 ms, `motion.curve.standard` | Invertir al cerrar/abrir de nuevo; no bloquear foco | Aparición inmediata |
| Selección de versión | Elegir opción | Actualizar etiqueta de seleccionada y resumen | 200 ms, `motion.curve.standard` | Estado nuevo reemplaza al anterior | Actualización inmediata |
| Instalación | Inicio y respuesta | El tablero presenta vacío, carga o error como frames estáticos | Sin animación reproducida en esta maqueta | Cancelación visible como acción independiente | Mantener texto de estado; indicador quieto |

FigCraft documenta estados y capas; no reproduce estas transiciones. No se afirma interacción ejecutable.

## Pendientes

- Verificar visualmente la aplicación en un escritorio Linux con compositor.
- Comprobar descargas e instalaciones contra los catálogos reales; PHP además requiere dependencias de compilación instaladas.
- Confirmar nombre del producto e idioma final; las muestras actuales usan español.
