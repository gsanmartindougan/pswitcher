use std::{
    fs,
    io::Write,
    os::unix::{fs::PermissionsExt, fs::symlink},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

const START: &str = "# >>> Pswitcher managed versions >>>";
const END: &str = "# <<< Pswitcher managed versions <<<";

pub fn bin_dir(root: &Path) -> PathBuf {
    root.join("bin")
}

pub fn sync_links(root: &Path, selected: &[(String, PathBuf)]) -> Result<(), String> {
    let bin = bin_dir(root);
    fs::create_dir_all(&bin).map_err(|error| error.to_string())?;

    for (tool, binary) in [("php", "php"), ("node", "node")] {
        let target = selected
            .iter()
            .find(|(name, _)| name == tool)
            .map(|(_, path)| path);
        match target {
            Some(target) => replace_link(&bin.join(tool), &target.join("bin").join(binary))?,
            None => remove_link(&bin.join(tool))?,
        }
    }

    if let Some((_, node_root)) = selected.iter().find(|(name, _)| name == "node") {
        for executable in ["npm", "npx", "corepack"] {
            let source = node_root.join("bin").join(executable);
            if source.is_file() {
                replace_link(&bin.join(executable), &source)?;
            } else {
                remove_link(&bin.join(executable))?;
            }
        }
    } else {
        for executable in ["npm", "npx", "corepack"] {
            remove_link(&bin.join(executable))?;
        }
    }

    if let Some((_, composer_root)) = selected.iter().find(|(name, _)| name == "composer") {
        let phar = composer_root.join("composer.phar");
        if phar.is_file() {
            write_composer_wrapper(&bin.join("composer"), &phar)?;
        } else {
            remove_composer_wrapper(&bin.join("composer"))?;
        }
    } else {
        remove_composer_wrapper(&bin.join("composer"))?;
    }

    for (tool, executables) in [("go", ["go", "gofmt"]), ("flutter", ["flutter", "dart"])] {
        if let Some((_, tool_root)) = selected.iter().find(|(name, _)| name == tool) {
            for executable in executables {
                let source = tool_root.join("bin").join(executable);
                if source.is_file() {
                    replace_link(&bin.join(executable), &source)?;
                } else {
                    remove_link(&bin.join(executable))?;
                }
            }
        } else {
            for executable in executables {
                remove_link(&bin.join(executable))?;
            }
        }
    }

    Ok(())
}

pub fn enable_bash(root: &Path) -> Result<String, String> {
    let block = bash_block(root);
    edit_bashrc(None, Some(&block))?;
    Ok(block)
}

pub fn disable_bash(expected_block: &str) -> Result<(), String> {
    edit_bashrc(Some(expected_block), None)
}

pub fn update_bash_root(root: &Path, expected_block: &str) -> Result<String, String> {
    let block = bash_block(root);
    edit_bashrc(Some(expected_block), Some(&block))?;
    Ok(block)
}

fn bash_block(root: &Path) -> String {
    let bin = bin_dir(root);
    let escaped = bin.to_string_lossy().replace('\'', "'\\''");
    format!("{START}\nexport PATH='{escaped}':\"$PATH\"\n{END}\n")
}

fn edit_bashrc(old_block: Option<&str>, new_block: Option<&str>) -> Result<(), String> {
    let home = directories::BaseDirs::new()
        .ok_or("No se pudo localizar el directorio personal")?
        .home_dir()
        .to_path_buf();
    let path = home.join(".bashrc");
    if fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err("~/.bashrc es un enlace simbólico; no se modificó".into());
    }
    let existing = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("No se pudo leer {}: {error}", path.display())),
    };

    let start_count = existing.matches(START).count();
    let end_count = existing.matches(END).count();
    if start_count != end_count || start_count > 1 {
        return Err(
            "El bloque Pswitcher de ~/.bashrc está duplicado o incompleto; no se modificó".into(),
        );
    }

    let updated = if let Some(old) = old_block {
        if start_count != 1 || existing.matches(old).count() != 1 {
            return Err(
                "El bloque Bash cambió desde que Pswitcher lo creó; no se modificó ~/.bashrc"
                    .into(),
            );
        }
        existing.replacen(old, new_block.unwrap_or_default(), 1)
    } else {
        if start_count != 0 {
            return Err("Ya existe un bloque Pswitcher en ~/.bashrc; no se modificó".into());
        }
        let mut result = existing;
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        if let Some(block) = new_block {
            result.push('\n');
            result.push_str(block);
        }
        result
    };

    atomic_write(&path, updated.as_bytes())
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("Ruta Bash inválida")?;
    let mut file = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    file.write_all(contents)
        .map_err(|error| error.to_string())?;
    if let Ok(metadata) = fs::metadata(path) {
        file.as_file()
            .set_permissions(fs::Permissions::from_mode(
                metadata.permissions().mode() & 0o7777,
            ))
            .map_err(|error| error.to_string())?;
    }
    file.persist(path).map_err(|error| error.to_string())?;
    Ok(())
}

fn replace_link(link: &Path, target: &Path) -> Result<(), String> {
    if let Ok(metadata) = fs::symlink_metadata(link) {
        if !metadata.file_type().is_symlink() {
            return Err(format!(
                "{} no es un enlace administrado por Pswitcher",
                link.display()
            ));
        }
    }
    let parent = link.parent().ok_or("Ruta de enlace inválida")?;
    let temporary = parent.join(format!(
        ".{}.new",
        link.file_name().unwrap_or_default().to_string_lossy()
    ));
    let _ = fs::remove_file(&temporary);
    symlink(target, &temporary).map_err(|error| error.to_string())?;
    fs::rename(&temporary, link).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        error.to_string()
    })
}

fn remove_link(link: &Path) -> Result<(), String> {
    match fs::symlink_metadata(link) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            fs::remove_file(link).map_err(|error| error.to_string())
        }
        Ok(_) => Err(format!(
            "{} no es un enlace administrado por Pswitcher",
            link.display()
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn write_composer_wrapper(path: &Path, phar: &Path) -> Result<(), String> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if !metadata.file_type().is_file()
            || !fs::read_to_string(path)
                .is_ok_and(|text| text.starts_with("#!/bin/sh\n# PSWITCHER MANAGED\n"))
        {
            return Err(format!(
                "{} no es un wrapper administrado por Pswitcher",
                path.display()
            ));
        }
    }
    let phar = phar.to_string_lossy().replace('\'', "'\\''");
    let contents = format!(
        "#!/bin/sh\n# PSWITCHER MANAGED\nphp=\"$(dirname \"$0\")/php\"\nif [ -x \"$php\" ]; then exec \"$php\" '{phar}' \"$@\"; fi\nexec php '{phar}' \"$@\"\n"
    );
    let parent = path.parent().ok_or("Ruta de Composer inválida")?;
    let mut file = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    file.write_all(contents.as_bytes())
        .map_err(|error| error.to_string())?;
    file.as_file()
        .set_permissions(fs::Permissions::from_mode(0o755))
        .map_err(|error| error.to_string())?;
    file.persist(path).map_err(|error| error.to_string())?;
    Ok(())
}

fn remove_composer_wrapper(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => {
            if !fs::read_to_string(path)
                .is_ok_and(|text| text.starts_with("#!/bin/sh\n# PSWITCHER MANAGED\n"))
            {
                return Err(format!(
                    "{} no es un wrapper administrado por Pswitcher",
                    path.display()
                ));
            }
            fs::remove_file(path).map_err(|error| error.to_string())
        }
        Ok(_) => Err(format!(
            "{} no es un wrapper administrado por Pswitcher",
            path.display()
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
