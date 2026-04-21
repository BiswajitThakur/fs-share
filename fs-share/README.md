# fs-share

Fast, simple file CLI based file transfer app.

`fs-share` lets you send files between devices on the same network without configuration. It automatically discovers receivers or lets you connect manually.


## Features

* Fast file transfer over TCP
* Automatic peer discovery (UDP broadcast)
* Send multiple files
* Works across Linux, macOS, Windows, and Android (Termux)


## Installation

```bash
cargo install fs-share
```

## Usage

### 1️⃣ Start Receiver

```bash
fs-share receive
```

Optional:

```bash
fs-share receive --download-dir ./downloads
```

---

### 2️⃣ Send Files

```bash
fs-share send file.txt
```

Send multiple files:

```bash
fs-share send file1.txt file2.jpg
```

---

## 🔗 Manual Connection (Skip Auto Discovery)

### Receiver:

```bash
fs-share receive --disable-broadcast --tcp-listener-addr 0.0.0.0:8080
```

### Sender:

```bash
fs-share send --receiver-addr 192.168.1.10:8080 file.txt
```

---

## ⚙️ Options

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

---

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


## How It Works

1. Receiver starts and optionally broadcasts its presence
2. Sender discovers receiver via UDP broadcast
3. TCP connection is established
4. Files are transferred


## 🧪 Example

```bash
# Terminal 1 (Receiver)
fs-share receive

# Terminal 2 (Sender)
fs-share send hello.txt
```

---

## 📱 Termux (Android)

Download Android binary:

```bash
chmod +x fs-share
./fs-share receive
```

---

## 🛠️ Build from Source

```bash
git clone https://github.com/your-username/fs-share
cd fs-share
cargo build --release
```

---

## 🤝 Contributing

Pull requests are welcome!
Feel free to open issues for bugs or feature requests.

