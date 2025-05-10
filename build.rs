use std::path::PathBuf;

fn main() {
    const SHADER_PATH: &str = "./shaders";
    println!("cargo::rerun-if-changed={SHADER_PATH}");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(SHADER_PATH);
    if !out_dir.exists() {
        _ = std::fs::create_dir(&out_dir);
    }

    let mut processes = vec![];
    for entry in std::fs::read_dir(SHADER_PATH).unwrap() {
        let entry = entry.unwrap();

        assert!(entry.file_type().unwrap().is_file());

        let path = entry.path();
        assert!(path.extension().unwrap() == "slang");

        let out_filepath = out_dir.join(PathBuf::from(entry.file_name()).with_extension("spv"));

        let process = std::process::Command::new("slangc")
            .arg(path)
            .arg("-o")
            .arg(out_filepath)
            .args(["-fvk-use-entrypoint-name", "-emit-spirv-directly"])
            .spawn()
            .unwrap();
        processes.push(process);
    }

    for process in processes {
        let output = process.wait_with_output().unwrap();
        if !output.status.success() {
            panic!("{}", String::from_utf8_lossy(&output.stderr))
        }
    }
}
