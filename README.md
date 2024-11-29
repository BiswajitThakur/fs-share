# FS-Share

A cross-platform file-sharing CLI application written in Rust. It enables efficient file transfers between devices using TCP and UDP protocols, with real-time progress tracking for each transfer.

# Under Development

## Features

- **Cross-platform support**: Works on Linux, macOS and Windows.
- **Two-way file sharing**: Both **Send** and **Receive** modes allow file transfers.
- **Efficient transfer**: Utilizes TCP for file transfers and UDP for peer discovery.
- **Real-time progress bar**: Tracks transfer status with detailed metrics.
- **Small binary size**: Less then 2 MB.

## Installation

```bash
git clone https://github.com/BiswajitThakur/fs-share.git
cd fs-share/
cargo build --release
sudo mv ./target/release/fs-share /usr/bin/
fs-share --version
```

## Usage

### Help

```
Usage: fs-share [OPTIONS] <MODE> [ARGS]...

Arguments:
  <MODE>     [possible values: send, receive]
  [ARGS]...  Args

Options:
      --name <NAME>          Name [default: Unknown]
      --password <PASSWORD>  password [default: password]
      --port <PORT>          port [default: 34254]
      --timeout <TIMEOUT>    Timeout [default: 60]
  -h, --help                 Print help
  -V, --version              Print version
```

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

This project is licensed under the MIT License. See the LICENSE file for details.
