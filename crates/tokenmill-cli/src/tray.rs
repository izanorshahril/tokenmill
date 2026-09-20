pub fn run(args: &[String]) -> Result<(), String> {
    if !args.is_empty() {
        return Err("tray takes no arguments; use TOKENMILL_HOME for isolated settings".into());
    }
    #[cfg(not(windows))]
    return Err("tray is available on Windows; use settings commands on this platform".into());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let dir = crate::settings::directory()?;
        let _instance = match crate::settings::lock(&dir, "tray.lock") {
            Ok(lock) => lock,
            Err(error) => {
                if !error.contains("is busy") {
                    return Err(error);
                }
                std::fs::write(dir.join("show-tray.request"), b"").map_err(|_| error)?;
                println!("Requested existing tray controls window.");
                return Ok(());
            }
        };
        let source = dir.join("tray-ui.cs");
        let gui = dir.join("tokenmill-tray-ui.exe");
        std::fs::write(&source, include_str!("tray_ui.cs"))
            .map_err(|_| "cannot write tray source")?;
        let executable =
            std::env::current_exe().map_err(|_| "cannot locate Tokenmill executable")?;
        let system_root = std::env::var_os("SystemRoot").ok_or("SystemRoot is missing")?;
        let compiler = std::path::PathBuf::from(system_root)
            .join("Microsoft.NET/Framework64/v4.0.30319/csc.exe");
        let build = std::process::Command::new(compiler)
            .args([
                "/nologo",
                "/target:winexe",
                "/r:System.Windows.Forms.dll",
                "/r:System.Drawing.dll",
                "/r:System.Web.Extensions.dll",
            ])
            .arg(format!("/out:{}", gui.display()))
            .arg(&source)
            .creation_flags(0x08000000)
            .output();
        let _ = std::fs::remove_file(source);
        match build {
            Ok(output) if output.status.success() => (),
            Ok(output) => {
                return Err(format!(
                    "tray compiler failed: {}",
                    String::from_utf8_lossy(&output.stdout)
                ));
            }
            Err(_) => return Err("tray requires the installed .NET Framework 4 compiler".into()),
        }
        let result = std::process::Command::new(&gui)
            .arg(executable)
            .env("TOKENMILL_HOME", &dir)
            .creation_flags(0x08000000)
            .status();
        match result {
            Ok(status) if status.success() => Ok(()),
            _ => Err("native tray UI failed to start".into()),
        }
    }
}
