{
  description = "Luan system flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nix-darwin.url = "github:LnL7/nix-darwin";
    nix-darwin.inputs.nixpkgs.follows = "nixpkgs";
    home-manager.url = "github:nix-community/home-manager";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";
    mac-app-util.url = "github:hraban/mac-app-util";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      nix-darwin,
      home-manager,
      mac-app-util,
    }:
    let
      mkDarwinConfiguration =
        { hostname, username }:
        nix-darwin.lib.darwinSystem {
          modules = [
            {
              nixpkgs.config.allowUnfree = true;
              services.nix-daemon.enable = true;
              nix.settings.experimental-features = "nix-command flakes";
              programs.zsh.enable = true; # default shell on catalina
              system.configurationRevision = self.rev or self.dirtyRev or null;
            }
            ./darwin
            ./hosts/${hostname}/configuration.nix
            home-manager.darwinModules.home-manager
            {
              home-manager.sharedModules = [
                mac-app-util.homeManagerModules.default
              ];

              home-manager.useGlobalPkgs = true;
              home-manager.useUserPackages = true;
              home-manager.verbose = true;

              home-manager.users.${username} = {
                imports = [
                  ./home
                  ./hosts/${hostname}/home.nix
                ];
              };
            }
          ];
        };
    in
    {
      # $ darwin-rebuild build --flake .#blaze
      darwinConfigurations."blaze" = mkDarwinConfiguration {
        hostname = "blaze";
        username = "luan";
      };

      # $ darwin-rebuild build --flake .#flux
      darwinConfigurations."flux" = mkDarwinConfiguration {
        hostname = "flux";
        username = "luan.santos";
      };
    };
}
