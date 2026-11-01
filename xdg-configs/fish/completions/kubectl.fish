if test -r "/Applications/OrbStack.app/Contents/Resources/completions/fish/kubectl.fish"
    source "/Applications/OrbStack.app/Contents/Resources/completions/fish/kubectl.fish"
else if command -sq kubectl
    kubectl completion fish | source
end
