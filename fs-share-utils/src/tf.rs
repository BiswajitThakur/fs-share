//! # File Transfer (tf)
//!
//! Handles sending and receiving files over upgraded streams.
//!
//! ## Protocol
//!
//! Header format:
//! ```text
//! :fff: | name_len(u16) | file_size(u64) | filename | file_bytes...
//! ```

use std::io::Read;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::receiver::App as ReceiverApp;
use crate::sender::App as SenderApp;

const BUFFER_SIZE: usize = 32 * 1024;

fn create_buffer(size: usize) -> Box<[u8]> {
    let v = Box::new_zeroed_slice(size);
    unsafe { v.assume_init() }
}

pub(crate) fn sender_send_file<A: SenderApp + ?Sized>(
    app: &A,
    path: impl AsRef<Path>,
    stream: &mut A::UpgradeStream,
) -> anyhow::Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        anyhow::bail!("Faild to send");
    }
    let file_name = path
        .file_name()
        .with_context(|| format!("Invalid file name: {}", path.display()))?
        .to_string_lossy();
    let name_bytes = file_name.as_bytes();

    let mut file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    let metadata = file.metadata()?;
    let total = metadata.len();

    let pb = app.create_progress_bar(total);

    println!("Sending file: {}, size: {} bytes", path.display(), total);
    stream.write_all(b":fff:")?;
    stream.write_all(&(name_bytes.len() as u16).to_be_bytes())?;
    stream.write_all(&total.to_be_bytes())?;
    stream.write_all(name_bytes)?;
    stream.flush()?;

    let mut buffer = create_buffer(std::cmp::min(total as usize, BUFFER_SIZE));

    let mut i = 0;
    loop {
        let read_count = file.read(&mut buffer)?;
        if read_count == 0 {
            break;
        }
        stream.write_all(&buffer[..read_count])?;

        i += read_count as u64;
        pb.update(i);
    }
    pb.finish();
    Ok(())
}
pub(crate) fn receiver_send_file<A: ReceiverApp + ?Sized>(
    app: &A,
    path: impl AsRef<Path>,
    stream: &mut A::UpgradeStream,
) -> anyhow::Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        anyhow::bail!("Faild to send");
    }
    let file_name = path
        .file_name()
        .with_context(|| format!("Invalid file name: {}", path.display()))?
        .to_string_lossy();
    let name_bytes = file_name.as_bytes();

    let mut file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    let metadata = file.metadata()?;
    let total = metadata.len();

    let pb = app.create_progress_bar(total);

    println!("Sending file: {}, size: {} bytes", path.display(), total);
    stream.write_all(b":fff:")?;
    stream.write_all(&(name_bytes.len() as u16).to_be_bytes())?;
    stream.write_all(&total.to_be_bytes())?;
    stream.write_all(name_bytes)?;
    stream.flush()?;

    let mut buffer = create_buffer(std::cmp::min(total as usize, BUFFER_SIZE));

    let mut i = 0;
    loop {
        let read_count = file.read(&mut buffer)?;
        if read_count == 0 {
            break;
        }
        stream.write_all(&buffer[..read_count])?;

        i += read_count as u64;
        pb.update(i);
    }
    pb.finish();
    Ok(())
}

pub(crate) fn sender_receive_file<A: SenderApp + ?Sized>(
    app: &A,
    stream: &mut A::UpgradeStream,
) -> anyhow::Result<()> {
    // Read name length
    let mut len_buf = [0u8; 2];
    stream.read_exact(&mut len_buf)?;
    let name_len = u16::from_be_bytes(len_buf) as usize;

    // Read file size
    let mut size_buf = [0u8; 8];
    stream.read_exact(&mut size_buf)?;
    let total = u64::from_be_bytes(size_buf);

    // Read filename
    let mut name_buf = vec![0u8; name_len];
    stream.read_exact(&mut name_buf)?;

    let file_name = String::from_utf8(name_buf).context("Invalid UTF-8 in file name")?;

    let mut save_path = PathBuf::from(app.download_dir());
    if !save_path.is_dir() {
        std::fs::create_dir_all(&save_path)
            .with_context(|| format!("Faild to create directoy: {}", save_path.display()))?;
    }
    save_path.push(&file_name);
    if save_path.exists() {
        anyhow::bail!("File already exists: {}", save_path.display());
    }

    let mut file = File::create(&save_path)?;

    let pb = app.create_progress_bar(total);

    println!("Receiving file: {}, size: {} bytes", file_name, total);

    let mut remaining = total;
    let mut buffer = create_buffer(BUFFER_SIZE);
    let mut received = 0;

    while remaining > 0 {
        let to_read = std::cmp::min(buffer.len() as u64, remaining) as usize;

        let n = stream.read(&mut buffer[..to_read])?;
        if n == 0 {
            anyhow::bail!("Unexpected EOF");
        }

        file.write_all(&buffer[..n])?;
        remaining -= n as u64;
        received += n as u64;

        pb.update(received);
    }

    pb.finish();

    Ok(())
}

pub(crate) fn receiver_receive_file<A: ReceiverApp + ?Sized>(
    app: &A,
    stream: &mut A::UpgradeStream,
) -> anyhow::Result<PathBuf> {
    // Read name length
    let mut len_buf = [0u8; 2];
    stream.read_exact(&mut len_buf)?;
    let name_len = u16::from_be_bytes(len_buf) as usize;

    // Read file size
    let mut size_buf = [0u8; 8];
    stream.read_exact(&mut size_buf)?;
    let total = u64::from_be_bytes(size_buf);

    // Read filename
    let mut name_buf = vec![0u8; name_len];
    stream.read_exact(&mut name_buf)?;

    let file_name = String::from_utf8(name_buf).context("Invalid UTF-8 in file name")?;

    let mut save_path = PathBuf::from(app.download_dir());
    if !save_path.is_dir() {
        std::fs::create_dir_all(&save_path)
            .with_context(|| format!("Faild to create directoy: {}", save_path.display()))?;
    }
    save_path.push(&file_name);
    if save_path.exists() {
        anyhow::bail!("File already exists: {}", save_path.display());
    }

    let mut file = File::create(&save_path)?;

    let pb = app.create_progress_bar(total);

    println!("Receiving file: {}, size: {} bytes", file_name, total);

    let mut remaining = total;
    let mut buffer = create_buffer(BUFFER_SIZE);
    let mut received = 0;

    while remaining > 0 {
        let to_read = std::cmp::min(buffer.len() as u64, remaining) as usize;

        let n = stream.read(&mut buffer[..to_read])?;
        if n == 0 {
            anyhow::bail!("Unexpected EOF");
        }

        file.write_all(&buffer[..n])?;
        remaining -= n as u64;
        received += n as u64;

        pb.update(received);
    }

    pb.finish();

    Ok(save_path)
}
