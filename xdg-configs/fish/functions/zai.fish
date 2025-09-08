function zai --description 'Claude via z.ai'
    set -lx ANTHROPIC_BASE_URL https://api.z.ai/api/anthropic
    set -lx ANTHROPIC_AUTH_TOKEN $Z_API_KEY
    set -lx ANTHROPIC_MODEL glm-4.5
    set -lx ANTHROPIC_SMALL_FAST_MODEL glm-4.5-air
    $HOME/.claude/local/claude $argv
end
