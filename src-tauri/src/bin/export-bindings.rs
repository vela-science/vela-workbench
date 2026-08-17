use std::{fs, path::PathBuf};

fn main() {
    let output =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/contracts/generated/ipc.ts");
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("create generated contract directory");
    }
    fs::write(
        &output,
        vela_workbench_lib::contracts::typescript_bindings(),
    )
    .expect("write generated TypeScript bindings");
    println!("generated {}", output.display());
}
