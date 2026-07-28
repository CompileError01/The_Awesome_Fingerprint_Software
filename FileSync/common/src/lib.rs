// Shared types for FileSync: the wire protocol between server and client.

use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// The only two things that ever cross: a full file, or a delete instruction.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    File { path: String, data: Vec<u8> },
    Delete { path: String },
}

// Writes one message as a 4-byte length prefix followed by its bincode bytes.
// The length prefix is what lets the reader know where one message ends and the next begins.
pub async fn send_message<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    msg: &Message,
) -> std::io::Result<()> {
    let bytes = bincode::serialize(msg).map_err(to_io_err)?;
    writer.write_u32(bytes.len() as u32).await?;
    writer.write_all(&bytes).await?;
    Ok(())
}

// Reads exactly one message written by send_message.
pub async fn read_message<R: AsyncReadExt + Unpin>(reader: &mut R) -> std::io::Result<Message> {
    let len = reader.read_u32().await?;
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;
    bincode::deserialize(&buf).map_err(to_io_err)
}

fn to_io_err<E: std::fmt::Display>(e: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

// Turns an absolute path under root into a portable relative path string.
// Returns None if path isn't actually inside root.
pub fn to_relative(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    Some(rel.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn relative_path_uses_forward_slashes() {
        let root = Path::new("C:/sync");
        let path = Path::new("C:/sync/sub/file.txt");
        assert_eq!(to_relative(root, path).unwrap(), "sub/file.txt");
    }

    #[test]
    fn relative_path_none_when_outside_root() {
        let root = Path::new("/sync");
        let path = Path::new("/other/file.txt");
        assert!(to_relative(root, path).is_none());
    }

    #[test]
    fn relative_path_handles_root_level_file() {
        let root = Path::new("/sync");
        let path = Path::new("/sync/file.txt");
        assert_eq!(to_relative(root, path).unwrap(), "file.txt");
    }

    #[tokio::test]
    async fn file_message_roundtrips_through_the_wire_format() {
        let msg = Message::File { path: "a/b.txt".into(), data: vec![1, 2, 3] };
        let mut buf = Cursor::new(Vec::new());
        send_message(&mut buf, &msg).await.unwrap();

        let mut reader = Cursor::new(buf.into_inner());
        match read_message(&mut reader).await.unwrap() {
            Message::File { path, data } => {
                assert_eq!(path, "a/b.txt");
                assert_eq!(data, vec![1, 2, 3]);
            }
            _ => panic!("expected a File message"),
        }
    }

    #[tokio::test]
    async fn delete_message_roundtrips_through_the_wire_format() {
        let msg = Message::Delete { path: "old/file.txt".into() };
        let mut buf = Cursor::new(Vec::new());
        send_message(&mut buf, &msg).await.unwrap();

        let mut reader = Cursor::new(buf.into_inner());
        match read_message(&mut reader).await.unwrap() {
            Message::Delete { path } => assert_eq!(path, "old/file.txt"),
            _ => panic!("expected a Delete message"),
        }
    }

    #[tokio::test]
    async fn two_messages_can_be_read_back_in_order() {
        let mut buf = Cursor::new(Vec::new());
        send_message(&mut buf, &Message::Delete { path: "one".into() }).await.unwrap();
        send_message(&mut buf, &Message::Delete { path: "two".into() }).await.unwrap();

        let mut reader = Cursor::new(buf.into_inner());
        let first = read_message(&mut reader).await.unwrap();
        let second = read_message(&mut reader).await.unwrap();
        assert!(matches!(first, Message::Delete { path } if path == "one"));
        assert!(matches!(second, Message::Delete { path } if path == "two"));
    }
}
