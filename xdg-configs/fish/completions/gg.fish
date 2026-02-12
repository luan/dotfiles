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


function __gg_grove_names --description "Get grove names from git worktree list"
    set -l git_dir (git rev-parse --git-common-dir 2>/dev/null)
    test -z "$git_dir"; and return

    set -l git_dir_resolved (path resolve $git_dir)

    for wt_line in (git worktree list 2>/dev/null)
        string match -q "*(bare)" $wt_line; and continue

        set -l wt_path (string split -f1 ' ' $wt_line)
        set -l resolved (path resolve $wt_path)

        if test (dirname $resolved) = "$git_dir_resolved"
            basename $resolved
        end
    end
end

function __gg_detached_groves --description "Get only detached grove names"
    set -l git_dir (git rev-parse --git-common-dir 2>/dev/null)
    test -z "$git_dir"; and return

    set -l git_dir_resolved (path resolve $git_dir)

    for wt_line in (git worktree list 2>/dev/null)
        string match -q "*(bare)" $wt_line; and continue

        set -l wt_path (string split -f1 ' ' $wt_line)
        set -l resolved (path resolve $wt_path)

        if test (dirname $resolved) = "$git_dir_resolved"
            set -l branch (git -C $resolved branch --show-current 2>/dev/null)
            if test -z "$branch"
                basename $resolved
            end
        end
    end
end

function __gg_attached_groves --description "Get only attached grove names"
    set -l git_dir (git rev-parse --git-common-dir 2>/dev/null)
    test -z "$git_dir"; and return

    set -l git_dir_resolved (path resolve $git_dir)

    for wt_line in (git worktree list 2>/dev/null)
        string match -q "*(bare)" $wt_line; and continue

        set -l wt_path (string split -f1 ' ' $wt_line)
        set -l resolved (path resolve $wt_path)

        if test (dirname $resolved) = "$git_dir_resolved"
            set -l branch (git -C $resolved branch --show-current 2>/dev/null)
            if test -n "$branch"
                basename $resolved
            end
        end
    end
end

# Helper function to get git branch names
function __gg_git_branches --description "Get git branch names for attach command"
    git branch --format='%(refname:short)' 2>/dev/null
end