use std::path::{Path, PathBuf};

pub fn path_to_string(path: &Path) -> String {
    match dirs::home_dir() {
        None => path.to_string_lossy().to_string(),
        Some(home_dir) => match path.strip_prefix(&home_dir) {
            Ok(stripped_path) => {
                let mut display_path = PathBuf::from("~");
                display_path.push(stripped_path);
                display_path.to_string_lossy().to_string()
            }
            _ => path.to_string_lossy().to_string(),
        },
    }
}
