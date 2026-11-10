use std::{fs, process::Command};

#[derive(Clone, Copy, Debug)]
pub enum PackageFamily {
    Debian,
    Fedora,
    Other,
}

#[derive(Clone, Debug)]
pub struct SystemCheck {
    pub linux: bool,
    pub architecture: String,
    pub supported_architecture: bool,
    pub compiler: Option<String>,
    pub gnu_make: bool,
    pub tar: bool,
    pub xz: bool,
    pub bash: bool,
    pub package_family: PackageFamily,
}

pub fn check() -> SystemCheck {
    let architecture = std::env::consts::ARCH.to_owned();
    SystemCheck {
        linux: cfg!(target_os = "linux"),
        supported_architecture: matches!(architecture.as_str(), "x86_64" | "aarch64"),
        architecture,
        compiler: c_compiler(),
        gnu_make: has_gnu_make(),
        tar: has_command("tar"),
        xz: has_command("xz"),
        bash: Command::new("bash")
            .arg("--version")
            .output()
            .is_ok_and(|result| result.status.success()),
        package_family: package_family(),
    }
}

fn has_command(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .is_ok_and(|result| result.status.success())
}

fn c_compiler() -> Option<String> {
    for name in ["cc", "gcc", "clang"] {
        let Ok(version) = Command::new(name).arg("--version").output() else {
            continue;
        };
        if !version.status.success() {
            continue;
        }
        let Ok(directory) = tempfile::tempdir() else {
            return None;
        };
        let source = directory.path().join("check.c");
        let binary = directory.path().join("check");
        if fs::write(&source, b"int main(void) { return 0; }\n").is_err() {
            return None;
        }
        let builds = Command::new(name)
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .is_ok_and(|result| result.status.success());
        if builds {
            let version = String::from_utf8_lossy(&version.stdout);
            let version = version.lines().next().unwrap_or(name);
            return Some(format!("{name} · {version}"));
        }
    }
    None
}

fn has_gnu_make() -> bool {
    Command::new("make")
        .arg("--version")
        .output()
        .is_ok_and(|result| {
            result.status.success()
                && String::from_utf8_lossy(&result.stdout)
                    .lines()
                    .next()
                    .is_some_and(|line| line.contains("GNU Make"))
        })
}

fn package_family() -> PackageFamily {
    let identifiers = fs::read_to_string("/etc/os-release")
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(key, _)| matches!(*key, "ID" | "ID_LIKE"))
        .map(|(_, value)| value.trim_matches('"').to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    if ["debian", "ubuntu"]
        .iter()
        .any(|id| identifiers.contains(id))
    {
        PackageFamily::Debian
    } else if ["fedora", "rhel", "centos"]
        .iter()
        .any(|id| identifiers.contains(id))
    {
        PackageFamily::Fedora
    } else {
        PackageFamily::Other
    }
}
