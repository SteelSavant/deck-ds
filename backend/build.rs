// build.rs

fn main() {
    use std::path::Path;

    // remove test outputs
    let dir = Path::new("test/out");
    if dir.is_dir() {
        let _ = std::fs::remove_dir_all(dir);
    }
    println!("cargo:rerun-if-changed=test/out");
}
