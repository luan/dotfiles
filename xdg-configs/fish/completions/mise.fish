if command -sq mise
    complete --erase --command mise
    mise completion fish | source
end
