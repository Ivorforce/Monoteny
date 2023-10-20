use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn write_file_safe(base_path: &Path, sub_path: &str, content: &str) -> PathBuf {
    let file_path = base_path.join(sub_path);

    if !file_path.starts_with(base_path) {
        panic!("Tried to write a file in unexpected directory: {}", file_path.as_os_str().to_string_lossy());
    }

    let mut f = File::create(file_path.clone()).expect("Unable to create file");
    let f: &mut (dyn Write) = &mut f;
    write!(f, "{}", content).expect("Error writing file");

    file_path
}