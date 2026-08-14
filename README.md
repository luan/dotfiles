# dotfiles

Run `just setup` on a new macOS machine.

GNU Stow links the managed configuration into the home directory.

The global Rust toolchain uses rolling nightly through Mise.

Cargo uses Kache with non-incremental development and test builds.

Mise exports Kache as the C and C++ compiler wrapper for Fish, Zsh, Bash, and PowerShell.

Run `just rust-upgrade` to update nightly and its managed components.

Run `just mux` to build, install, and sign the Rust `mux` binary.
