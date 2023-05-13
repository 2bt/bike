fn main() {
    // generate level
    println!("cargo:rerun-if-changed=assets/level1.tmj");
    let status = std::process::Command::new("python")
        .arg("assets/convert-level.py")
        .arg("assets/level1.tmj")
        .status()
        .expect("failed to execute process");
    assert!(status.success());
}
