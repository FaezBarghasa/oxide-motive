use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

fn run_virtual_mcu() -> std::process::Child {
    Command::new("cargo")
        .args(&["run", "--package", "virtual-mcu"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start virtual-mcu")
}

fn run_virtual_host() -> std::process::Child {
    Command::new("cargo")
        .args(&["run", "--package", "virtual-host"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start virtual-host")
}

#[test]
fn test_full_simulation() {
    let mut mcu_process = run_virtual_mcu();
    thread::sleep(Duration::from_secs(2)); // Give MCU time to start
    let mut host_process = run_virtual_host();

    let host_output = host_process.wait_with_output().expect("Failed to wait on host");
    mcu_process.kill().expect("Failed to kill MCU process");

    let stdout = String::from_utf8_lossy(&host_output.stdout);
    println!("Host output:\n{}", stdout);

    assert!(stdout.contains("Host received: SyncResponse"));
    assert!(stdout.contains("Host received: Ack"));
}
