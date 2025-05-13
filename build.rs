use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

fn main() {
    const SHADER_PATH: &str = "./shaders";
    println!("cargo::rerun-if-changed={SHADER_PATH}");

    let files = [Path::new("ray_tracing.slang")];

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(SHADER_PATH);
    if !out_dir.exists() {
        _ = std::fs::create_dir(&out_dir);
    }

    let mut processes = vec![];
    for file in files {
        let file_path = Path::new(SHADER_PATH).join(file);

        assert!(file_path.extension().unwrap() == "slang");

        let out_filepath = out_dir.join(file.with_extension("spv"));

        let process = std::process::Command::new("slangc")
            .arg(&file_path)
            .arg("-o")
            .arg(out_filepath)
            .args(["-warnings-as-errors", "all", "-Xspirv-opt", "-o 3"])
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        processes.push((file_path, process));
    }

    for (file, process) in processes {
        let output = process.wait_with_output().unwrap();
        if !output.status.success() {
            panic!(
                "{}\n{}",
                file.to_string_lossy(),
                String::from_utf8_lossy(&output.stderr)
            )
        }
    }
}
