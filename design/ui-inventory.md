# Inventario inicial de interfaz

**Versión de identidad:** 0.2, propuesta visual; implementación autorizada · **Alcance:** selección global dentro de la aplicación · **Muestras:** FigCraft `pswitcher.figcraft`, páginas 02 Catálogo UI, 03 Aplicación y 04 Flujos.  
Inventario de selección global, versiones instaladas, instalación y directorio configurable. La selección por proyecto queda fuera del primer alcance visual. No representa controles ya implementados.

Los frames de la página 04: Gestión global 1280 × 820; Cambiar versión global 1280 × 820; Instalar + elegir carpeta 1280 × 820; Vacío/carga/error 1440 × 820; Preferencias Bash desactivada, Confirmación Bash y Bash activa/revertir, cada uno 1280 × 820.

| ID | Pieza | Finalidad y uso | Variantes / estados | Muestra |
|---|---|---|---|---|
| NAV-01 | Navegación lateral | Cambiar entre gestión de versiones y preferencias globales. | Activa, normal, foco teclado; texto visible; en ventana estrecha puede colapsar con nombre accesible. | Catálogo UI: navegación; Aplicación y Flujos: barra lateral |
| SCOPE-01 | Indicador de ámbito global | Aclarar que las selecciones afectan al ámbito global de la aplicación. Es etiqueta informativa, no selector en este alcance. | Etiqueta global visible; sin acción ni alternativa de proyecto. | Aplicación: encabezado global; Flujos: gestión global y cambio de versión |
| ACT-01 | Botón primario | Iniciar instalación o guardar una selección global. | Normal, hover, foco, pulsado, deshabilitado y carga; solo una acción primaria por contexto. | Catálogo UI: acciones; Flujos: instalar y guardar |
| ACT-02 | Botón secundario | Cambiar versión, abrir selector o cancelar sin quitar peso a la acción principal. | Normal, hover, foco, pulsado, deshabilitado. | Catálogo UI: acciones |
| TOOL-01 | Fila de herramienta | Identificar PHP, Node.js o Composer, versión global seleccionada y acción para cambiarla. | Activa/inactiva; con varias versiones, vacía, carga y error. Nombre y estado textuales; versión en monoespaciada. | Catálogo UI: herramienta; Aplicación y Flujos: selección global |
| FIELD-01 | Campo de versión | Elegir una versión instalada para una herramienta en ámbito global. | Cerrado, abierto, foco, deshabilitado; vacía con ayuda; error con mensaje asociado. Mantener teclado y selección legible. | Catálogo UI: campo; Flujos: cambiar versión |
| PATH-01 | Campo de ruta | Mostrar y editar directorio base configurable; `~/Tools` solo ejemplo. | Lectura/edición, foco, inválida, guardando/guardada. Etiqueta persistente y error junto al campo; selector nativo de carpeta complementario. | Catálogo UI: ruta; Aplicación y Flujos: directorio |
| STATUS-01 | Etiqueta de estado | Comunicar activa, instalada, actualización o error junto a texto. | Éxito, aviso, error, neutral; icono opcional de familia única, nunca color solo. | Catálogo UI: estados |
| PANEL-01 | Panel de contenido | Agrupar selección global o preferencias relacionadas. | Superficie blanca, borde 1 px, padding 24 px; contenido flexible; sin comportamiento de clic implícito. | Catálogo UI: panel; Aplicación y Flujos: selección global y preferencias |
| INSTALL-01 | Formulario de instalación | Elegir herramienta, versión disponible y directorio base antes de instalar. | Herramienta PHP, Node.js o Composer; versión seleccionable; normal, foco, carga, éxito, error. Evitar doble envío mientras instala. | Flujos: instalación |
| FOLDER-01 | Diálogo de directorio | Revisar o cambiar la carpeta base; “Examinar” abre el selector nativo de directorios Linux. | Ruta válida, edición, cancelación; validar entrada y permisos antes de aceptar. Prototipo muestra ruta, no ejecuta selector del sistema. | Flujos: elegir carpeta |
| DIALOG-01 | Diálogo de selección | Elegir y guardar una versión para una herramienta en ámbito global. | Normal, foco, versión seleccionada; foco inicial en búsqueda; Escape cierra, devuelve foco al origen. | Flujos: cambiar versión global |
| STATE-01 | Presentación de estados | Explicar lista vacía, instalación en curso y error con acción de salida. | Vacío con CTA; carga con progreso indeterminado y cancelar; error con detalle y reintento. Texto e iconografía si se incorpora, nunca color solo. | Flujos: vacío, carga y error |
| BASH-01 | Preferencia de integración Bash | Conectar versiones globales con `php`, `node` y `composer` en terminal Bash solo por activación explícita. | Desactivada por defecto; confirmación previa; activa; desactivar y retirar solo el bloque gestionado. Explicar que se añadirá a `~/.bashrc`, preservando el resto. | Flujos: activar, confirmar y desactivar |

## Reglas compartidas

- Roles y valores: [visual-identity.md](visual-identity.md) y [design-tokens.json](design-tokens.json); no duplicar valores en implementaciones.
- Texto y estados: `text.body` para lectura, `text.label` para etiquetas breves, `text.technical` para versiones y rutas; acción primaria Red Violet, foco Red Violet profundo, mínimo de control 40 px. Isabella Color base solo decorativo; usar `color.accent.isabellaText` sobre `color.accent.isabellaBackground` para texto.
- Teclado: orden de tabulación coherente con el contenido, foco siempre visible, etiquetas visibles y mensajes de error asociados. Escape cierra los diálogos y devuelve foco a la acción que los abrió.
- Adaptación: escritorio Linux; por debajo de 1024 unidades lógicas, navegación plegable etiquetada y paneles apilados. No se diseña una vista móvil.
- Movimiento: 140/200 ms según guía; alternativa inmediata con reducción de movimiento.
- FigCraft: `ACT-01` es componente primario con instancias en instalación y estado vacío; código aún no implementado.
- Terminal: integración Bash **optativa**, apagada por defecto. La activación añade un bloque gestionado a `~/.bashrc`; la desactivación retira solo ese bloque. No se abre ni modifica ningún archivo del equipo durante esta maqueta.
- Revisión: identidad y flujos documentados como propuesta; implementación autorizada por instrucción del usuario; FigCraft especifica estados y movimiento, no los reproduce como interacción ejecutable.
