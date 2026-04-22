# fs-share

A fast, simple cross platform CLI tool to transfer files.

`fs-share` allows you to send files between devices with real-time progress. It supports automatic peer discovery as well as manual connections when needed.

## Installation

Make sure you have Rust installed. Then run:

```bash
cargo install fs-share
```

## Usage

`fs-share` works in two modes:

`send` → Send files then receive files
`receive` → Receive files then send files

One device must run send, and the other must run receive.


### Send files from `send` mode

```bash
fs-share send <file1> <file2> <file3> ...
```

### Send files from `receive` mode

```bash
fs-share receive <file1> <file2> <file3> ...
```


## Manual Connection (Skip Auto Discovery)

### Send files from `send` mode

```bash
fs-share send --receiver-addr <ip>:<port> <file1> <file2> ...
```

### Send files from `receive` mode:

```bash
fs-share receive --disable-broadcast --tcp-listener-addr <ip>:<port> <file1> <file2> ...
```

## Options

### Sender

```text
Arguments:
  [ARGS]...  Files to send to the Receiver

Options:
  -r, --receiver-addr <RECEIVER_ADDR>    Manually specify receiver address (skip auto-discovery)
  -d, --download-dir <DOWNLOAD_DIR>      Directory where received files will be saved
      --disable-progress                 Disable progress bar output
      --broadcast-port <BROADCAST_PORT>  UDP broadcast port for discovering receivers [default: 7755]
  -h, --help                             Print help
```

### Receiver

```text
Arguments:
  [ARGS]...  Files to send to the Sender

Options:
  -t, --tcp-listener-addr <TCP_LISTENER_ADDR>  TCP listener address (IP:PORT) for incoming connections
  -d, --download-dir <DOWNLOAD_DIR>            Directory to save received files
      --disable-broadcast                      Disable broadcasting presence (no auto-discovery)
      --disable-progress                       Disable progress bar output
  -b, --broadcast-port <BROADCAST_PORT>        UDP broadcast port used for discovery [default: 7755]
  -h, --help                                   Print help
```


## Contributing

Pull requests are welcome!
Feel free to open issues for bugs or feature requests.

