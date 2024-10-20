{
  description = "Luan system flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nix-darwin.url = "github:LnL7/nix-darwin";
    nix-darwin.inputs.nixpkgs.follows = "nixpkgs";
    nix-homebrew.url = "github:zhaofengli-wip/nix-homebrew";
    home-manager.url = "github:nix-community/home-manager";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    inputs@{
      self,
      nix-darwin,
      home-manager,
      nixpkgs,
      nix-homebrew,
    }:
    let
      configuration =
        {
          extraPackages ? (pkgs: [ ]),
          extraFonts ? (pkgs: [ ]),
          extraBrews ? [ ],
          extraCasks ? [ ],
        }:
        {
          pkgs,
          config,
          ...
        }:
        {
          nixpkgs.config.allowUnfree = true;

          environment.systemPackages =
            with pkgs;
            [
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
            ]
            ++ extraPackages pkgs;

          fonts.packages =
            with pkgs;
            [
              nerdfonts
              monaspace
              font-awesome
            ]
            ++ extraFonts pkgs;

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
            ] ++ extraBrews;

            masApps = {
              "Amphetamine" = 937984704;
              "ColorSlurp" = 1287239339;
              "NextDNS" = 1464122853;
            };

            casks = [
              "1password-cli"
              "1password@nightly"
              "chatgpt"
              "github"
              "home-assistant"
              "setapp"
              "steam"
              "ticktick"
              "tuple"
              "warp"
              "whatsapp"
              "zed@preview"
            ] ++ extraCasks;
          };

          # Auto upgrade nix package and the daemon service.
          services.nix-daemon.enable = true;
          # nix.package = pkgs.nix;

          # Necessary for using flakes on this system.
          nix.settings.experimental-features = "nix-command flakes";

          # Create /etc/zshrc that loads the nix-darwin environment.
          programs.zsh.enable = true; # default shell on catalina
          # programs.fish.enable = true;

          # Set Git commit hash for darwin-version.
          system.configurationRevision = self.rev or self.dirtyRev or null;

          # Used for backwards compatibility, please read the changelog before changing.
          # $ darwin-rebuild changelog
          system.stateVersion = 5;

          # The platform the configuration will be used on.
          nixpkgs.hostPlatform = "aarch64-darwin";

          system.activationScripts.applications.text =
            let
              env = pkgs.buildEnv {
                name = "system-applications";
                paths = config.environment.systemPackages;
                pathsToLink = "/Applications";
              };
            in
            pkgs.lib.mkForce ''
              # Set up applications.
              echo "setting up /Applications..." >&2
              rm -rf /Applications/Nix\ Apps
              mkdir -p /Applications/Nix\ Apps
              find ${env}/Applications -maxdepth 1 -type l -exec readlink '{}' + |
              while read src; do
                app_name=$(basename "$src")
                echo "copying $src" >&2
                ${pkgs.mkalias}/bin/mkalias "$src" "/Applications/Nix Apps/$app_name"
              done
            '';
        };
    in
    {
      # $ darwin-rebuild build --flake .#mbpro
      darwinConfigurations."mbpro" = nix-darwin.lib.darwinSystem {
        modules = [
          (configuration {
            extraPackages =
              pkgs: with pkgs; [
                stripe-cli
                sshfs
                utm
                xquartz
              ];
            extraCasks = [
              "arc"
              "blender"
              "dropbox"
              "firefox"
              "firefox@nightly"
              "macfuse"
              "parsec"
              "plex"
              "pop"
              "vlc"
            ];
          })
          home-manager.darwinModules.home-manager
          {
            home-manager.useGlobalPkgs = true;
            home-manager.useUserPackages = true;
            home-manager.users."luan" = import ./home.nix;
            users.users."luan".home = "/Users/luan";
          }
        ];
      };

      darwinConfigurations."mbpro2" = nix-darwin.lib.darwinSystem {
        modules = [
          (configuration { })
          nix-homebrew.darwinModules.nix-homebrew
          {
            nix-homebrew = {
              enable = true;
              enableRosetta = true;
              user = "luan.santos";
              autoMigrate = true;
            };
          }
        ];
      };
    };
}
