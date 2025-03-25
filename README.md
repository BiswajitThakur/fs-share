# FS-Share

[![Crates.io](https://img.shields.io/crates/v/fs-share.svg)](https://crates.io/crates/fs-share)
[![downloads](https://img.shields.io/crates/d/fs-share.svg)](https://crates.io/crates/fs-share)

A cross-platform file-sharing CLI application written in Rust. It enables efficient file transfers between devices using TCP and UDP protocols, with real-time progress tracking for each transfer.

# Under Development

## Installation

```
cargo install fs-share
```

or

```bash
git clone https://github.com/BiswajitThakur/fs-share.git
cd fs-share/
cargo build --release
sudo mv ./target/release/fs-share /usr/bin/
fs-share --version
```

## Usage

### Modes

- **send**: Sends files to another user.
- **receive**: Receives files and allows sending files back.

## Examples

### Send Files

```
fs-share send file1.mkv file2.mp4 ...
```

### Receive and Send Files

```
fs-share receive file1.mkv file2.mp4 ...
```

In `receive` mode, you can also send files by specifying them after the command, just like in send mode.

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

# License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
