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
    fd
    ffmpeg
    fzf
    gh
    git
    git-lfs
    go
    grc
    helix
    hexedit
    highlight
    htop
    jq
    mkalias
    neovim
    nixd
    nixfmt-rfc-style
    ripgrep
    rsync
    rustup
    shellcheck
    stow
    stylua
    tmux
    tree-sitter
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
    kitty
    obsidian
    raycast
    signal-desktop
    slack
    spotify
    vscode
    zoom-us
  ];

  fonts.packages = with pkgs; [
    nerdfonts
    monaspace
    font-awesome
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
      "github"
      "home-assistant"
      "setapp"
      "steam"
      "ticktick"
      "tuple"
      "warp"
      "whatsapp"
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
