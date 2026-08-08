# koto-calc CLI

koto-calc provides a command-line interface for running `.koto` scripts and an
interactive REPL.

## Installation

Build from source with the [Rust](https://rust-lang.org) toolchain
(see [rustup.sh](https://rustup.sh) for installation instructions):

```bash
cargo build --release
cargo install --path .
```

This provides the `koto_calc` command.

## Usage

```
koto_calc [FLAGS] [script] [<args>...]
```

### Flags

| Flag | Description |
|---|---|
| `-e`, `--eval` | Evaluate the argument as a script string instead of loading from disk |
| `-i`, `--show_instructions` | Show compiled instructions annotated with source lines |
| `-b`, `--show_bytecode` | Show the script's compiled bytecode |
| `-t`, `--tests` | Run the script's tests before running the script |
| `-T`, `--import_tests` | Run the script's tests, plus tests in imported modules |
| `-f`, `--format` | Format the input (from script path or stdin) |
| `-c`, `--config PATH` | Config file to load |
| `-C`, `--print_config` | Print the default config |
| `-v`, `--version` | Print version information |
| `-h`, `--help` | Print help information |

### Arguments

Arguments following the script name are available to the script via
[`os.args`](./core_lib/os.md#args).

## Running Scripts

```bash
# Run a script from a file
koto_calc examples/fibonacci.koto

# Evaluate an expression directly
koto_calc -e "print (1..10).sum()"
# Output: 55

# Run tests in a script
koto_calc -t tests/example.koto

# Pass arguments to the script
koto_calc print_args.koto a b c
```

## Using the REPL

Running `koto_calc` without arguments starts the REPL:

```
> koto_calc
Welcome to Koto

» 1 + 1
➝ 2

» 'hello!'
➝ hello!

» size [1, 2, 3]
➝ 3
```

The `help` command in the REPL provides access to the language guide and core
library reference:

```
» help number
Numbers and Arithmetic
======================
...
```
