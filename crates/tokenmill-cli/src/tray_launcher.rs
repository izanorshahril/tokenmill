#![cfg_attr(windows, windows_subsystem = "windows")]

use std::process::Command;

fn main() {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let executable = std::env::current_exe()?.with_file_name(if cfg!(windows) {
            "tokenmill.exe"
        } else {
            "tokenmill"
        });
        let mut command = Command::new(executable);
        command.arg("tray");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        if !command.status()?.success() {
            return Err("Tokenmill tray did not start".into());
        }
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("{error}");
        #[cfg(windows)]
        if let Some(root) = std::env::var_os("SystemRoot") {
            use std::os::windows::process::CommandExt;
            let _ = Command::new(std::path::PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe"))
                .args(["-NoProfile", "-NonInteractive", "-Command", "Add-Type -AssemblyName System.Windows.Forms; [void][System.Windows.Forms.MessageBox]::Show('Tokenmill could not start. Run tokenmill.exe tray in a terminal for details.', 'Tokenmill startup failed')"])
                .creation_flags(0x08000000)
                .status();
        }
        std::process::exit(1);
    }
}
