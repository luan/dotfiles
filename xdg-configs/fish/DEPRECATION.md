# Fish shell — deprecated

Replaced by zsh on 2026-04-22. See `xdg-configs/zsh/`.

Kept around as a safety net. Delete when confident zsh setup is stable.

## To fully remove

1. Verify secrets migrated: `test -f ~/.zsh_secrets && echo OK`
2. Delete fish state: `rm -rf ~/.config/fish ~/.local/share/fish`
3. Delete this dir: `rm -rf xdg-configs/fish`
4. If fish was brew-installed: `brew uninstall fish`
5. Remove the two fish-specific lines from `.gitignore` (`xdg-configs/fish/fish_variables`, `xdg-configs/fish/conf.d/bcny.fish`)
6. `just link` to refresh symlinks
7. Commit.
