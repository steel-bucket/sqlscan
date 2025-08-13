// setup.rs - Installation script for SQLScan
use clap::{Arg, Command};
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, exit};

const VERSION: &str = "2.0";
const AUTHOR: &str = "Ghost (Rust Port)";

// Installation paths
const LINUX_INSTALL_PATH: &str = "/usr/share/sqlscan";
const LINUX_EXEC_PATH: &str = "/usr/bin/sqlscan";
const MACOS_INSTALL_PATH: &str = "/usr/local/share/sqlscan";
const MACOS_EXEC_PATH: &str = "/usr/local/bin/sqlscan";

fn main() {
    let matches = build_cli().get_matches();

    let (install_path, exec_path) = get_platform_paths();

    if matches.get_flag("install")
        && !matches.get_flag("reinstall")
        && !matches.get_flag("uninstall")
    {
        handle_install(&install_path, &exec_path);
    } else if matches.get_flag("uninstall")
        && !matches.get_flag("install")
        && !matches.get_flag("reinstall")
    {
        handle_uninstall(&install_path, &exec_path);
    } else if matches.get_flag("reinstall")
        && !matches.get_flag("install")
        && !matches.get_flag("uninstall")
    {
        handle_reinstall(&install_path, &exec_path);
    } else {
        show_metadata();
        println!();
        build_cli().print_help().unwrap();
    }
}

fn build_cli() -> Command {
    Command::new("sqlscan-setup")
        .version(VERSION)
        .author(AUTHOR)
        .about("SQLScan installation and setup utility")
        .arg(
            Arg::new("install")
                .short('i')
                .long("install")
                .help("Install sqlscan in the system")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("reinstall")
                .short('r')
                .long("reinstall")
                .help("Remove old files and reinstall to the system")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("uninstall")
                .short('u')
                .long("uninstall")
                .help("Uninstall sqlscan from the system")
                .action(clap::ArgAction::SetTrue),
        )
}

fn show_metadata() {
    println!("SQLScan ({}) by {}", VERSION, AUTHOR);
    println!("Massive SQL injection vulnerability scanner");
}

fn get_platform_paths() -> (PathBuf, PathBuf) {
    let os = std::env::consts::OS;

    match os {
        "linux" => {
            check_root_access();
            (
                PathBuf::from(LINUX_INSTALL_PATH),
                PathBuf::from(LINUX_EXEC_PATH),
            )
        }
        "macos" => (
            PathBuf::from(MACOS_INSTALL_PATH),
            PathBuf::from(MACOS_EXEC_PATH),
        ),
        "windows" => {
            eprintln!("Windows platform is not supported for installation");
            exit(1);
        }
        _ => {
            eprintln!("Unsupported platform: {}", os);
            exit(1);
        }
    }
}

fn check_root_access() {
    #[cfg(unix)]
    {
        if unsafe { libc::getuid() } != 0 {
            eprintln!("Linux system requires root access for the installation");
            exit(1);
        }
    }
}

fn handle_install(install_path: &Path, exec_path: &Path) {
    if install_path.exists() {
        eprintln!(
            "sqlscan is already installed under {}",
            install_path.display()
        );
        exit(1);
    }

    if exec_path.exists() {
        eprintln!("executable file exists under {}", exec_path.display());
        exit(1);
    }

    if let Err(e) = install(install_path, exec_path) {
        eprintln!("Installation failed: {}", e);
        exit(1);
    }

    println!("Installation finished");
    println!("Files are installed under {}", install_path.display());
    println!("Run: sqlscan --help");
}

fn handle_uninstall(install_path: &Path, exec_path: &Path) {
    uninstall(install_path, exec_path);

    println!("Uninstallation finished");
}

fn handle_reinstall(install_path: &Path, exec_path: &Path) {
    uninstall(install_path, exec_path);
    println!("Removed previous installed files");

    if let Err(e) = install(install_path, exec_path) {
        eprintln!("Reinstallation failed: {}", e);
        exit(1);
    }

    println!("Reinstallation finished");
    println!("Files are installed under {}", install_path.display());
    println!("Run: sqlscan --help");
}

fn install(install_path: &Path, exec_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create installation directory
    fs::create_dir_all(install_path)?;

    // Get current executable path
    let current_exe = std::env::current_exe()?;
    let current_dir = current_exe
        .parent()
        .ok_or("Cannot determine current directory")?;

    // Check if we're running from a built binary or development environment
    let binary_name = if current_exe.file_name().unwrap_or_default() == "setup" {
        "sqlscan"
    } else {
        current_exe
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("sqlscan")
    };

    let source_binary = current_dir.join(binary_name);
    let dest_binary = install_path.join(binary_name);

    // Copy main binary
    if source_binary.exists() {
        fs::copy(&source_binary, &dest_binary)?;
    } else {
        // Try to build the project first
        println!("Building sqlscan...");
        let output = ProcessCommand::new("cargo")
            .args(&["build", "--release"])
            .current_dir(current_dir)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to build sqlscan: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        // Copy the built binary
        let release_binary = current_dir.join("target/release").join(binary_name);
        if release_binary.exists() {
            fs::copy(&release_binary, &dest_binary)?;
        } else {
            return Err("Built binary not found".into());
        }
    }

    // Copy additional files if they exist
    let files_to_copy = vec![
        ("Cargo.toml", "Cargo.toml"),
        ("README.md", "README.md"),
        ("LICENSE", "LICENSE"),
    ];

    for (src_name, dest_name) in files_to_copy {
        let src_path = current_dir.join(src_name);
        if src_path.exists() {
            let dest_path = install_path.join(dest_name);
            fs::copy(&src_path, &dest_path).ok(); // Don't fail if optional files don't exist
        }
    }

    // Copy source directory structure
    let src_dir = current_dir.join("");
    if src_dir.exists() {
        let dest_src = install_path.join("");
        copy_dir_recursive(&src_dir, &dest_src)?;
    }

    // Create executable wrapper script
    create_executable_wrapper(exec_path, install_path, binary_name)?;

    println!("Installation completed successfully");
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

fn create_executable_wrapper(
    exec_path: &Path,
    install_path: &Path,
    binary_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(exec_path)?;

    writeln!(file, "#!/bin/bash")?;
    writeln!(file)?;
    writeln!(file, "{}/{} \"$@\"", install_path.display(), binary_name)?;

    // Set executable permissions
    #[cfg(unix)]
    {
        let metadata = file.metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(exec_path, permissions)?;
    }

    Ok(())
}

fn uninstall(install_path: &Path, exec_path: &Path) {
    if install_path.exists() {
        if let Err(e) = fs::remove_dir_all(install_path) {
            eprintln!("Failed to remove {}: {}", install_path.display(), e);
        } else {
            println!("Removed {}", install_path.display());
        }
    }

    if exec_path.exists() {
        if let Err(e) = fs::remove_file(exec_path) {
            eprintln!("Failed to remove {}: {}", exec_path.display(), e);
        } else {
            println!("Removed {}", exec_path.display());
        }
    }
}

fn _prompt_user(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cli_creation() {
        let app = build_cli();
        assert_eq!(app.get_name(), "sqlscan-setup");
        assert_eq!(app.get_version(), Some(VERSION));
    }

    #[test]
    fn test_cli_install_flag() {
        let app = build_cli();
        let matches = app.try_get_matches_from(vec!["sqlscan-setup", "--install"]);
        assert!(matches.is_ok());

        let matches = matches.unwrap();
        assert!(matches.get_flag("install"));
    }

    #[test]
    fn test_cli_uninstall_flag() {
        let app = build_cli();
        let matches = app.try_get_matches_from(vec!["sqlscan-setup", "--uninstall"]);
        assert!(matches.is_ok());

        let matches = matches.unwrap();
        assert!(matches.get_flag("uninstall"));
    }

    #[test]
    fn test_cli_reinstall_flag() {
        let app = build_cli();
        let matches = app.try_get_matches_from(vec!["sqlscan-setup", "--reinstall"]);
        assert!(matches.is_ok());

        let matches = matches.unwrap();
        assert!(matches.get_flag("reinstall"));
    }

    #[test]
    fn test_get_platform_paths_non_windows() {
        // This test will pass on non-Windows systems
        if std::env::consts::OS != "windows" {
            let (install_path, exec_path) = if std::env::consts::OS == "linux" {
                (
                    PathBuf::from(LINUX_INSTALL_PATH),
                    PathBuf::from(LINUX_EXEC_PATH),
                )
            } else {
                (
                    PathBuf::from(MACOS_INSTALL_PATH),
                    PathBuf::from(MACOS_EXEC_PATH),
                )
            };

            assert!(install_path.is_absolute());
            assert!(exec_path.is_absolute());
        }
    }

    #[test]
    fn test_copy_dir_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        let dest_dir = temp_dir.path().join("dest");

        // Create source structure
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("file1.txt"), "content1").unwrap();

        let sub_dir = src_dir.join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("file2.txt"), "content2").unwrap();

        // Test copy
        copy_dir_recursive(&src_dir, &dest_dir).unwrap();

        // Verify
        assert!(dest_dir.exists());
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("subdir").exists());
        assert!(dest_dir.join("subdir/file2.txt").exists());

        let content1 = fs::read_to_string(dest_dir.join("file1.txt")).unwrap();
        let content2 = fs::read_to_string(dest_dir.join("subdir/file2.txt")).unwrap();

        assert_eq!(content1, "content1");
        assert_eq!(content2, "content2");
    }

    #[test]
    fn test_create_executable_wrapper() {
        let temp_dir = TempDir::new().unwrap();
        let exec_path = temp_dir.path().join("sqlscan");
        let install_path = PathBuf::from("/usr/share/sqlscan");

        create_executable_wrapper(&exec_path, &install_path, "sqlscan").unwrap();

        assert!(exec_path.exists());
        let content = fs::read_to_string(&exec_path).unwrap();
        assert!(content.contains("#!/bin/bash"));
        assert!(content.contains("/usr/share/sqlscan/sqlscan"));
    }

    #[test]
    fn test_uninstall_nonexistent_paths() {
        let temp_dir = TempDir::new().unwrap();
        let fake_install = temp_dir.path().join("fake_install");
        let fake_exec = temp_dir.path().join("fake_exec");

        // Should not panic when paths don't exist
        uninstall(&fake_install, &fake_exec);
    }

    #[test]
    fn test_uninstall_existing_paths() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = temp_dir.path().join("install");
        let exec_file = temp_dir.path().join("exec");

        // Create test files
        fs::create_dir_all(&install_dir).unwrap();
        fs::write(&exec_file, "#!/bin/bash\necho test").unwrap();

        assert!(install_dir.exists());
        assert!(exec_file.exists());

        uninstall(&install_dir, &exec_file);

        assert!(!install_dir.exists());
        assert!(!exec_file.exists());
    }

    #[test]
    fn test_metadata_display() {
        // This test just ensures the metadata function doesn't panic
        show_metadata();
    }
}
