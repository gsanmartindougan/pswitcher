use flate2::read::GzDecoder;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
    process::Stdio,
};
use tar::{Archive, EntryType};
use tempfile::TempDir;
use tokio::process::Command;

const PHP_RELEASES: &str = "https://www.php.net/releases/index.php?json";
const NODE_RELEASES: &str = "https://nodejs.org/dist/index.json";
const COMPOSER_RELEASES: &str = "https://getcomposer.org/versions";
const GO_RELEASES: &str = "https://go.dev/dl/?mode=json&include=all";
const FLUTTER_RELEASES: &str =
    "https://storage.googleapis.com/flutter_infra_release/releases/releases_linux.json";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tool {
    Php,
    Node,
    Composer,
    Go,
    Flutter,
}

impl Tool {
    pub const ALL: [Self; 5] = [
        Self::Php,
        Self::Node,
        Self::Composer,
        Self::Go,
        Self::Flutter,
    ];
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Release {
    pub version: String,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct InstallProgress {
    pub percent: f32,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhpExtensions {
    pub gd: bool,
    pub mbstring: bool,
    pub curl: bool,
    pub intl: bool,
    pub zip: bool,
    pub pdo_mysql: bool,
}

#[derive(Clone, Debug)]
pub struct VersionRemoval {
    original: PathBuf,
    trash_root: PathBuf,
}

fn report(progress: &mut impl FnMut(InstallProgress), percent: f32, label: &'static str) {
    progress(InstallProgress { percent, label });
}

#[derive(Deserialize)]
struct PhpRelease {
    version: String,
    #[serde(default)]
    supported_versions: Vec<String>,
    #[serde(default)]
    source: Vec<PhpSource>,
}

#[derive(Deserialize)]
struct PhpSource {
    filename: String,
    sha256: Option<String>,
}

#[derive(Deserialize)]
struct NodeRelease {
    version: String,
    files: Vec<String>,
}

#[derive(Deserialize)]
struct ComposerIndex {
    stable: Vec<ComposerRelease>,
}

#[derive(Deserialize)]
struct ComposerRelease {
    version: String,
    path: String,
}

#[derive(Deserialize)]
struct GoRelease {
    version: String,
    #[serde(default)]
    stable: bool,
    files: Vec<GoFile>,
}

#[derive(Deserialize)]
struct GoFile {
    filename: String,
    os: String,
    arch: String,
    sha256: String,
    kind: String,
}

#[derive(Deserialize)]
struct FlutterManifest {
    base_url: String,
    releases: Vec<FlutterRelease>,
}

#[derive(Deserialize)]
struct FlutterRelease {
    channel: String,
    version: String,
    #[serde(default)]
    dart_sdk_arch: Option<String>,
    archive: String,
    sha256: String,
}

pub async fn releases(tool: Tool) -> Result<Vec<Release>, String> {
    ensure_supported_platform()?;
    let client = reqwest::Client::new();
    match tool {
        Tool::Node => {
            let arch = node_arch()?;
            let releases: Vec<NodeRelease> = get_json(&client, NODE_RELEASES).await?;
            Ok(releases
                .into_iter()
                .filter(|item| {
                    item.files
                        .iter()
                        .any(|file| file == &format!("linux-{arch}"))
                })
                .map(|item| Release {
                    label: item.version.clone(),
                    version: item.version,
                })
                .collect())
        }
        Tool::Go => {
            let arch = go_arch()?;
            let versions: Vec<GoRelease> = get_json(&client, GO_RELEASES).await?;
            Ok(versions
                .into_iter()
                .filter(|release| {
                    release.stable
                        && release.files.iter().any(|file| {
                            file.os == "linux" && file.arch == arch && file.kind == "archive"
                        })
                })
                .filter_map(|release| {
                    let version = release.version.strip_prefix("go")?.to_owned();
                    valid_version(&version).then_some(Release {
                        label: version.clone(),
                        version,
                    })
                })
                .collect())
        }
        Tool::Flutter => {
            let arch = flutter_arch()?;
            let manifest: FlutterManifest = get_json(&client, FLUTTER_RELEASES).await?;
            Ok(manifest
                .releases
                .into_iter()
                .filter(|release| {
                    release.channel == "stable"
                        && release.dart_sdk_arch.as_deref() == Some(arch)
                        && valid_version(&release.version)
                })
                .map(|release| Release {
                    label: release.version.clone(),
                    version: release.version,
                })
                .collect())
        }
        Tool::Composer => {
            let index: ComposerIndex = get_json(&client, COMPOSER_RELEASES).await?;
            Ok(index
                .stable
                .into_iter()
                .filter(|item| {
                    valid_version(&item.version)
                        && item.path == format!("/download/{}/composer.phar", item.version)
                })
                .map(|item| Release {
                    label: item.version.clone(),
                    version: item.version,
                })
                .collect())
        }
        Tool::Php => {
            let index: serde_json::Map<String, serde_json::Value> =
                get_json(&client, PHP_RELEASES).await?;
            let mut branches = index
                .values()
                .filter_map(|v| serde_json::from_value::<PhpRelease>(v.clone()).ok())
                .flat_map(|v| v.supported_versions)
                .collect::<Vec<_>>();
            branches.sort();
            branches.dedup();
            let mut output = Vec::new();
            for branch in branches {
                if !valid_version(&branch) {
                    continue;
                }
                let url = format!("{PHP_RELEASES}&version={branch}");
                let release: PhpRelease = get_json(&client, &url).await?;
                if release
                    .source
                    .iter()
                    .any(|s| s.filename.ends_with(".tar.gz") && is_sha256(s.sha256.as_deref()))
                    && valid_version(&release.version)
                {
                    output.push(Release {
                        label: release.version.clone(),
                        version: release.version,
                    });
                }
            }
            output.sort_by(|a, b| b.version.cmp(&a.version));
            Ok(output)
        }
    }
}

pub async fn install(
    tool: Tool,
    version: &str,
    root: &Path,
    php_extensions: PhpExtensions,
    replace_php: bool,
    mut progress: impl FnMut(InstallProgress) + Send,
) -> Result<PathBuf, String> {
    ensure_supported_platform()?;
    if !valid_version(version) {
        return Err("Invalid version string".into());
    }
    if tool == Tool::Php && !php_build_dependencies_available(php_extensions) {
        return Err("One or more selected PHP extensions need missing development libraries. Install the extension dependencies shown on the installation screen, then try again.".into());
    }
    fs::create_dir_all(root).map_err(|e| format!("Cannot create install directory: {e}"))?;
    let destination = root.join(tool_name(tool)).join(version);
    if destination.exists() && !(tool == Tool::Php && replace_php) {
        return Err(format!(
            "Installation already exists: {}",
            destination.display()
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| "Invalid installation path".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("Cannot create install directory: {e}"))?;
    let stage =
        TempDir::new_in(root).map_err(|e| format!("Cannot create staging directory: {e}"))?;
    let payload = stage.path().join("payload");
    fs::create_dir(&payload).map_err(|e| format!("Cannot create staging payload: {e}"))?;

    report(&mut progress, 2.0, "Preparando instalación");
    match tool {
        Tool::Node => install_node(version, &payload, &mut progress).await?,
        Tool::Composer => install_composer(version, &payload, &mut progress).await?,
        Tool::Go => install_go(version, stage.path(), &payload, &mut progress).await?,
        Tool::Flutter => install_flutter(version, stage.path(), &payload, &mut progress).await?,
        Tool::Php => {
            install_php(
                version,
                stage.path(),
                &payload,
                &destination,
                php_extensions,
                &mut progress,
            )
            .await?
        }
    }
    report(&mut progress, 99.0, "Finalizando instalación");
    if destination.exists() {
        let backup = stage.path().join("previous-installation");
        fs::rename(&destination, &backup)
            .map_err(|e| format!("Could not preserve previous PHP installation: {e}"))?;
        if let Err(error) = fs::rename(&payload, &destination) {
            if let Err(rollback_error) = fs::rename(&backup, &destination) {
                let preserved_at = stage.keep();
                return Err(format!(
                    "Could not promote rebuilt PHP: {error}. Previous PHP is preserved at {}: {rollback_error}",
                    preserved_at.join("previous-installation").display()
                ));
            }
            return Err(format!("Could not promote rebuilt PHP: {error}"));
        }
    } else {
        fs::rename(&payload, &destination)
            .map_err(|e| format!("Could not promote completed installation: {e}"))?;
    }
    Ok(destination)
}

pub fn stage_version_removal(
    root: &Path,
    tool: Tool,
    version: &str,
) -> Result<VersionRemoval, String> {
    if !valid_version(version) {
        return Err("Invalid version string".into());
    }
    let original = root.join(tool_name(tool)).join(version);
    let metadata = fs::symlink_metadata(&original)
        .map_err(|error| format!("Could not inspect installed version: {error}"))?;
    if !metadata.file_type().is_dir() {
        return Err("The selected version is not a managed directory".into());
    }
    let executable = match tool {
        Tool::Php => original.join("bin/php"),
        Tool::Node => original.join("bin/node"),
        Tool::Composer => original.join("composer.phar"),
        Tool::Go => original.join("bin/go"),
        Tool::Flutter => original.join("bin/flutter"),
    };
    if !executable.is_file() {
        return Err("The selected folder is not a complete managed installation".into());
    }
    let parent = original
        .parent()
        .ok_or_else(|| "Invalid installation path".to_string())?;
    let trash = TempDir::new_in(parent)
        .map_err(|error| format!("Could not prepare version removal: {error}"))?;
    let trash_root = trash.keep();
    if let Err(error) = fs::rename(&original, trash_root.join("version")) {
        let _ = fs::remove_dir_all(&trash_root);
        return Err(format!(
            "Could not stage the selected version for removal: {error}"
        ));
    }
    Ok(VersionRemoval {
        original,
        trash_root,
    })
}

pub fn restore_version_removal(removal: VersionRemoval) -> Result<(), String> {
    fs::rename(removal.trash_root.join("version"), &removal.original).map_err(|error| {
        format!(
            "Could not restore the version; its files remain at {}: {error}",
            removal.trash_root.display()
        )
    })?;
    fs::remove_dir_all(&removal.trash_root)
        .map_err(|error| format!("Could not clean the temporary removal directory: {error}"))
}

pub fn finish_version_removal(removal: VersionRemoval) -> Result<(), String> {
    fs::remove_dir_all(&removal.trash_root).map_err(|error| {
        format!(
            "Could not completely remove the staged version at {}: {error}",
            removal.trash_root.display()
        )
    })
}

async fn install_node(
    version: &str,
    destination: &Path,
    progress: &mut (impl FnMut(InstallProgress) + Send),
) -> Result<(), String> {
    report(progress, 3.0, "Consultando catálogo de Node.js");
    let arch = node_arch()?;
    let client = reqwest::Client::new();
    let releases: Vec<NodeRelease> = get_json(&client, NODE_RELEASES).await?;
    if !releases.iter().any(|item| {
        item.version == version
            && item
                .files
                .iter()
                .any(|file| file == &format!("linux-{arch}"))
    }) {
        return Err(format!(
            "Node version is not available for Linux {arch}: {version}"
        ));
    }
    let base = format!("https://nodejs.org/dist/{version}/");
    let filename = format!("node-{version}-linux-{arch}.tar.gz");
    let sums = get_text(&client, &format!("{base}SHASUMS256.txt")).await?;
    let expected = checksum_line(&sums, &filename).ok_or("Node checksum not found")?;
    let archive = get_bytes(
        &client,
        &format!("{base}{filename}"),
        (8.0, 70.0),
        "Descargando Node.js",
        progress,
    )
    .await?;
    report(progress, 76.0, "Verificando Node.js");
    verify_sha256(&archive, expected)?;
    report(progress, 80.0, "Extrayendo Node.js");
    extract_tar_gz(
        &archive,
        destination,
        &format!("node-{version}-linux-{arch}"),
    )?;
    report(progress, 97.0, "Node.js listo");
    Ok(())
}

async fn install_go(
    version: &str,
    stage: &Path,
    destination: &Path,
    progress: &mut (impl FnMut(InstallProgress) + Send),
) -> Result<(), String> {
    report(progress, 3.0, "Consultando catálogo de Go");
    let arch = go_arch()?;
    let client = reqwest::Client::new();
    let versions: Vec<GoRelease> = get_json(&client, GO_RELEASES).await?;
    let tag = format!("go{version}");
    let file = versions
        .iter()
        .filter(|release| release.version == tag && release.stable)
        .flat_map(|release| &release.files)
        .find(|file| file.os == "linux" && file.arch == arch && file.kind == "archive")
        .ok_or_else(|| format!("Stable Go version is not available for Linux {arch}: {version}"))?;
    if !is_sha256(Some(&file.sha256)) {
        return Err("Go returned an invalid SHA-256".into());
    }
    let archive = stage.join(&file.filename);
    let actual = download_file(
        &client,
        &format!("https://go.dev/dl/{}", file.filename),
        &archive,
        (8.0, 72.0),
        "Descargando Go",
        progress,
    )
    .await?;
    report(progress, 76.0, "Verificando Go");
    verify_digest(&actual, &file.sha256)?;
    report(progress, 80.0, "Extrayendo Go");
    extract_tar_file(&archive, destination).await?;
    if !destination.join("bin/go").is_file() || !destination.join("bin/gofmt").is_file() {
        return Err("Go archive did not contain bin/go and bin/gofmt".into());
    }
    report(progress, 97.0, "Go listo");
    Ok(())
}

async fn install_flutter(
    version: &str,
    stage: &Path,
    destination: &Path,
    progress: &mut (impl FnMut(InstallProgress) + Send),
) -> Result<(), String> {
    report(progress, 3.0, "Consultando catálogo de Flutter");
    let arch = flutter_arch()?;
    let client = reqwest::Client::new();
    let manifest: FlutterManifest = get_json(&client, FLUTTER_RELEASES).await?;
    let release = manifest
        .releases
        .iter()
        .find(|release| {
            release.channel == "stable"
                && release.version == version
                && release.dart_sdk_arch.as_deref() == Some(arch)
                && valid_version(&release.version)
        })
        .ok_or_else(|| {
            format!("Stable Flutter version is not available for Linux {arch}: {version}")
        })?;
    if !is_sha256(Some(&release.sha256)) {
        return Err("Flutter returned an invalid SHA-256".into());
    }
    let archive = stage.join("flutter.tar.xz");
    let actual = download_file(
        &client,
        &format!(
            "{}/{}",
            manifest.base_url.trim_end_matches('/'),
            release.archive
        ),
        &archive,
        (5.0, 76.0),
        "Descargando Flutter",
        progress,
    )
    .await?;
    report(progress, 78.0, "Verificando Flutter");
    verify_digest(&actual, &release.sha256)?;
    report(progress, 82.0, "Extrayendo Flutter");
    extract_tar_file(&archive, destination).await?;
    if !destination.join("bin/flutter").is_file() || !destination.join("bin/dart").is_file() {
        return Err("Flutter archive did not contain bin/flutter and bin/dart".into());
    }
    report(progress, 97.0, "Flutter listo");
    Ok(())
}

async fn install_composer(
    version: &str,
    destination: &Path,
    progress: &mut (impl FnMut(InstallProgress) + Send),
) -> Result<(), String> {
    report(progress, 3.0, "Consultando catálogo de Composer");
    let client = reqwest::Client::new();
    let index: ComposerIndex = get_json(&client, COMPOSER_RELEASES).await?;
    let item = index
        .stable
        .iter()
        .find(|item| {
            item.version == version && item.path == format!("/download/{version}/composer.phar")
        })
        .ok_or_else(|| format!("Composer version is not available: {version}"))?;
    let base = "https://getcomposer.org";
    let phar = get_bytes(
        &client,
        &format!("{base}{}", item.path),
        (8.0, 70.0),
        "Descargando Composer",
        progress,
    )
    .await?;
    report(progress, 76.0, "Verificando Composer");
    let digest = get_text(&client, &format!("{base}{}.sha256sum", item.path)).await?;
    let expected = digest
        .split_whitespace()
        .next()
        .ok_or("Composer checksum missing")?;
    if !is_sha256(Some(expected)) {
        return Err("Composer returned an invalid SHA-256".into());
    }
    verify_sha256(&phar, expected)?;
    report(progress, 92.0, "Guardando Composer");
    let mut file = File::create(destination.join("composer.phar"))
        .map_err(|e| format!("Cannot write Composer archive: {e}"))?;
    file.write_all(&phar)
        .map_err(|e| format!("Cannot write Composer archive: {e}"))
}

async fn install_php(
    version: &str,
    stage: &Path,
    destination: &Path,
    install_prefix: &Path,
    extensions: PhpExtensions,
    progress: &mut (impl FnMut(InstallProgress) + Send),
) -> Result<(), String> {
    report(progress, 2.0, "Consultando catálogo de PHP");
    let client = reqwest::Client::new();
    let url = format!("{PHP_RELEASES}&version={version}");
    let release: PhpRelease = get_json(&client, &url).await?;
    let source = release
        .source
        .iter()
        .find(|source| source.filename == format!("php-{version}.tar.gz"))
        .ok_or_else(|| format!("PHP source for {version} is not available"))?;
    let expected = source.sha256.as_deref().ok_or("PHP checksum missing")?;
    if !is_sha256(Some(expected)) {
        return Err("PHP returned an invalid SHA-256".into());
    }
    let archive = get_bytes(
        &client,
        &format!("https://www.php.net/distributions/{}", source.filename),
        (6.0, 38.0),
        "Descargando PHP",
        progress,
    )
    .await?;
    report(progress, 40.0, "Verificando PHP");
    verify_sha256(&archive, expected)?;
    report(progress, 43.0, "Extrayendo código fuente de PHP");
    extract_tar_gz(&archive, destination, &format!("php-{version}"))?;

    let configure = destination.join("configure");
    let mut command = Command::new(configure);
    command
        .current_dir(destination)
        .arg(format!("--prefix={}", install_prefix.display()))
        .arg("--disable-all")
        .arg("--enable-cli")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    for option in [
        extensions.gd.then_some("--enable-gd"),
        extensions.gd.then_some("--with-jpeg"),
        extensions.gd.then_some("--with-freetype"),
        extensions.mbstring.then_some("--enable-mbstring"),
        extensions.curl.then_some("--with-curl"),
        extensions.intl.then_some("--enable-intl"),
        extensions.zip.then_some("--with-zip"),
        extensions.pdo_mysql.then_some("--enable-pdo"),
        extensions.pdo_mysql.then_some("--with-pdo-mysql=mysqlnd"),
    ]
    .into_iter()
    .flatten()
    {
        command.arg(option);
    }
    report(progress, 47.0, "Configurando PHP");
    run_build(command, "PHP configure").await?;

    let jobs = std::thread::available_parallelism().map_or(1, usize::from);
    let mut make = Command::new("make");
    make.current_dir(destination)
        .arg(format!("-j{jobs}"))
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    report(
        progress,
        57.0,
        "Compilando PHP (puede tardar varios minutos)",
    );
    run_build(make, "PHP make").await?;

    let mut install = Command::new("make");
    let install_root = stage.join("install-root");
    install
        .current_dir(destination)
        .arg("install")
        .arg(format!("INSTALL_ROOT={}", install_root.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    report(progress, 87.0, "Instalando PHP compilado");
    run_build(install, "PHP make install").await?;
    let built = install_root.join(
        install_prefix
            .strip_prefix("/")
            .map_err(|_| "PHP installation path must be absolute")?,
    );
    if !built.join("bin/php").is_file() {
        return Err("PHP build completed without bin/php".into());
    }
    if !php_extensions_present(&built.join("bin/php"), extensions) {
        return Err(
            "PHP build completed, but one or more selected extensions are not enabled".into(),
        );
    }
    fs::remove_dir_all(destination).map_err(|e| format!("Cannot clean PHP source staging: {e}"))?;
    fs::rename(built, destination).map_err(|e| format!("Cannot stage PHP installation: {e}"))?;
    report(progress, 97.0, "PHP listo");
    Ok(())
}

pub fn installed_php_extensions(root: &Path, version: &str) -> PhpExtensions {
    let php = root.join("php").join(version).join("bin/php");
    let modules = php_modules(&php);
    PhpExtensions {
        gd: modules.contains("gd"),
        mbstring: modules.contains("mbstring"),
        curl: modules.contains("curl"),
        intl: modules.contains("intl"),
        zip: modules.contains("zip"),
        pdo_mysql: modules.contains("pdo_mysql"),
    }
}

fn php_modules(php: &Path) -> std::collections::HashSet<String> {
    std::process::Command::new(php)
        .arg("-n")
        .arg("-m")
        .output()
        .map(|result| {
            if !result.status.success() {
                return std::collections::HashSet::new();
            }
            String::from_utf8_lossy(&result.stdout)
                .lines()
                .map(|module| module.trim().to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

fn php_extensions_present(php: &Path, requested: PhpExtensions) -> bool {
    let modules = php_modules(php);
    (!requested.gd || modules.contains("gd"))
        && (!requested.mbstring || modules.contains("mbstring"))
        && (!requested.curl || modules.contains("curl"))
        && (!requested.intl || modules.contains("intl"))
        && (!requested.zip || modules.contains("zip"))
        && (!requested.pdo_mysql || modules.contains("pdo_mysql"))
}

fn php_build_dependencies_available(extensions: PhpExtensions) -> bool {
    let mut packages = Vec::new();
    if extensions.gd {
        packages.extend(["libpng", "libjpeg", "freetype2"]);
    }
    if extensions.mbstring {
        packages.push("oniguruma");
    }
    if extensions.curl {
        packages.push("libcurl");
    }
    if extensions.intl {
        packages.extend(["icu-uc", "icu-i18n", "icu-io"]);
    }
    if extensions.zip {
        packages.push("libzip");
    }
    packages.is_empty()
        || std::process::Command::new("pkg-config")
            .arg("--exists")
            .args(packages)
            .status()
            .is_ok_and(|status| status.success())
}

async fn run_build(mut command: Command, step: &str) -> Result<(), String> {
    let output = command
        .output()
        .await
        .map_err(|e| format!("{step} could not start (check build dependencies): {e}"))?;
    if !output.status.success() {
        let details = String::from_utf8_lossy(&output.stderr);
        let details = details.trim();
        return Err(if details.is_empty() {
            format!("{step} failed with {}", output.status)
        } else {
            format!("{step} failed: {details}")
        });
    }
    Ok(())
}

fn extract_tar_gz(bytes: &[u8], destination: &Path, expected_root: &str) -> Result<(), String> {
    let decoder = GzDecoder::new(bytes);
    let mut archive = Archive::new(decoder);
    let entries = archive
        .entries()
        .map_err(|e| format!("Invalid gzip tar archive: {e}"))?;
    let mut count = 0usize;
    let mut links = Vec::new();
    for entry in entries {
        let mut entry = entry.map_err(|e| format!("Invalid tar entry: {e}"))?;
        let path = entry
            .path()
            .map_err(|e| format!("Invalid archive path: {e}"))?
            .into_owned();
        let mut components = path.components();
        if components.next() != Some(Component::Normal(OsStr::new(expected_root))) {
            return Err(format!("Unexpected archive root: {}", path.display()));
        }
        let relative = components.as_path();
        if relative.as_os_str().is_empty() {
            continue;
        }
        if relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(format!("Unsafe archive path: {}", path.display()));
        }
        let output = destination.join(relative);
        match entry.header().entry_type() {
            EntryType::Directory => {
                fs::create_dir_all(&output).map_err(|e| format!("Cannot extract directory: {e}"))?
            }
            EntryType::Regular | EntryType::GNUSparse => {
                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("Cannot extract file: {e}"))?;
                }
                let mut file =
                    File::create(&output).map_err(|e| format!("Cannot extract file: {e}"))?;
                io::copy(&mut entry, &mut file).map_err(|e| format!("Cannot extract file: {e}"))?;
                let mode = entry.header().mode().unwrap_or(0o644) & 0o777 & !0o6000;
                fs::set_permissions(&output, fs::Permissions::from_mode(mode))
                    .map_err(|e| format!("Cannot set extracted file permissions: {e}"))?;
                count += 1;
            }
            EntryType::Symlink => {
                let target = entry
                    .link_name()
                    .map_err(|e| format!("Invalid symbolic link: {e}"))?
                    .ok_or("Symbolic link has no target")?
                    .into_owned();
                let resolved = safe_link_target(relative, &target)
                    .ok_or_else(|| format!("Unsafe archive link: {}", path.display()))?;
                links.push((output, destination.join(resolved)));
            }
            _ => {
                return Err(format!(
                    "Unsupported link or special file in archive: {}",
                    path.display()
                ));
            }
        }
    }
    if count == 0 {
        return Err("Archive contained no files".into());
    }
    use std::os::unix::fs::symlink;
    for (link, target) in links {
        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Cannot extract link: {e}"))?;
        }
        let relative_target = target
            .strip_prefix(link.parent().unwrap_or(destination))
            .map_err(|_| "Unsafe archive link target".to_string())?;
        symlink(relative_target, link).map_err(|e| format!("Cannot extract symbolic link: {e}"))?;
    }
    Ok(())
}

fn safe_link_target(link_path: &Path, target: &Path) -> Option<PathBuf> {
    let mut resolved = link_path.parent()?.components().collect::<Vec<_>>();
    for component in target.components() {
        match component {
            Component::Normal(part) => resolved.push(Component::Normal(part)),
            Component::CurDir => {}
            Component::ParentDir if resolved.pop().is_some() => {}
            _ => return None,
        }
    }
    Some(resolved.into_iter().collect())
}

async fn get_json<T: for<'de> Deserialize<'de>>(
    client: &reqwest::Client,
    url: &str,
) -> Result<T, String> {
    client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Request failed: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Invalid response from {url}: {e}"))
}

async fn get_bytes(
    client: &reqwest::Client,
    url: &str,
    range: (f32, f32),
    label: &'static str,
    progress: &mut impl FnMut(InstallProgress),
) -> Result<Vec<u8>, String> {
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download failed: {e}"))?;
    let total = response.content_length().unwrap_or(0);
    let mut bytes = Vec::new();
    let mut received = 0_u64;
    let mut last_percent = range.0.floor();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("Download failed: {e}"))?
    {
        received += chunk.len() as u64;
        bytes.extend_from_slice(&chunk);
        if total > 0 {
            let percent =
                (range.0 + (received as f32 / total as f32) * (range.1 - range.0)).min(range.1);
            if percent.floor() > last_percent {
                report(progress, percent, label);
                last_percent = percent.floor();
            }
        }
    }
    report(progress, range.1, label);
    Ok(bytes)
}

async fn download_file(
    client: &reqwest::Client,
    url: &str,
    path: &Path,
    range: (f32, f32),
    label: &'static str,
    progress: &mut impl FnMut(InstallProgress),
) -> Result<String, String> {
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("Download failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Download failed: {error}"))?;
    let total = response.content_length().unwrap_or(0);
    let mut file = File::create(path).map_err(|error| format!("Cannot create archive: {error}"))?;
    let mut digest = Sha256::new();
    let mut received = 0_u64;
    let mut last_percent = range.0.floor();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("Download failed: {error}"))?
    {
        file.write_all(&chunk)
            .map_err(|error| format!("Cannot save downloaded archive: {error}"))?;
        digest.update(&chunk);
        received += chunk.len() as u64;
        if total > 0 {
            let percent =
                (range.0 + (received as f32 / total as f32) * (range.1 - range.0)).min(range.1);
            if percent.floor() > last_percent {
                report(progress, percent, label);
                last_percent = percent.floor();
            }
        }
    }
    file.flush()
        .map_err(|error| format!("Cannot finish downloaded archive: {error}"))?;
    report(progress, range.1, label);
    Ok(format!("{:x}", digest.finalize()))
}

async fn extract_tar_file(archive: &Path, destination: &Path) -> Result<(), String> {
    let output = Command::new("tar")
        .arg("--extract")
        .arg("--file")
        .arg(archive)
        .arg("--directory")
        .arg(destination)
        .arg("--strip-components=1")
        .arg("--no-same-owner")
        .arg("--no-same-permissions")
        .output()
        .await
        .map_err(|error| format!("Could not start tar to extract the SDK: {error}"))?;
    if !output.status.success() {
        let details = String::from_utf8_lossy(&output.stderr);
        return Err(format!("SDK extraction failed: {}", details.trim()));
    }
    Ok(())
}

async fn get_text(client: &reqwest::Client, url: &str) -> Result<String, String> {
    client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Request failed: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Invalid response from {url}: {e}"))
}

fn checksum_line<'a>(list: &'a str, filename: &str) -> Option<&'a str> {
    list.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let checksum = fields.next()?;
        let file = fields.next()?.trim_start_matches('*');
        (file == filename && is_sha256(Some(checksum))).then_some(checksum)
    })
}

fn verify_sha256(bytes: &[u8], expected: &str) -> Result<(), String> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err("SHA-256 checksum mismatch".into())
    }
}

fn verify_digest(actual: &str, expected: &str) -> Result<(), String> {
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err("SHA-256 checksum mismatch".into())
    }
}

fn is_sha256(value: Option<&str>) -> bool {
    value.is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn valid_version(version: &str) -> bool {
    let plain = version.strip_prefix('v').unwrap_or(version);
    !plain.is_empty()
        && plain != "."
        && plain != ".."
        && plain.as_bytes()[0].is_ascii_digit()
        && plain
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
}

fn tool_name(tool: Tool) -> &'static str {
    match tool {
        Tool::Php => "php",
        Tool::Node => "node",
        Tool::Composer => "composer",
        Tool::Go => "go",
        Tool::Flutter => "flutter",
    }
}

fn node_arch() -> Result<&'static str, String> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("x64"),
        "aarch64" => Ok("arm64"),
        arch => Err(format!(
            "Node installer does not support architecture {arch}"
        )),
    }
}

fn go_arch() -> Result<&'static str, String> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("amd64"),
        "aarch64" => Ok("arm64"),
        arch => Err(format!("Go installer does not support architecture {arch}")),
    }
}

fn flutter_arch() -> Result<&'static str, String> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("x64"),
        "aarch64" => Ok("arm64"),
        arch => Err(format!(
            "Flutter installer does not support architecture {arch}"
        )),
    }
}

fn ensure_supported_platform() -> Result<(), String> {
    if !cfg!(target_os = "linux") {
        return Err("Installers support Linux only".into());
    }
    node_arch().map(|_| ())
}

// shortcut: only official Linux x86_64/aarch64 archives and the latest PHP release per supported branch; expand when other targets or historical catalogs are required.
