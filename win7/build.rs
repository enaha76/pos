fn main() {
    // Embed fonts into the binary for the software renderer, so the app doesn't
    // depend on any system font being present (bare Windows 7 / Wine ship none,
    // which otherwise panics in the renderer's system-font lookup).
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/app.slint", config).expect("Slint compilation failed");
}
