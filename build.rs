fn main() -> std::io::Result<()> {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut winres = winresource::WindowsResource::new();
        winres.set_icon("build-aux/win32-installer/cartero.ico");
        winres.compile()?;
    }
    Ok(())
}
