use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

pub trait ResultTrait<T> {
    fn map_to_string(self) -> Result<T, String>;
    fn map_to_string_mess(self, mess: &str) -> Result<T, String>;
}

impl<T, E> ResultTrait<T> for Result<T, E>
where
    E: Display,
{
    fn map_to_string(self) -> Result<T, String> {
        self.map_err(|err| format!("{err}"))
    }

    fn map_to_string_mess(self, mess: &str) -> Result<T, String> {
        self.map_err(|err| format!("{mess} {err}"))
    }
}

pub fn resolve_path(dir: &Path, path: &str) -> PathBuf {
    // Handle both Unix and Windows style relative paths
    if path.starts_with("./") || path.starts_with(".\\") {
        dir.join(&path[2..]) // Remove "./" or ".\\" and join with dir
    } else {
        PathBuf::from(path) // Use command as-is if it's not relative
    }
}

pub fn extract_code_block(source: &str) -> String {
    if !source.contains("@code") {
        return source.into()
    }

    let mut lines = source.lines();
    // Find the start marker
    while let Some(line) = lines.next() {
        if line.contains("@code") && line.contains("begin") {
            break;
        }
    }
    // Collect lines until the end marker
    let mut code_block = Vec::new();
    for line in lines {
        if line.contains("@code") && line.contains("end") {
            break;
        }
        code_block.push(line);
    }
    if code_block.is_empty() {
        source.to_string()
    } else {
        code_block.join("\n").trim().to_string()
    }
}
