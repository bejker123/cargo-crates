# Installation
#### 1. Installing from crates.io (cargo)
First make sure you have [cargo](https://rustup.rs/) installed.
Than you can the command below.
```sh
  $ cargo install cargo-crates
```
#### 2. Building from source
```sh
    #clone the repo
    $ git clone https://github.com/bejker123/cargo-crates
    $ cd cargo-crates
    #build it
    $ cargo build --release
```

# Usage:
```sh
$ cargo ls-crates [OPTIONS]
# OPTIONS:
#       -h --help print help
#       -v print versions
#       -d print descriptions
# Examples:
    $ cargo ls-crates -v   # print package names and versions
    $ cargo ls-crates -d   # print package names and descriptions
    $ cargo ls-crates -vd  # print package names, descriptions and versions
    $ cargo ls-crates -dv  # print package names, descriptions and versions
# Note: Invalid arguments will be ignored.
```
