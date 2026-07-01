
use std::process::Command;

#[test]
fn test_mcu_compiles() {
    let output = Command::new("cargo")
        .args(&["check", "--target", "thumbv7em-none-eabihf", "-p", "oxide-firmware"])
        .output()
        .expect("Failed to execute cargo check for MCU.");

    assert!(output.status.success(), "MCU compilation failed: {:?}", output);
}

#[test]
fn test_host_compiles() {
    let output = Command::new("cargo")
        .args(&["check", "--target", "x86_64-unknown-linux-gnu", "-p", "oxide-host"])
        .output()
        .expect("Failed to execute cargo check for host.");

    assert!(output.status.success(), "Host compilation failed: {:?}", output);
}
