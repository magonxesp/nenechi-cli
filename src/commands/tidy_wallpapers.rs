use log::info;

pub fn tidy_wallpapers(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("ordenando wallpapers en: {}", path);
    Ok(())
}
