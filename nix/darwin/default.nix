{
  pkgs,
  ...
}:
{

  imports = [
    ./systemDefaults.nix
  ];

  environment.systemPackages = with pkgs; [
    awscli
    bat
    cmake
    coreutils
    delta
    deno
    direnv
    docker
    fd
    ffmpeg
    fzf
    gh
    git
    git-lfs
    go
    google-chrome
    grc
    helix
    hexedit
    highlight
    htop
    imagemagick
    jq
    mkalias
    neovim
    nixd
    nixfmt-rfc-style
    ollama
    python3
    ripgrep
    rsync
    rustup
    shellcheck
    sqlite
    stow
    stylua
    tableplus
    tmux
    tree-sitter
    turso-cli
    vale
    vcpkg
    vim
    vsce
    wget
    zig
    zoxide
    zsh
    zsh-completions

    # GUI Apps
    discord
    iterm2
    keka
    obsidian
    raycast
    signal-desktop
    slack
    spotify
    vscode
    zoom-us
  ];

  fonts.packages = with pkgs; [
    font-awesome
    monaspace
    nerd-fonts._0xproto
    nerd-fonts.hack
    nerd-fonts.iosevka
    nerd-fonts.iosevka-term
    nerd-fonts.iosevka-term-slab
    nerd-fonts.monaspace
    nerd-fonts.monofur
    nerd-fonts.victor-mono
    nerd-fonts.zed-mono
  ];

  homebrew = {
    enable = true;
    onActivation = {
      upgrade = true;
      autoUpdate = true;
      cleanup = "zap";
    };

    brews = [
      "mas"
      "zsh"
    ];

    masApps = {
      "Amphetamine" = 937984704;
      "ColorSlurp" = 1287239339;
      "NextDNS" = 1464122853;
    };

    casks = [
      "1password-cli"
      "1password@nightly"
      "betterdisplay"
      "chatgpt"
      "cursor"
      "ghostty"
      "github"
      "home-assistant"
      "legcord"
      "setapp"
      "steam"
      "ticktick"
      "tuple"
      "whatsapp"
      "xquartz"
      "zen-browser"
      "zed@preview"

      "font-zed-mono"
      "font-zed-sans"
      "font-zed-mono-nerd-font"
    ];
  };

  # Used for backwards compatibility, please read the changelog before changing.
  # $ darwin-rebuild changelog
  system.stateVersion = 5;

  # The platform the configuration will be used on.
  nixpkgs.hostPlatform = "aarch64-darwin";
}
