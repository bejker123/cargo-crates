# Installation
```sh
  cargo install cargo-crates
```
# Usage:
cargo ls-crates [OPTIONS]
OPTIONS:
        -h --help print help
        -v print versions
        -d print descriptions
Examples:
cargo ls-crates -v - print package names and versions
cargo ls-crates -d - print package names and descriptions
cargo ls-crates -vd - print package names, descriptions and versions
cargo ls-crates -dv - print package names, descriptions and versions
Note:
Invalid arguments will be ignored.
