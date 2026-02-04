use crate::simulation::SimulationConfig;
use std::fs;
use std::path::Path;

pub fn save_config(config: &SimulationConfig, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(path, json).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

pub fn load_config(path: &Path) -> Result<SimulationConfig, String> {
    let json = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    serde_json::from_str(&json).map_err(|e| format!("Failed to parse config: {}", e))
}

pub fn save_config_dialog(config: &SimulationConfig) -> Option<Result<(), String>> {
    let file = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_file_name("simulation_config.json")
        .save_file()?;

    Some(save_config(config, &file))
}

pub fn load_config_dialog() -> Option<Result<SimulationConfig, String>> {
    let file = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .pick_file()?;

    Some(load_config(&file))
}
