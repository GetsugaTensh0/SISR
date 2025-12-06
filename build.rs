fn main() {
    // Set CMAKE_GENERATOR to Ninja for SDL3 build
    println!("cargo:rustc-env=CMAKE_GENERATOR=Ninja");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();

        res.set_icon("assets/icon.ico");

        let version = option_env!("SISR_VERSION")
            .or(option_env!("CARGO_PKG_VERSION"))
            .unwrap_or("0.0.1");

        let version_clean = version.strip_prefix('v').unwrap_or(version);

        let version_parts: Vec<&str> = version_clean
            .split('-')
            .next()
            .unwrap_or(version_clean)
            .split('.')
            .collect();
        let major = version_parts
            .get(0)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let minor = version_parts
            .get(1)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let patch = version_parts
            .get(2)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        res.set_version_info(
            winres::VersionInfo::PRODUCTVERSION,
            (major << 48) | (minor << 32) | (patch << 16),
        );
        res.set_version_info(
            winres::VersionInfo::FILEVERSION,
            (major << 48) | (minor << 32) | (patch << 16),
        );

        res.set("FileVersion", &format!("{}.{}.{}.0", major, minor, patch));
        res.set("ProductVersion", version_clean);
        res.set("ProductName", "SISR");
        res.set("FileDescription", "SISR - Steam Input System Redirector");
        res.set("CompanyName", "Peter Repukat");
        res.set(
            "LegalCopyright",
            "Copyright (C) 2025 Peter Repukat - GPL-3.0",
        );
        res.set("OriginalFilename", "SISR.exe");
        res.set("InternalName", "sisr");

        let manifest = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    version="{}.{}.{}.0"
    publicKeyToken="0000000000000000"
    name="SISR"
    type="win32"
  />
  <description>SISR - Steam Input System Redirector</description>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>
"#,
            major, minor, patch
        );

        res.set_manifest(&manifest);

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }

    println!("cargo:rerun-if-env-changed=SISR_VERSION");
    println!("cargo:rerun-if-changed=assets/icon.ico");
}
