{ pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
    bartender
    stripe-cli
    sshfs
    utm
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
