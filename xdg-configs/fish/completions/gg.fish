# Completions for gg - Git grove manager

# Disable file completions for gg
complete -c gg -f

# Global options
complete -c gg -s h -l help -d "Show help message"
complete -c gg -s v -l verbose -d "Enable verbose output"
complete -c gg -s q -l quiet -d "Suppress informational output"

# Subcommands
complete -c gg -n "__fish_use_subcommand" -a add -d "Add a new grove"
complete -c gg -n "__fish_use_subcommand" -a remove -d "Remove a grove"
complete -c gg -n "__fish_use_subcommand" -a rm -d "Remove a grove"
complete -c gg -n "__fish_use_subcommand" -a list -d "List all worktrees"
complete -c gg -n "__fish_use_subcommand" -a ls -d "List all worktrees"
complete -c gg -n "__fish_use_subcommand" -a detach -d "Detach from branch"
complete -c gg -n "__fish_use_subcommand" -a d -d "Detach from branch (alias)"
complete -c gg -n "__fish_use_subcommand" -a go -d "Switch to first available worktree"
complete -c gg -n "__fish_use_subcommand" -a pool -d "Show pool status"
complete -c gg -n "__fish_use_subcommand" -a init -d "Initialize grove configuration"
complete -c gg -n "__fish_use_subcommand" -a help -d "Show help message"

# Options for 'add' subcommand
complete -c gg -n "__fish_seen_subcommand_from add" -l from -x -d "Base commit/branch/tag (default: main)"
complete -c gg -n "__fish_seen_subcommand_from add" -s b -l branch -xa "(__gg_git_branches)" -d "Create and checkout branch (not detached)"
complete -c gg -n "__fish_seen_subcommand_from add" -l sync -d "Sync staged/modified/untracked files"
complete -c gg -n "__fish_seen_subcommand_from add" -l no-hook -d "Skip hook execution"
complete -c gg -n "__fish_seen_subcommand_from add" -s h -l help -d "Show help for add command"

# Options for 'remove/rm' subcommand
complete -c gg -n "__fish_seen_subcommand_from remove rm" -xa "(__gg_detached_groves)" -d "Grove to remove"
complete -c gg -n "__fish_seen_subcommand_from remove rm" -s h -l help -d "Show help for remove command"

# Options for 'list/ls' subcommand
complete -c gg -n "__fish_seen_subcommand_from list ls" -s h -l help -d "Show help for list command"

# Options for 'detach/d' subcommand
complete -c gg -n "__fish_seen_subcommand_from detach d" -xa "(__gg_attached_groves)" -d "Grove to detach from"
complete -c gg -n "__fish_seen_subcommand_from detach d" -s h -l help -d "Show help for detach command"

# Options for 'go' subcommand
complete -c gg -n "__fish_seen_subcommand_from go" -xa "(__gg_git_branches)" -d "Branch to find or checkout"
complete -c gg -n "__fish_seen_subcommand_from go" -s h -l help -d "Show help for go command"

# Options for 'pool' subcommand
complete -c gg -n "__fish_seen_subcommand_from pool" -s h -l help -d "Show help for pool command"

# Options for 'init' subcommand
complete -c gg -n "__fish_seen_subcommand_from init" -s h -l help -d "Show help for init command"


# Helper function to get all grove names
function __gg_grove_names --description "Get grove names from grove_worktrees directory"
    # Look for grove worktrees directory and list subdirectories
    set -l grove_dir "$HOME/grove_worktrees"
    if test -d "$grove_dir"
        for dir in "$grove_dir"/*
            if test -d "$dir"
                basename "$dir"
            end
        end
    end
end

# Helper function to get detached grove names only
function __gg_detached_groves --description "Get only detached grove names for remove/clean"
    # This would need to check grove status to determine which are detached
    # For now, return all grove names - the actual implementation would filter
    __gg_grove_names
end

# Helper function to get attached grove names only
function __gg_attached_groves --description "Get only attached grove names for detach"
    # This would need to check grove status to determine which are attached
    # For now, return all grove names - the actual implementation would filter
    __gg_grove_names
end

# Helper function to get git branch names
function __gg_git_branches --description "Get git branch names for attach command"
    git branch --format='%(refname:short)' 2>/dev/null
end