# Identidad visual — gestor de versiones

**Versión:** 0.2 · **Estado:** propuesta visual; implementación autorizada por instrucción del usuario  
**Plataforma:** escritorio Linux · **Stack:** Rust + Iced  
**Idioma de las muestras:** español (supuesto reversible) · **Tema:** claro

## Dirección

Un espacio de trabajo técnico y directo, con identidad editorial cálida. Red Violet concentra la acción y la navegación activa; Isabella Color aporta acento material en muestras y detalles decorativos. Superficies marfil mantienen densidad de escritorio sin convertir cada grupo en una tarjeta. La tipografía monoespaciada identifica versiones y rutas, no párrafos. Sin logotipo ni imágenes: el nombre del producto sigue pendiente. El sistema elimina teal y azul.

## Color y contraste

| Rol | Valor | Uso |
|---|---|---|
| Lienzo | `#F7F5F0` | Fondo principal, marfil cálido |
| Superficie | `#FFFFFF` | Paneles, campos y barra superior |
| Superficie secundaria | `#F0EADD` | Navegación y selección secundaria |
| Texto principal | `#28231F` | Títulos y contenido |
| Texto secundario | `#5C564D` | Ayuda, metadatos y etiquetas |
| Borde | `#D8D0C3` | Separación no interactiva |
| Red Violet base | `#C71585` | Color de identidad y acción principal |
| Acción hover | `#A1146E` | Acción bajo puntero |
| Acción pulsada | `#850E59` | Acción activada |
| Foco | `#76104F` | Anillo de teclado derivado de Red Violet |
| Isabella Color base | `#C3A55C` | Acento decorativo y muestra de identidad; no texto ni estado crítico |
| Oro oscuro / superficie | `#6D5725` / `#F5EFDE` | Texto legible y panel de acento |
| Tinte Red Violet | `#F9EAF2` | Fondo tenue de carga o selección secundaria |
| Éxito | `#336344` / fondo `#E8F1E9` | Estado acompañado por etiqueta |
| Aviso | `#6F551A` / fondo `#F5EEDB` | Advertencia acompañada por texto |
| Error | `#9D2E28` / fondo `#F8E8E6` | Error acompañado por texto |

**Bases pedidas:** Red Violet `#C71585`, referencia [Figma Colors](https://www.figma.com/colors/red-violet/); Isabella Color `#C3A55C`, interpretación del nombre del [Dictionary of Color Combinations de Sanzo Wada](https://sanzo-wada.dmbk.io/). Tonos de acción se derivan de la escala de Red Violet; oro oscuro y superficies marfil se derivan para mantener contraste y calidez.

Contraste calculado con luminancia relativa sRGB: texto principal/superficie **15,55:1**; secundario/superficie **7,26:1**; principal/lienzo **14,27:1**; secundario/lienzo **6,66:1**; secundario/superficie secundaria **6,05:1**; texto blanco/acción Red Violet **5,42:1**; texto blanco/acción hover **7,39:1**; texto blanco/acción pulsada **9,53:1**; foco/superficie **10,73:1**; foco/lienzo **9,85:1**; oro oscuro/superficie oro **6,01:1**; éxito/fondo **6,05:1**; aviso/fondo **6,06:1**; error/fondo **6,19:1**. Isabella Color base sobre blanco ofrece **2,37:1**: no se usa como texto, borde funcional ni indicador; queda reservado a decoración. Los pares de texto medidos superan 4,5:1. El borde es estructural, no texto ni indicador único. Esto no sustituye auditoría completa de accesibilidad ni de estados implementados.

## Tipografía

| Rol | Familia | Peso | Tamaño / interlineado | Uso |
|---|---|---:|---:|---|
| Interfaz | Noto Sans | 400 | 14 / 20 px | Cuerpo, listas y ayuda |
| Interfaz fuerte | Noto Sans | 600 | 14 / 20 px | Botones, navegación y encabezados de panel |
| Subtítulo | Noto Sans | 600 | 20 / 28 px | Encabezado de vista |
| Título | Noto Sans | 700 | 28 / 36 px | Título principal |
| Etiqueta | Noto Sans | 600 | 12 / 16 px | Etiquetas breves, nunca texto extenso |
| Dato técnico | Fira Code | 400 | 13 / 20 px | Versiones, comandos y rutas |

Noto Sans y Fira Code están instaladas en el entorno de diseño; FigCraft no reportó fuentes ausentes. Se usan como fuentes del sistema, sin empaquetarlas. Confirmar disponibilidad y glifos del conjunto de idiomas objetivo antes de distribución. El consumidor debe resolver familias del sistema y mantener legible el tamaño si el usuario cambia la escala de escritorio.

## Escala y composición

- Espaciado en unidades lógicas de diseño: **4, 8, 12, 16, 24, 32, 48**. Usar 8–12 px dentro de controles, 16–24 px entre grupos y 32 px entre secciones.
- Radios: **6 px** en campos y botones; **10 px** en paneles; **14 px** solo para contenedores grandes. Evitar pastillas decorativas.
- Borde: **1 px** neutro; **2 px** para foco. Sin sombra por defecto; separar por fondo y borde.
- Escritorio: navegación lateral de referencia **224 px**, barra superior **56 px**, margen de contenido **32 px**, ancho de lectura **1120 px**. Son guías para la maqueta, no tamaños físicos de pantalla.
- Controles: altura visual mínima **40 px**; objetivo interactivo mínimo **40 × 40 px** en la propuesta. En la implementación, respetar escala DPI, teclado y preferencias del escritorio.
- Densidad moderada: una fila por herramienta con nombre, versión activa y acción inmediata; detalles secundarios no compiten con la versión.

## Iconografía, contenido y movimiento

No se requiere logo ni imagen. Preferir etiquetas de texto; si aparece iconografía funcional, usar una sola familia lineal de 16–20 px, sin emojis y con nombre accesible. No atribuir marcas o logotipos oficiales a PHP, Node.js o Composer sin recursos aprobados. Alinear versiones y rutas con Fira Code; nunca comunicar estado solo mediante color.

Transiciones breves: **140 ms** para hover y foco, **200 ms** para cambios de panel, curva `cubic-bezier(0.2, 0, 0, 1)`. Respetar reducción de movimiento del sistema con cambios inmediatos o sin desplazamiento. Especificar disparadores concretos en diseño de interacción.

## Aplicación y límites

Usar Red Violet para instalar, guardar o confirmar; su tono profundo señala hover, pulsado y foco. Usar Isabella Color solo como acento decorativo, y sus tonos derivados cuando el texto o el estado requieran contraste. Usar marfil como lienzo, blanco como área de trabajo y la superficie secundaria para selección o navegación. Mantener errores y avisos con texto explícito. Evitar teal, azul, degradados, sombras extensas, color como único estado y contenedores anidados sin necesidad.

El editable `pswitcher.figcraft` comparte esta identidad con las siguientes pantallas. Página **01 Sistema visual** muestra paleta, roles tipográficos y escala; **02 Catálogo UI** muestra piezas reutilizables; **03 Aplicación** es una composición de referencia, no una interfaz implementada.

## Pendientes

- Confirmar nombre del producto e idioma final; español es supuesto reversible.
- Verificar visualmente la aplicación en un escritorio Linux con compositor y revisar los estados de foco/teclado.
- Revisar la implementación frente a esta propuesta; cambios posteriores de identidad deben volver a maquetarse.
