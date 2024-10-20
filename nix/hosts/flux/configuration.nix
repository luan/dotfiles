{ pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
    stripe-cli
    sshfs
    utm
    xquartz
  ];

  homebrew.brews = [
    "xcodes"
  ];

  users.users."luan.santos" = {
    name = "luan.santos";
    home = "/Users/luan.santos";
  };
}
