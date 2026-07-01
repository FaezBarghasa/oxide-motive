use std::process::Command;
use std::thread;
use std::time::Duration;

fn spawn_child(name: &str) -> std::process::Child {
    Command::new("cargo")
        .args(&["run", "--package", name])
        .spawn()
        .expect(&format!("Failed to start {}", name))
}

fn main() {
    let mut children = vec![
        spawn_child("oxide-core"),
        spawn_child("oxide-logger"),
        spawn_child("oxide-telemetry"),
    ];

    loop {
        for child in &mut children {
            match child.try_wait() {
                Ok(Some(status)) => {
                    println!("Child process exited with: {}", status);
                    // Respawn the child
                    // In a real scenario, we'd need to know which child exited
                }
                Ok(None) => {
                    // Child is still running
                }
                Err(e) => {
                    println!("Error attempting to wait for child: {}", e);
                }
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}
