if test -r "/Applications/OrbStack.app/Contents/Resources/completions/fish/docker.fish"
    source "/Applications/OrbStack.app/Contents/Resources/completions/fish/docker.fish"
else if command -sq docker
    docker completion fish | source
end
