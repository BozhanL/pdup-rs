# pdup-rs

Rust programme for uploading to Pixeldrain from the terminal

## Installation

Either download the latest release from the [releases page](https://github.com/BozhanL/pdup-rs/releases), [Action page](https://github.com/BozhanL/pdup-rs/actions/workflows/cicd.yml?query=branch%3Amain+is%3Asuccess) or build from source with:

```bash
git clone https://github.com/BozhanL/pdup-rs.git
cd pdup-rs
cargo install --path .
```

## Usage

```text
Usage: pdup-rs [OPTIONS] --api-key <API_KEY> <FILES>...

Arguments:
  <FILES>...

Options:
  -a, --api-key <API_KEY>
  -w, --workers <WORKERS>  [default: 4]
  -h, --help               Print help
```
