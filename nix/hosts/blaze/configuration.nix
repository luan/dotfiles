{ pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
    stripe-cli
    sshfs
    utm
    xquartz
  ];

  homebrew.casks = [
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

  users.users."luan" = {
    name = "luan";
    home = "/Users/luan";
  };
}
