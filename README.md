# dotfiles

Run `just setup` on a new macOS or Linux machine. `just setup-macos` installs
the Homebrew workstation packages; `just setup-linux` links and provisions
the shared tools after you install your distribution's packages.

GNU Stow links individual managed files into real configuration directories.
Machine-local configuration can coexist without being written into this checkout.

The global Rust toolchain uses rolling nightly through Mise.

Other global Mise tools follow the latest stable release; Kache stays on its qualified v2 prerelease.

Mise installs the standalone Vite+ CLI. Vite+ manages Node.js on rolling LTS and npm, pnpm, Yarn, and Bun through its shims. Run `just vite-plus` to provision these defaults; update the CLI with `mise upgrade --global github:voidzero-dev/vite-plus`. Existing project tool pins remain in place.

Cargo uses Kache v2 with executable caching and private adaptive incremental state. Each worktree keeps its own target directory.

Mise exports Kache as the C and C++ compiler wrapper for Fish, Zsh, Bash, and PowerShell.

Run `just rust-upgrade` to update nightly and its managed components.
