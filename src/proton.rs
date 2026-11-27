use anyhow::{Context, Result, bail};
use std::{
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

/// Locate a Proton executable (`proton` script) from an explicit path,
/// `KELVAROQ_PROTON`, or common Steam / compatibilitytools.d directories.
pub(crate) fn find_proton(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        if p.is_file() {
            return Ok(p.to_path_buf());
        }
        let candidate = p.join("proton");
        if candidate.is_file() {
            return Ok(candidate);
        }
        bail!("Proton not found at {}", p.display());
    }

    if let Some(env_p) = std::env::var_os("KELVAROQ_PROTON") {
        let p = PathBuf::from(env_p);
        if p.is_file() {
            return Ok(p);
        }
        let candidate = p.join("proton");
        if candidate.is_file() {
            return Ok(candidate);
        }
        bail!("KELVAROQ_PROTON points to missing path: {}", p.display());
    }

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    let search_roots = [
        home.join(".steam/steam/steamapps/common"),
        home.join(".local/share/Steam/steamapps/common"),
        home.join(".steam/root/compatibilitytools.d"),
        home.join(".local/share/Steam/compatibilitytools.d"),
        PathBuf::from("/usr/share/steam/compatibilitytools.d"),
    ];

    let mut candidates = Vec::new();
    for root in &search_roots {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.contains("Proton") || name_str.contains("proton") || name_str.contains("GE") {
                    let proton_bin = path.join("proton");
                    if proton_bin.is_file() {
                        candidates.push(proton_bin);
                    }
                }
            }
        }
    }

    // Prefer Proton Experimental or highest-versioned Proton.
    candidates.sort();
    candidates.reverse();
    if let Some(found) = candidates.into_iter().next() {
        return Ok(found);
    }

    bail!(
        "Could not find Proton automatically. Pass --proton <path/to/proton> or set KELVAROQ_PROTON."
    )
}

/// Locate the Windows `kelvaroq.exe` binary next to the current Linux executable
/// or via `KELVAROQ_WINDOWS_EXE`.
pub(crate) fn find_windows_exe(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        if p.is_file() {
            return Ok(p.to_path_buf());
        }
        bail!("Windows kelvaroq.exe not found at {}", p.display());
    }
    if let Some(env_p) = std::env::var_os("KELVAROQ_WINDOWS_EXE") {
        let p = PathBuf::from(env_p);
        if p.is_file() {
            return Ok(p);
        }
        bail!("KELVAROQ_WINDOWS_EXE points to missing file: {}", p.display());
    }
    let exe = std::env::current_exe()?;
    let sibling = exe.with_file_name("kelvaroq.exe");
    if sibling.is_file() {
        return Ok(sibling);
    }
    bail!(
        "Windows kelvaroq.exe not found next to {} (expected {}). Cross-compile for x86_64-pc-windows-gnu or pass --windows-exe.",
        exe.display(),
        sibling.display()
    )
}

/// Run `kelvaroq.exe play ...` under Proton.
pub(crate) fn run_with_proton(
    proton: &Path,
    windows_exe: &Path,
    image: &Path,
    console: &str,
    experimental_render: bool,
    reference: bool,
    compat_data: Option<&Path>,
) -> Result<ExitCode> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    let compat_path = match compat_data {
        Some(p) => p.to_path_buf(),
        None => std::env::var_os("STEAM_COMPAT_DATA_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share/kelvaroq/proton-prefix")),
    };
    std::fs::create_dir_all(&compat_path)
        .with_context(|| format!("creating Proton compatdata dir {}", compat_path.display()))?;

    let steam_client = std::env::var_os("STEAM_COMPAT_CLIENT_INSTALL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let p1 = home.join(".steam/steam");
            if p1.is_dir() {
                p1
            } else {
                home.join(".local/share/Steam")
            }
        });

    let mut cmd = Command::new(proton);
    cmd.arg("run")
        .arg(windows_exe)
        .arg("play")
        .arg(image)
        .arg("--console")
        .arg(console);

    if experimental_render {
        cmd.arg("--experimental-render");
    }
    if reference {
        cmd.arg("--reference");
    }

    cmd.env("STEAM_COMPAT_DATA_PATH", &compat_path)
        .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &steam_client);

    let status = cmd
        .status()
        .with_context(|| format!("launching Proton ({})", proton.display()))?;
    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}
