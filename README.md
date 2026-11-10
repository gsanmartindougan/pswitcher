# Pswitcher

**[English](README.en.md)**

Pswitcher es una aplicación de escritorio para Linux que instala y administra versiones de PHP, Node.js, Composer, Go y Flutter en una carpeta elegida por ti. Permite cambiar la versión activa desde la aplicación; la integración opcional con Bash también habilita los comandos seleccionados en terminales nuevas.

## Descargar

Descarga el AppImage x86_64 desde [Releases](https://github.com/gsanmartindougan/pswitcher/releases). Después:

```sh
chmod +x Pswitcher-*-x86_64.AppImage
./Pswitcher-*-x86_64.AppImage
```

## Uso

- Elige la carpeta de instalación en **Ajustes**. La ruta inicial es `~/Tools/pswitcher`.
- Instala y selecciona varias versiones de cada herramienta desde **Versiones**. Los catálogos se consultan al abrir la instalación; Node, Go y Flutter permiten buscar por número de versión.
- Para PHP, elige extensiones al instalar o reinstalar. Pswitcher compila PHP desde sus fuentes oficiales y comprueba las extensiones solicitadas.
- Activa la integración con Bash desde **Ajustes** para que Pswitcher administre su bloque marcado en `~/.bashrc`. Abre una terminal nueva tras cambiar la selección.
- Sin integración con Bash, la selección se aplica dentro de Pswitcher, pero no modifica el `PATH` de las terminales.
- **Ajustes** también comprueba las herramientas de compilación del sistema. En Debian/Ubuntu y derivados se indican dependencias para `apt`; en Fedora/RHEL y derivados, para `dnf`.

Las versiones quedan separadas por herramienta y número, dentro de la carpeta elegida. La configuración de la aplicación se guarda en el directorio XDG del usuario.

## Requisitos y alcance

- Linux en x86_64 o aarch64. El AppImage publicado en esta release es x86_64.
- Para compilar PHP se necesitan un compilador C, GNU Make, `tar`, `xz` y las bibliotecas de desarrollo requeridas por las extensiones elegidas. Pswitcher no instala paquetes del sistema por ti.
- La disponibilidad de extensiones PHP depende de sus bibliotecas de desarrollo presentes en la distro.
- Node.js, Composer, Go y Flutter se descargan desde sus catálogos y fuentes oficiales; Pswitcher valida las descargas cuando el catálogo proporciona una suma.
- Esta versión gestiona selecciones globales desde la aplicación; no aplica versiones por proyecto.

## Compilar desde el código fuente

Se necesita Rust estable y las dependencias de desarrollo de Iced para Linux (X11/Wayland). Para ejecutar en modo desarrollo:

```sh
cargo run
```

Para crear el AppImage x86_64 con el contenedor de empaquetado:

```sh
./packaging/build-appimage.sh
```

El archivo se genera en la raíz del proyecto como `Pswitcher-<versión>-x86_64.AppImage`.
