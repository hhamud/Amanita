# Amanita

A tool to sync your directory over N machines.

## Installation

```shell
cargo build
```

## Usage

To use this project, you need to build and run it on both the sender and receiver sides. The following command-line arguments are available for running the project:

### Sender
- `--from`: Specifies the source directory from which files will be sent.
- `--to`: Specifies the WebSocket URL in the format 'ws://localhost:PORT/ws' to which files will be sent.


```shell
cargo run --release -- --from ~/Desktop  --to ws://localhost:4000/ws
```

### Receiver
- `--output_dir`: Specifies the directory where received files will be saved.
- `--port`: Specifies the port on which the WebSocket server will listen for incoming connections.


- make sure that the output directory already exists.
```shell
cargo run --release -- --port 4000 --output-dir  ~/test
```


