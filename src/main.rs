mod config;
mod installer;
mod shell;
mod system_check;
mod theme;

use std::{fs, path::PathBuf};

use iced::{
    Alignment, Element, Length, Task,
    futures::SinkExt,
    widget::{
        button, column, container, pick_list, progress_bar, row, scrollable, text, text_input,
    },
};
use installer::{Release, Tool};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Page {
    Versions,
    Preferences,
    Install,
}

#[derive(Clone)]
enum Message {
    Navigate(Page),
    RootEdited(String),
    BrowseRoot,
    SaveRoot,
    SelectVersion(Tool, String),
    OpenInstall(Tool),
    OpenReinstallPhp(String),
    RequestDeleteVersion(Tool, String),
    ConfirmDeleteVersion,
    CancelDeleteVersion,
    VersionRemovalFinished(Tool, String, Result<(), String>),
    InstallVersionChanged(String),
    PhpGdChanged(bool),
    PhpMbstringChanged(bool),
    PhpCurlChanged(bool),
    PhpIntlChanged(bool),
    PhpZipChanged(bool),
    PhpPdoMysqlChanged(bool),
    ConfirmInstall,
    CancelInstall,
    InstallProgress(installer::InstallProgress),
    Installed(Tool, String, Result<PathBuf, String>),
    ReleasesLoaded(Tool, Result<Vec<Release>, String>),
    RequestBash,
    ConfirmBash,
    CancelBash,
    DisableBash,
    CheckRequirements,
    RequirementsChecked(system_check::SystemCheck),
    CopyNotice,
    ClearNotice,
    LanguageChanged(config::Language),
}

struct App {
    config: config::Config,
    config_writable: bool,
    root_edit: String,
    page: Page,
    installed: [Vec<String>; 5],
    releases: [Vec<Release>; 5],
    catalog_loading: Option<Tool>,
    install_tool: Option<Tool>,
    install_version: String,
    reinstall_php: bool,
    php_gd: bool,
    php_mbstring: bool,
    php_curl: bool,
    php_intl: bool,
    php_zip: bool,
    php_pdo_mysql: bool,
    installing: Option<(Tool, String)>,
    install_progress: Option<installer::InstallProgress>,
    pending_delete: Option<(Tool, String)>,
    deleting_version: Option<(Tool, String)>,
    confirm_bash: bool,
    notice: Option<String>,
    system_check: Option<system_check::SystemCheck>,
    checking_system: bool,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (config, notice, config_writable) = match config::load() {
            Ok(config) => (config, None, true),
            Err(error) => (config::Config::default(), Some(error), false),
        };
        let root_edit = config.install_root.to_string_lossy().into_owned();
        let mut app = Self {
            installed: scan_all(&config.install_root),
            config,
            config_writable,
            root_edit,
            page: Page::Versions,
            releases: std::array::from_fn(|_| Vec::new()),
            catalog_loading: None,
            install_tool: None,
            install_version: String::new(),
            reinstall_php: false,
            php_gd: false,
            php_mbstring: false,
            php_curl: false,
            php_intl: false,
            php_zip: false,
            php_pdo_mysql: false,
            installing: None,
            install_progress: None,
            pending_delete: None,
            deleting_version: None,
            confirm_bash: false,
            notice,
            system_check: None,
            checking_system: false,
        };
        if app.config.bash_enabled {
            if let Err(error) = sync_selected(&app.config) {
                app.notice = Some(error);
            }
        }
        (app, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(page) => {
                if self.installing.is_none() && self.deleting_version.is_none() {
                    if self.page == Page::Install && page != Page::Install {
                        self.clear_install_catalog();
                    }
                    self.page = page;
                    if page == Page::Preferences && self.system_check.is_none() {
                        return self.check_requirements();
                    }
                }
            }
            Message::LanguageChanged(language) => self.set_language(language),
            Message::RootEdited(path) => self.root_edit = path,
            Message::BrowseRoot => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(&self.config.install_root)
                    .pick_folder()
                {
                    self.root_edit = path.to_string_lossy().into_owned();
                }
            }
            Message::SaveRoot => self.save_root(),
            Message::SelectVersion(tool, version) => {
                if self.deleting_version.is_none() {
                    self.select_version(tool, version);
                }
            }
            Message::OpenInstall(tool) => return self.open_install_task(tool),
            Message::OpenReinstallPhp(version) => {
                self.install_tool = Some(Tool::Php);
                self.install_version = version.clone();
                self.reinstall_php = true;
                let extensions =
                    installer::installed_php_extensions(&self.config.install_root, &version);
                self.php_gd = extensions.gd;
                self.php_mbstring = extensions.mbstring;
                self.php_curl = extensions.curl;
                self.php_intl = extensions.intl;
                self.php_zip = extensions.zip;
                self.php_pdo_mysql = extensions.pdo_mysql;
                self.install_progress = None;
                self.releases[tool_index(Tool::Php)] = Vec::new();
                self.catalog_loading = None;
                self.page = Page::Install;
            }
            Message::RequestDeleteVersion(tool, version) => {
                if self.deleting_version.is_none() {
                    self.pending_delete = Some((tool, version));
                }
            }
            Message::ConfirmDeleteVersion => return self.delete_version_task(),
            Message::CancelDeleteVersion => self.pending_delete = None,
            Message::VersionRemovalFinished(tool, version, result) => {
                self.deleting_version = None;
                self.installed = scan_all(&self.config.install_root);
                self.notice = Some(match result {
                    Ok(()) => format!(
                        "{} {} {}",
                        tool_name(tool),
                        version,
                        self.tr("eliminado", "removed")
                    ),
                    Err(error) => error,
                });
            }
            Message::InstallVersionChanged(version) => {
                self.install_version = version;
                if let Some(tool) = self
                    .install_tool
                    .filter(|tool| matches!(tool, Tool::Node | Tool::Go | Tool::Flutter))
                {
                    if !self.install_version.trim().is_empty()
                        && self.releases[tool_index(tool)].is_empty()
                        && self.catalog_loading != Some(tool)
                    {
                        return self.load_catalog_task(tool);
                    }
                }
            }
            Message::PhpGdChanged(enabled) => self.php_gd = enabled,
            Message::PhpMbstringChanged(enabled) => self.php_mbstring = enabled,
            Message::PhpCurlChanged(enabled) => self.php_curl = enabled,
            Message::PhpIntlChanged(enabled) => self.php_intl = enabled,
            Message::PhpZipChanged(enabled) => self.php_zip = enabled,
            Message::PhpPdoMysqlChanged(enabled) => self.php_pdo_mysql = enabled,
            Message::ConfirmInstall => return self.install_task(),
            Message::CancelInstall => {
                self.clear_install_catalog();
                self.page = Page::Versions;
            }
            Message::InstallProgress(progress) => self.install_progress = Some(progress),
            Message::Installed(tool, version, result) => {
                self.installing = None;
                self.install_progress = None;
                self.clear_install_catalog();
                self.install_tool = None;
                self.reinstall_php = false;
                self.php_gd = false;
                self.php_mbstring = false;
                self.php_curl = false;
                self.php_intl = false;
                self.php_zip = false;
                self.php_pdo_mysql = false;
                self.page = Page::Versions;
                match result {
                    Ok(_) => {
                        self.installed[tool_index(tool)].push(version.clone());
                        self.installed[tool_index(tool)].sort();
                        self.installed[tool_index(tool)].dedup();
                        self.notice = Some(format!(
                            "{} {version} {}",
                            tool_name(tool),
                            self.tr("instalado", "installed")
                        ));
                    }
                    Err(error) => self.notice = Some(error),
                }
            }
            Message::ReleasesLoaded(tool, result) => {
                if self.page == Page::Install && self.install_tool == Some(tool) {
                    self.catalog_loading = None;
                    let search_has_query = !matches!(tool, Tool::Node | Tool::Go | Tool::Flutter)
                        || !self.install_version.trim().is_empty();
                    match result {
                        Ok(releases) if search_has_query => {
                            self.releases[tool_index(tool)] = releases;
                        }
                        Ok(_) => self.releases[tool_index(tool)] = Vec::new(),
                        Err(error) if search_has_query => self.notice = Some(error),
                        Err(_) => {}
                    }
                }
            }
            Message::RequestBash => self.confirm_bash = true,
            Message::ConfirmBash => self.enable_bash(),
            Message::CancelBash => self.confirm_bash = false,
            Message::DisableBash => self.disable_bash(),
            Message::CheckRequirements => return self.check_requirements(),
            Message::RequirementsChecked(result) => {
                self.system_check = Some(result);
                self.checking_system = false;
            }
            Message::CopyNotice => {
                if let Some(notice) = &self.notice {
                    return iced::clipboard::write(notice.clone());
                }
            }
            Message::ClearNotice => self.notice = None,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let header = row![
            column![
                text("Pswitcher").size(19),
                container(text(""))
                    .height(Length::Fixed(2.0))
                    .width(Length::Fixed(42.0))
                    .style(|_| { container::Style::default().background(theme::ISABELLA) }),
                text("PHP · Node · Composer · Go · Flutter").size(10)
            ]
            .spacing(1)
            .width(Length::Fill),
            nav_button(self.tr("Versiones", "Versions"), Page::Versions, self.page),
            nav_button(self.tr("Ajustes", "Settings"), Page::Preferences, self.page),
        ]
        .spacing(6)
        .padding([10, 12])
        .align_y(Alignment::Center);
        let body = match self.page {
            Page::Versions => self.versions_page(),
            Page::Preferences => self.preferences_page(),
            Page::Install => self.install_page(),
        };

        let mut content = column![body].spacing(18).width(Length::Fill);
        if let Some(notice) = &self.notice {
            content = content.push(
                container(
                    column![
                        row![
                            button(self.tr("Copiar mensaje", "Copy message"))
                                .on_press(Message::CopyNotice),
                            button(self.tr("Cerrar", "Close")).on_press(Message::ClearNotice),
                        ]
                        .spacing(8),
                        text(notice).width(Length::Fill),
                    ]
                    .spacing(8),
                )
                .padding(12)
                .style(container::rounded_box),
            );
        }

        let page_content: Element<'_, Message> = if self.page == Page::Install {
            column![
                scrollable(container(content).padding(14).width(Length::Fill)).height(Length::Fill),
                container(self.install_actions())
                    .padding([0, 14])
                    .width(Length::Fill),
            ]
            .height(Length::Fill)
            .into()
        } else {
            scrollable(container(content).padding(14).width(Length::Fill))
                .height(Length::Fill)
                .into()
        };

        column![
            container(header).style(container::rounded_box),
            page_content
        ]
        .spacing(8)
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn versions_page(&self) -> Element<'_, Message> {
        let mut rows = column![].spacing(12);
        for tool in Tool::ALL {
            let selected = self.config.selected.get(tool_key(tool)).cloned();
            let installed = self.installed[tool_index(tool)].clone();
            let selector = pick_list(installed.clone(), selected.clone(), move |version| {
                Message::SelectVersion(tool, version)
            })
            .placeholder(self.tr("Sin versión instalada", "No version installed"))
            .width(Length::Fill);

            let current = self
                .config
                .selected
                .get(tool_key(tool))
                .map(|version| {
                    if self.installed[tool_index(tool)].contains(version) {
                        format!("{} {version}", self.tr("Activa:", "Active:"))
                    } else {
                        format!(
                            "{} {version}",
                            self.tr("Guardada, no instalada aquí:", "Saved, not installed here:")
                        )
                    }
                })
                .unwrap_or_else(|| {
                    self.tr("Elige una versión instalada", "Choose an installed version")
                        .into()
                });
            let selected_installed = selected
                .as_ref()
                .filter(|version| installed.contains(version))
                .cloned();
            let mut card = column![
                row![text(tool_name(tool)).size(16), text(current).size(11)]
                    .align_y(Alignment::Center)
                    .spacing(6),
                selector,
            ]
            .spacing(8);
            let install_label = if tool == Tool::Php {
                self.tr("Instalar", "Install")
            } else {
                self.tr("Instalar versión", "Install version")
            };
            let install_button = button(install_label)
                .style(theme::primary_button)
                .on_press_maybe(
                    self.deleting_version
                        .is_none()
                        .then_some(Message::OpenInstall(tool)),
                );
            if tool == Tool::Php {
                let mut actions = row![install_button.width(Length::FillPortion(1))]
                    .spacing(6)
                    .width(Length::Fill);
                if let Some(version) = selected_installed.clone() {
                    actions = actions.push(
                        button(self.tr("Reinstalar", "Reinstall"))
                            .width(Length::FillPortion(1))
                            .on_press_maybe(
                                self.deleting_version
                                    .is_none()
                                    .then_some(Message::OpenReinstallPhp(version.clone())),
                            ),
                    );
                    actions = actions.push(
                        button(self.tr("Eliminar", "Delete"))
                            .width(Length::FillPortion(1))
                            .on_press_maybe(
                                (self.deleting_version.is_none() && self.pending_delete.is_none())
                                    .then_some(Message::RequestDeleteVersion(tool, version)),
                            ),
                    );
                }
                card = card.push(actions);
            } else {
                card = card.push(install_button.width(Length::Fill));
            }
            if let Some(version) = selected_installed {
                if self.pending_delete.as_ref() == Some(&(tool, version.clone())) {
                    card = card.push(text(format!(
                        "{} {} {}?",
                        self.tr("¿Eliminar", "Delete"),
                        tool_name(tool),
                        version
                    )));
                    card = card.push(text(
                        self.tr("No se puede deshacer.", "This cannot be undone."),
                    ));
                    card = card.push(
                        row![
                            button(self.tr("Cancelar", "Cancel"))
                                .on_press(Message::CancelDeleteVersion),
                            button(self.tr("Eliminar", "Delete"))
                                .style(theme::primary_button)
                                .on_press(Message::ConfirmDeleteVersion),
                        ]
                        .spacing(8),
                    );
                } else if self.deleting_version.as_ref() == Some(&(tool, version.clone())) {
                    card = card.push(text(self.tr("Eliminando versión…", "Removing version…")));
                } else if tool != Tool::Php {
                    card = card.push(
                        button(
                            self.tr("Eliminar versión seleccionada…", "Remove selected version…"),
                        )
                        .width(Length::Fill)
                        .on_press_maybe(
                            (self.deleting_version.is_none() && self.pending_delete.is_none())
                                .then_some(Message::RequestDeleteVersion(tool, version)),
                        ),
                    );
                }
            }
            rows = rows.push(container(card).padding(12).style(container::rounded_box));
        }

        column![rows,].spacing(10).into()
    }

    fn install_page(&self) -> Element<'_, Message> {
        let Some(tool) = self.install_tool else {
            return column![text(self.tr(
                "No hay una instalación seleccionada.",
                "No installation selected."
            ))]
            .into();
        };
        let releases = &self.releases[tool_index(tool)];
        let options: Vec<String> = releases
            .iter()
            .map(|release| release.version.clone())
            .collect();
        let selector: Element<'_, Message> = if self.reinstall_php {
            column![text(format!(
                "{} {}",
                self.tr("Versión instalada", "Installed version"),
                self.install_version
            ))]
            .into()
        } else if matches!(tool, Tool::Node | Tool::Go | Tool::Flutter) {
            let query = self.install_version.trim().trim_start_matches('v');
            let input = text_input(
                self.tr("Número de versión exacto", "Exact version number"),
                &self.install_version,
            )
            .on_input(Message::InstallVersionChanged)
            .on_submit(Message::ConfirmInstall)
            .width(Length::Fill);
            let suggestions = releases
                .iter()
                .filter(|release| {
                    !query.is_empty() && release.version.trim_start_matches('v').starts_with(query)
                })
                .take(8)
                .fold(column![input].spacing(4), |items, release| {
                    items.push(
                        button(text(&release.version))
                            .width(Length::Fill)
                            .on_press(Message::InstallVersionChanged(release.version.clone())),
                    )
                });
            suggestions.into()
        } else {
            pick_list(
                options,
                (!self.install_version.is_empty()).then(|| self.install_version.clone()),
                Message::InstallVersionChanged,
            )
            .placeholder(self.tr(
                "Selecciona versión disponible",
                "Select an available version",
            ))
            .width(Length::Fill)
            .into()
        };

        let extensions: Element<'_, Message> = if tool == Tool::Php {
            column![
                text(self.tr("Extensiones PHP", "PHP extensions")).size(16),
                iced::widget::checkbox(self.php_gd)
                    .label(self.tr("GD · imágenes PNG/JPEG", "GD · PNG/JPEG images"))
                    .on_toggle(Message::PhpGdChanged),
                iced::widget::checkbox(self.php_mbstring)
                    .label("mbstring · Unicode")
                    .on_toggle(Message::PhpMbstringChanged),
                iced::widget::checkbox(self.php_curl)
                    .label("cURL · HTTP")
                    .on_toggle(Message::PhpCurlChanged),
                iced::widget::checkbox(self.php_intl)
                    .label(self.tr("Intl · internacionalización", "Intl · internationalization"))
                    .on_toggle(Message::PhpIntlChanged),
                iced::widget::checkbox(self.php_zip)
                    .label(self.tr("ZIP · archivos comprimidos", "ZIP · compressed archives"))
                    .on_toggle(Message::PhpZipChanged),
                iced::widget::checkbox(self.php_pdo_mysql)
                    .label("PDO MySQL · mysqlnd")
                    .on_toggle(Message::PhpPdoMysqlChanged),
                text(self.tr(
                    "Instala manualmente las dependencias de las extensiones seleccionadas. No se instalarán paquetes del sistema automáticamente.",
                    "Install the selected extensions' dependencies manually. System packages are never installed automatically.",
                ))
                .size(11),
                text(self.tr(
                    "Debian: sudo apt install pkg-config libpng-dev libjpeg-dev libfreetype-dev libonig-dev libcurl4-openssl-dev libicu-dev libzip-dev",
                    "Debian: sudo apt install pkg-config libpng-dev libjpeg-dev libfreetype-dev libonig-dev libcurl4-openssl-dev libicu-dev libzip-dev",
                ))
                .size(10),
                text(self.tr(
                    "Fedora: sudo dnf install pkgconf-pkg-config libpng-devel libjpeg-turbo-devel freetype-devel oniguruma-devel libcurl-devel libicu-devel libzip-devel",
                    "Fedora: sudo dnf install pkgconf-pkg-config libpng-devel libjpeg-turbo-devel freetype-devel oniguruma-devel libcurl-devel libicu-devel libzip-devel",
                ))
                .size(10),
            ]
            .spacing(6)
            .into()
        } else {
            column![text("")].into()
        };

        let progress = if let Some(progress) = &self.install_progress {
            column![
                text(format!(
                    "{} · {:.0}%",
                    self.progress_label(progress.label),
                    progress.percent
                ))
                .size(12),
                progress_bar(0.0..=100.0, progress.percent),
            ]
            .spacing(6)
        } else {
            column![text("")]
        };

        let catalog_status: Element<'_, Message> = if self.catalog_loading == Some(tool) {
            text(self.tr(
                "Cargando versiones disponibles…",
                "Loading available versions…",
            ))
            .size(12)
            .into()
        } else {
            column![].into()
        };

        column![
            button(self.tr("← Volver a Versiones", "← Back to versions"))
                .on_press_maybe(self.installing.is_none().then_some(Message::CancelInstall),),
            text(if self.reinstall_php {
                format!(
                    "{} PHP {}",
                    self.tr("Reinstalar", "Reinstall"),
                    self.install_version
                )
            } else {
                format!("{} {}", self.tr("Instalar", "Install"), tool_name(tool))
            })
            .size(24),
            text(if matches!(tool, Tool::Node | Tool::Go | Tool::Flutter) {
                self.tr(
                    "Escribe una versión o parte de ella y elige una sugerencia del catálogo oficial.",
                    "Enter a version or part of one, then choose a suggestion from the official catalog.",
                )
            } else {
                self.tr(
                    "Selecciona una versión disponible. La descarga se verifica antes de instalarse.",
                    "Select an available version. The download is verified before installation.",
                )
            })
            .size(13),
            selector,
            catalog_status,
            extensions,
            progress,
        ]
        .spacing(14)
        .into()
    }

    fn install_actions(&self) -> Element<'_, Message> {
        let Some(tool) = self.install_tool else {
            return row![].into();
        };

        row![
            button(self.tr("Cancelar", "Cancel"))
                .on_press_maybe(self.installing.is_none().then_some(Message::CancelInstall),),
            button(if self.reinstall_php {
                self.tr("Reinstalar", "Reinstall")
            } else {
                self.tr("Instalar", "Install")
            })
            .style(theme::primary_button)
            .on_press_maybe(
                (!self.install_version.trim().is_empty()
                    && (!matches!(tool, Tool::Node | Tool::Go | Tool::Flutter)
                        || self.releases[tool_index(tool)].iter().any(|release| {
                            release.version.trim_start_matches('v')
                                == self.install_version.trim().trim_start_matches('v')
                        }))
                    && self.installing.is_none())
                .then_some(Message::ConfirmInstall)
            ),
        ]
        .spacing(12)
        .into()
    }

    fn preferences_page(&self) -> Element<'_, Message> {
        let bash_status = self.tr(
            if self.config.bash_enabled {
                "Activa"
            } else {
                "Desactivada"
            },
            if self.config.bash_enabled {
                "Enabled"
            } else {
                "Disabled"
            },
        );
        let bash_action = if self.confirm_bash {
            column![
                text(self.tr(
                    "Pswitcher añadirá un bloque propio a ~/.bashrc. El resto del archivo se conserva.",
                    "Pswitcher will add its own block to ~/.bashrc and preserve the rest of the file.",
                )),
                row![
                    button(self.tr("Cancelar", "Cancel")).on_press(Message::CancelBash),
                    button(self.tr("Confirmar", "Confirm"))
                        .style(theme::primary_button)
                        .on_press(Message::ConfirmBash),
                ]
                .spacing(8),
            ]
            .spacing(8)
        } else if self.config.bash_enabled {
            column![
                text(self.tr(
                    "Las terminales Bash nuevas usarán las versiones elegidas.",
                    "New Bash terminals will use the selected versions.",
                )),
                button(self.tr("Desactivar Bash", "Disable Bash")).on_press(Message::DisableBash),
            ]
            .spacing(8)
        } else {
            column![
                text(self.tr(
                    "La selección de la app no cambia comandos del terminal hasta activar esta opción.",
                    "App selections do not change terminal commands until this option is enabled.",
                )),
                button(self.tr("Activar integración Bash", "Enable Bash integration"))
                    .style(theme::primary_button)
                    .on_press(Message::RequestBash),
            ]
            .spacing(8)
        };
        let selected_language = match self.config.language {
            config::Language::Spanish => "Español",
            config::Language::English => "English",
        };
        let requirements = if self.checking_system {
            column![text(
                self.tr("Comprobando requisitos…", "Checking requirements…")
            )]
            .spacing(8)
        } else if let Some(check) = &self.system_check {
            let ok = self.tr("Correcto", "Ready");
            let missing = self.tr("No encontrado", "Missing");
            let architecture = if check.supported_architecture {
                format!("{} · {ok}", check.architecture)
            } else {
                format!(
                    "{} · {}",
                    check.architecture,
                    self.tr("No compatible", "Unsupported")
                )
            };
            let compiler = check.compiler.as_deref().unwrap_or(missing);
            let make = if check.gnu_make { ok } else { missing };
            let bash = if check.bash { ok } else { missing };
            let distro = match check.package_family {
                system_check::PackageFamily::Debian => {
                    self.tr("Debian y derivados", "Debian family")
                }
                system_check::PackageFamily::Fedora => {
                    self.tr("Fedora y derivados", "Fedora family")
                }
                system_check::PackageFamily::Other => {
                    self.tr("Otra distribución", "Other distribution")
                }
            };
            let package_hint = match check.package_family {
                system_check::PackageFamily::Debian => self.tr(
                    "En Debian/Ubuntu y derivados: sudo apt install build-essential tar xz-utils",
                    "On Debian/Ubuntu and derivatives: sudo apt install build-essential tar xz-utils",
                ),
                system_check::PackageFamily::Fedora => self.tr(
                    "En Fedora/RHEL y derivados: sudo dnf install gcc make tar xz",
                    "On Fedora/RHEL and derivatives: sudo dnf install gcc make tar xz",
                ),
                system_check::PackageFamily::Other => self.tr(
                    "Instala un compilador C, GNU Make, tar y xz con el gestor de paquetes de tu distro.",
                    "Install a C compiler, GNU Make, tar and xz with your distribution's package manager.",
                ),
            };
            let tar = if check.tar { ok } else { missing };
            let xz = if check.xz { ok } else { missing };
            column![
                text(format!("Linux: {}", if check.linux { ok } else { missing })).size(12),
                text(format!("{}: {distro}", self.tr("Familia detectada", "Detected family"))).size(12),
                text(format!("{}: {architecture}", self.tr("Arquitectura", "Architecture"))).size(12),
                text(format!("{}: {compiler}", self.tr("Compilador C + enlazador", "C compiler + linker"))).size(12),
                text(format!("GNU Make: {make}")).size(12),
                text(format!("tar: {tar}")).size(12),
                text(format!("xz: {xz}")).size(12),
                text(format!("Bash: {bash} · {}", self.tr("opcional", "optional"))).size(12),
                if check.compiler.is_none() || !check.gnu_make || !check.tar || !check.xz {
                    text(package_hint).size(11)
                } else {
                    text(self.tr(
                        "Los paquetes no se instalan automáticamente.",
                        "Packages are not installed automatically.",
                    )).size(11)
                },
                text(self.tr(
                    "PHP necesita un compilador C y GNU Make; Go requiere tar y Flutter, tar y xz. Las extensiones PHP opcionales requieren bibliotecas adicionales y no se validan aquí.",
                    "PHP needs a C compiler and GNU Make; Go needs tar, and Flutter needs tar and xz. Optional PHP extensions need additional libraries and are not checked here.",
                )).size(11),
            ]
            .spacing(6)
        } else {
            column![text(self.tr(
                "Comprueba los requisitos básicos para instalar PHP, Go y Flutter.",
                "Check the basic requirements for installing PHP, Go and Flutter.",
            ))]
            .spacing(8)
        };

        column![
            text(self.tr("PREFERENCIAS", "PREFERENCES")).size(12),
            text(self.tr("Ajustes", "Settings")).size(24),
            container(
                row![
                    text(self.tr("Idioma", "Language")).width(Length::Fill),
                    pick_list(
                        ["Español", "English"],
                        Some(selected_language),
                        |language| {
                            Message::LanguageChanged(if language == "English" {
                                config::Language::English
                            } else {
                                config::Language::Spanish
                            })
                        }
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(8),
            )
            .padding(12)
            .style(container::rounded_box),
            container(
                column![
                    text(self.tr("Carpeta de instalaciones", "Installation folder")).size(16),
                    text(self.tr("DIRECTORIO DE INSTALACIÓN", "INSTALLATION DIRECTORY")).size(10),
                    text(self.config.install_root.display().to_string()).size(11),
                    row![
                        text_input(
                            self.tr("Ruta de instalación", "Installation path"),
                            &self.root_edit
                        )
                        .on_input(Message::RootEdited)
                        .on_submit(Message::SaveRoot)
                        .width(Length::Fill),
                        button("…").on_press(Message::BrowseRoot),
                    ]
                    .spacing(6),
                    button(self.tr("Guardar carpeta", "Save folder"))
                        .width(Length::Fill)
                        .on_press(Message::SaveRoot),
                    text(self.tr(
                        "Los catálogos se consultan cuando hacen falta.",
                        "Catalogs are fetched only when needed.",
                    ))
                    .size(11),
                    text(self.tr(
                        "Una subcarpeta por herramienta y versión.",
                        "A subfolder is created for each tool and version.",
                    ))
                    .size(11),
                ]
                .spacing(8),
            )
            .padding(12)
            .style(container::rounded_box),
            container(
                column![
                    text(self.tr("Integración con Bash", "Bash integration")).size(16),
                    text(format!("{}: {bash_status}", self.tr("Estado", "Status"))).size(12),
                    bash_action,
                ]
                .spacing(8),
            )
            .padding(12)
            .style(container::rounded_box),
            container(
                column![
                    text(self.tr("Requisitos del sistema", "System requirements")).size(16),
                    requirements,
                    button(if self.system_check.is_some() {
                        self.tr("Volver a comprobar", "Check again")
                    } else {
                        self.tr("Comprobar", "Check now")
                    })
                    .width(Length::Fill)
                    .on_press_maybe((!self.checking_system).then_some(Message::CheckRequirements)),
                ]
                .spacing(8),
            )
            .padding(12)
            .style(container::rounded_box),
        ]
        .spacing(10)
        .into()
    }

    fn set_language(&mut self, language: config::Language) {
        if !self.config_writable || self.config.language == language {
            return;
        }
        let previous = self.config.language;
        self.config.language = language;
        if let Err(error) = config::save(&self.config) {
            self.config.language = previous;
            self.notice = Some(error);
        }
    }

    fn tr(&self, spanish: &'static str, english: &'static str) -> &'static str {
        match self.config.language {
            config::Language::Spanish => spanish,
            config::Language::English => english,
        }
    }

    fn progress_label(&self, label: &'static str) -> &'static str {
        if self.config.language == config::Language::Spanish {
            return label;
        }
        match label {
            "Preparando instalación" => "Preparing installation",
            "Consultando catálogo de Node.js" => "Fetching Node.js catalog",
            "Descargando Node.js" => "Downloading Node.js",
            "Verificando Node.js" => "Verifying Node.js",
            "Extrayendo Node.js" => "Extracting Node.js",
            "Node.js listo" => "Node.js ready",
            "Consultando catálogo de Go" => "Fetching Go catalog",
            "Descargando Go" => "Downloading Go",
            "Verificando Go" => "Verifying Go",
            "Extrayendo Go" => "Extracting Go",
            "Go listo" => "Go ready",
            "Consultando catálogo de Flutter" => "Fetching Flutter catalog",
            "Descargando Flutter" => "Downloading Flutter",
            "Verificando Flutter" => "Verifying Flutter",
            "Extrayendo Flutter" => "Extracting Flutter",
            "Flutter listo" => "Flutter ready",
            "Consultando catálogo de Composer" => "Fetching Composer catalog",
            "Descargando Composer" => "Downloading Composer",
            "Verificando Composer" => "Verifying Composer",
            "Guardando Composer" => "Saving Composer",
            "Consultando catálogo de PHP" => "Fetching PHP catalog",
            "Descargando PHP" => "Downloading PHP",
            "Verificando PHP" => "Verifying PHP",
            "Extrayendo código fuente de PHP" => "Extracting PHP source",
            "Configurando PHP" => "Configuring PHP",
            "Compilando PHP (puede tardar varios minutos)" => {
                "Building PHP (this may take several minutes)"
            }
            "Instalando PHP compilado" => "Installing compiled PHP",
            "PHP listo" => "PHP ready",
            "Finalizando instalación" => "Finishing installation",
            _ => label,
        }
    }

    fn open_install_task(&mut self, tool: Tool) -> Task<Message> {
        self.install_tool = Some(tool);
        self.install_version.clear();
        self.reinstall_php = false;
        self.php_gd = false;
        self.php_mbstring = false;
        self.php_curl = false;
        self.php_intl = false;
        self.php_zip = false;
        self.php_pdo_mysql = false;
        self.install_progress = None;
        self.releases[tool_index(tool)] = Vec::new();
        self.catalog_loading = None;
        self.notice = None;
        self.page = Page::Install;
        if matches!(tool, Tool::Node | Tool::Go | Tool::Flutter) {
            Task::none()
        } else {
            self.load_catalog_task(tool)
        }
    }

    fn load_catalog_task(&mut self, tool: Tool) -> Task<Message> {
        self.catalog_loading = Some(tool);
        Task::perform(installer::releases(tool), move |result| {
            Message::ReleasesLoaded(tool, result)
        })
    }

    fn clear_install_catalog(&mut self) {
        if let Some(tool) = self.install_tool.take() {
            self.releases[tool_index(tool)] = Vec::new();
        }
        self.catalog_loading = None;
    }

    fn check_requirements(&mut self) -> Task<Message> {
        self.checking_system = true;
        Task::perform(
            async {
                tokio::task::spawn_blocking(system_check::check)
                    .await
                    .unwrap()
            },
            Message::RequirementsChecked,
        )
    }

    fn install_task(&mut self) -> Task<Message> {
        let Some(tool) = self.install_tool else {
            return Task::none();
        };
        let entered = self.install_version.trim();
        let version = if matches!(tool, Tool::Node | Tool::Go | Tool::Flutter) {
            let query = entered.trim_start_matches('v');
            let Some(release) = self.releases[tool_index(tool)]
                .iter()
                .find(|release| release.version.trim_start_matches('v') == query)
            else {
                self.notice = Some(
                    self.tr(
                        "Elige una versión exacta de las sugerencias.",
                        "Choose an exact version from the suggestions.",
                    )
                    .into(),
                );
                return Task::none();
            };
            release.version.clone()
        } else {
            entered.to_owned()
        };
        let install_version = version.clone();
        self.installing = Some((tool, version.clone()));
        self.install_progress = Some(installer::InstallProgress {
            percent: 0.0,
            label: "Preparando instalación",
        });
        let root = self.config.install_root.clone();
        let php_extensions = installer::PhpExtensions {
            gd: self.php_gd,
            mbstring: self.php_mbstring,
            curl: self.php_curl,
            intl: self.php_intl,
            zip: self.php_zip,
            pdo_mysql: self.php_pdo_mysql,
        };
        let replace_php = self.reinstall_php;
        Task::run(
            iced::stream::channel::<Message>(
                128,
                move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                    let result = installer::install(
                        tool,
                        &install_version,
                        &root,
                        php_extensions,
                        replace_php,
                        |progress| {
                            let _ = output.try_send(Message::InstallProgress(progress));
                        },
                    )
                    .await;
                    let _ = output.send(Message::Installed(tool, version, result)).await;
                },
            ),
            |message| message,
        )
    }

    fn delete_version_task(&mut self) -> Task<Message> {
        let Some((tool, version)) = self.pending_delete.clone() else {
            return Task::none();
        };
        if self.deleting_version.is_some() {
            return Task::none();
        }
        if !self.config_writable {
            self.pending_delete = None;
            self.notice =
                Some("No se puede actualizar la configuración; no se borró la versión".into());
            return Task::none();
        }
        let removal =
            match installer::stage_version_removal(&self.config.install_root, tool, &version) {
                Ok(removal) => removal,
                Err(error) => {
                    self.pending_delete = None;
                    self.notice = Some(error);
                    return Task::none();
                }
            };

        let previous = self.config.clone();
        if self
            .config
            .selected
            .get(tool_key(tool))
            .is_some_and(|selected| selected == &version)
        {
            self.config.selected.remove(tool_key(tool));
        }
        if let Err(error) = self.commit_config_and_links(previous) {
            self.pending_delete = None;
            self.notice = Some(match installer::restore_version_removal(removal) {
                Ok(()) => error,
                Err(restore_error) => format!("{error}; {restore_error}"),
            });
            return Task::none();
        }

        self.pending_delete = None;
        self.deleting_version = Some((tool, version.clone()));
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || installer::finish_version_removal(removal))
                    .await
                    .unwrap_or_else(|error| Err(format!("Version removal task failed: {error}")))
            },
            move |result| Message::VersionRemovalFinished(tool, version, result),
        )
    }

    fn select_version(&mut self, tool: Tool, version: String) {
        if !self.installed[tool_index(tool)].contains(&version) {
            self.notice = Some(
                self.tr(
                    "Solo se pueden activar versiones instaladas",
                    "Only installed versions can be selected",
                )
                .into(),
            );
            return;
        }
        let previous = self.config.clone();
        self.config.selected.insert(tool_key(tool).into(), version);
        if let Err(error) = self.commit_config_and_links(previous) {
            self.notice = Some(error);
        }
    }

    fn save_root(&mut self) {
        let root = expand_path(&self.root_edit);
        if !root.is_absolute() {
            self.notice = Some(
                self.tr(
                    "El directorio debe ser una ruta absoluta o empezar por ~/",
                    "The folder must be an absolute path or start with ~/",
                )
                .into(),
            );
            return;
        }
        let previous = self.config.clone();
        self.config.install_root = root.clone();
        if let Err(error) = self.commit_config_and_links(previous) {
            self.root_edit = self.config.install_root.to_string_lossy().into_owned();
            self.notice = Some(error);
            return;
        }
        self.installed = scan_all(&root);
        self.notice = Some(self.tr("Directorio guardado", "Folder saved").into());
    }

    fn commit_config_and_links(&mut self, previous: config::Config) -> Result<(), String> {
        if !self.config_writable {
            return Err("No se pudo leer la configuración; se evitará sobrescribirla".into());
        }
        let mut changed_bash_block = None;
        if self.config.bash_enabled && previous.install_root != self.config.install_root {
            let expected = previous
                .bash_block
                .as_deref()
                .ok_or("No se encontró el bloque Bash administrado")?;
            let updated = shell::update_bash_root(&self.config.install_root, expected)?;
            self.config.bash_block = Some(updated.clone());
            changed_bash_block = Some(updated);
        }
        if let Err(error) = config::save(&self.config) {
            if let Some(updated) = &changed_bash_block {
                let _ = shell::update_bash_root(&previous.install_root, updated);
            }
            self.config = previous;
            return Err(error);
        }
        if let Err(error) = sync_selected(&self.config) {
            if let Some(updated) = &changed_bash_block {
                let _ = shell::update_bash_root(&previous.install_root, updated);
            }
            self.config = previous;
            let _ = config::save(&self.config);
            let _ = sync_selected(&self.config);
            return Err(error);
        }
        Ok(())
    }

    fn enable_bash(&mut self) {
        if !self.config_writable {
            self.notice =
                Some("No se pudo leer la configuración; se evitará sobrescribirla".into());
            self.confirm_bash = false;
            return;
        }
        let mut updated = self.config.clone();
        updated.bash_enabled = true;
        let result = match sync_selected(&updated) {
            Err(error) => Err(error),
            Ok(()) => match shell::enable_bash(&updated.install_root) {
                Ok(block) => {
                    updated.bash_block = Some(block.clone());
                    match config::save(&updated) {
                        Ok(()) => {
                            self.config = updated;
                            Ok(())
                        }
                        Err(error) => {
                            let _ = shell::disable_bash(&block);
                            Err(error)
                        }
                    }
                }
                Err(error) => Err(error),
            },
        };
        self.confirm_bash = false;
        self.notice = Some(match result {
            Ok(()) => self
                .tr(
                    "Integración Bash activada; abre una terminal nueva",
                    "Bash integration enabled; open a new terminal",
                )
                .into(),
            Err(error) => error,
        });
    }

    fn disable_bash(&mut self) {
        if !self.config_writable {
            self.notice =
                Some("No se pudo leer la configuración; se evitará sobrescribirla".into());
            return;
        }
        let result = (|| {
            let mut updated = self.config.clone();
            let block = updated
                .bash_block
                .as_deref()
                .ok_or("No se encontró el bloque Bash administrado")?;
            shell::disable_bash(block)?;
            updated.bash_enabled = false;
            updated.bash_block = None;
            if let Err(error) = config::save(&updated) {
                let _ = shell::enable_bash(&self.config.install_root);
                return Err(error);
            }
            self.config = updated;
            Ok(())
        })();
        self.notice = Some(match result {
            Ok(()) => self
                .tr("Integración Bash desactivada", "Bash integration disabled")
                .into(),
            Err(error) => error,
        });
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(|app: &App| {
            format!(
                "Pswitcher · {}",
                app.tr("Gestor de versiones", "Version manager")
            )
        })
        .theme(|_: &App| theme::theme())
        .window_size(iced::Size::new(430.0, 610.0))
        .resizable(false)
        .run()
}

fn nav_button(
    label: &'static str,
    page: Page,
    current: Page,
) -> iced::widget::Button<'static, Message> {
    let button = button(text(label)).width(Length::Shrink);
    if page == current {
        button
            .style(theme::primary_button)
            .on_press(Message::Navigate(page))
    } else {
        button
            .style(button::secondary)
            .on_press(Message::Navigate(page))
    }
}

fn tool_key(tool: Tool) -> &'static str {
    match tool {
        Tool::Php => "php",
        Tool::Node => "node",
        Tool::Composer => "composer",
        Tool::Go => "go",
        Tool::Flutter => "flutter",
    }
}

fn tool_name(tool: Tool) -> &'static str {
    match tool {
        Tool::Php => "PHP",
        Tool::Node => "Node.js",
        Tool::Composer => "Composer",
        Tool::Go => "Go",
        Tool::Flutter => "Flutter",
    }
}

fn tool_index(tool: Tool) -> usize {
    match tool {
        Tool::Php => 0,
        Tool::Node => 1,
        Tool::Composer => 2,
        Tool::Go => 3,
        Tool::Flutter => 4,
    }
}

fn scan_all(root: &std::path::Path) -> [Vec<String>; 5] {
    Tool::ALL.map(|tool| {
        let path = root.join(tool_key(tool));
        let Ok(entries) = fs::read_dir(path) else {
            return Vec::new();
        };
        let mut versions = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|version| match tool {
                Tool::Php => root.join("php").join(version).join("bin/php").is_file(),
                Tool::Node => root.join("node").join(version).join("bin/node").is_file(),
                Tool::Composer => root
                    .join("composer")
                    .join(version)
                    .join("composer.phar")
                    .is_file(),
                Tool::Go => root.join("go").join(version).join("bin/go").is_file(),
                Tool::Flutter => root
                    .join("flutter")
                    .join(version)
                    .join("bin/flutter")
                    .is_file(),
            })
            .collect::<Vec<_>>();
        versions.sort();
        versions
    })
}

fn sync_selected(config: &config::Config) -> Result<(), String> {
    if !config.bash_enabled {
        return Ok(());
    }
    let selected = Tool::ALL
        .into_iter()
        .filter_map(|tool| {
            config.selected.get(tool_key(tool)).map(|version| {
                (
                    tool_key(tool).to_owned(),
                    config.install_root.join(tool_key(tool)).join(version),
                )
            })
        })
        .filter(|(tool, path)| match tool.as_str() {
            "php" => path.join("bin/php").is_file(),
            "node" => path.join("bin/node").is_file(),
            "composer" => path.join("composer.phar").is_file(),
            "go" => path.join("bin/go").is_file(),
            "flutter" => path.join("bin/flutter").is_file(),
            _ => false,
        })
        .collect::<Vec<_>>();
    shell::sync_links(&config.install_root, &selected)
}

fn expand_path(raw: &str) -> PathBuf {
    if raw == "~" {
        directories::BaseDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(raw))
    } else if let Some(rest) = raw.strip_prefix("~/") {
        directories::BaseDirs::new()
            .map(|dirs| dirs.home_dir().join(rest))
            .unwrap_or_else(|| PathBuf::from(raw))
    } else {
        PathBuf::from(raw)
    }
}
