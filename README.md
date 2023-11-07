# Amanita

A tool to sync your directory over N machines.
[![Tests](https://github.com/hhamud/Amanita/actions/workflows/ci.yml/badge.svg)](https://github.com/hhamud/Amanita/actions/workflows/ci.yml)

## Installation

```shell
cargo install --git https://github.com/hhamud/amanita.git --bin
```

## Usage

To use this project, you need to build and run it on both the sender and receiver sides. The following command-line arguments are available for running the project:

### Sender
- `--from`: Specifies the source directory from which files will be sent.
- `--to`: Specifies the WebSocket URL in the format 'ws://localhost:PORT/ws' to which files will be sent.


```shell
amanita --from ~/Desktop  --to ws://localhost:4000/ws
```

### Receiver
- make sure that the output directory already exists.
- `--output_dir`: Specifies the directory where received files will be saved.
- `--port`: Specifies the port on which the WebSocket server will listen for incoming connections.


```shell
amanita --port 4000 --output-dir  ~/test
```


