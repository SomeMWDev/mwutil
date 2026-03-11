# mwutil-rs

This is a rewrite of mwutil in rust. It aims to heavily improve the performance and stability of the tool and replaces
the python version.

## Installation

0. Set up cargo (no, not the MediaWiki extension) if you haven't yet
1. Clone this repository
2. `cd rs`
3. `cargo install --path . --profile release`

Try running `mwutil` in a new shell.


If the command wasn't found, maybe you didn't add Cargo to the PATH?
```shell
export PATH="$PATH:$HOME/.cargo/bin"
```
