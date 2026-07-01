
use serde::{Serialize, de::DeserializeOwned};
use std::io::{Read, Write};
use std::os::unix::net::{UnixStream, UnixListener};
use std::thread;
use std::time::Duration;
use postcard::{to_stdvec, from_bytes};

pub trait IpcChannel<T> {
    fn send(&self, msg: &T) -> Result<(), postcard::Error>;
    fn recv(&self) -> Result<T, postcard::Error>;
}

// --- Local Socket IPC for Development ---
pub struct LocalSocketIpc {
    stream: UnixStream,
}

impl<T: Serialize + DeserializeOwned> IpcChannel<T> for LocalSocketIpc {
    fn send(&self, msg: &T) -> Result<(), postcard::Error> {
        let mut buf = to_stdvec(msg)?;
        let len = buf.len() as u32;
        self.stream.write_all(&len.to_be_bytes())?;
        self.stream.write_all(&buf)?;
        Ok(())
    }

    fn recv(&self) -> Result<T, postcard::Error> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf)?;

        from_bytes(&buf)
    }
}

impl LocalSocketIpc {
    pub fn new(path: &str) -> (Self, Self) {
        let listener = UnixListener::bind(path).unwrap();
        let stream1 = UnixStream::connect(path).unwrap();
        let (stream2, _) = listener.accept().unwrap();
        (Self { stream: stream1 }, Self { stream: stream2 })
    }
}


// --- Redox OS IPC ---
#[cfg(target_os = "redox")]
pub struct RedoxIpc {
    // Redox-specific file handle
    fd: usize,
}

#[cfg(target_os = "redox")]
impl<T: Serialize + DeserializeOwned> IpcChannel<T> for RedoxIpc {
    fn send(&self, msg: &T) -> Result<(), postcard::Error> {
        let mut buf = to_stdvec(msg)?;
        // Redox syscall to write to the IPC file descriptor
        syscall::call::write(self.fd, &buf).map_err(|_| postcard::Error::SerializeBufferFull)?;
        Ok(())
    }

    fn recv(&self) -> Result<T, postcard::Error> {
        let mut buf = vec![0u8; 4096]; // Max IPC message size
        let len = syscall::call::read(self.fd, &mut buf).map_err(|_| postcard::Error::DeserializeUnexpectedEnd)?;
        from_bytes(&buf[..len])
    }
}

#[cfg(target_os = "redox")]
impl RedoxIpc {
    pub fn new(path: &str) -> Self {
        let fd = syscall::call::open(path, syscall::O_RDWR).unwrap();
        Self { fd }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestMessage {
        id: u32,
        data: String,
    }

    #[test]
    fn test_local_socket_ipc() {
        let path = "/tmp/oxide-ipc-test.sock";
        let _ = std::fs::remove_file(path);
        let (client, server) = LocalSocketIpc::new(path);

        let msg_to_send = TestMessage {
            id: 123,
            data: "Hello, IPC!".to_string(),
        };

        let server_thread = thread::spawn(move || {
            let received_msg: TestMessage = server.recv().unwrap();
            assert_eq!(received_msg, msg_to_send);
        });

        let client_thread = thread::spawn(move || {
            client.send(&msg_to_send).unwrap();
        });

        client_thread.join().unwrap();
        server_thread.join().unwrap();
        std::fs::remove_file(path).unwrap();
    }
}
