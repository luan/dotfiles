# Helper function to handle both bare and regular repositories
function gg_get_repo_info --description "Get repository information for bare and regular repos"
    # Get git directory (works in both regular and bare repos)
    set -l git_common_dir (git rev-parse --git-common-dir 2>/dev/null)
    if test -z "$git_common_dir"
        return 1
    end
    
    # Resolve Git directory path
    set -l git_dir_resolved (path resolve $git_common_dir)
    
    # Check if we're in a bare repository
    set -l is_bare (git rev-parse --is-bare-repository 2>/dev/null)
    set -l repo_root
    
    if test "$is_bare" = "true"
        # In a bare repository - git directory is the repo root
        set repo_root $git_dir_resolved
    else
        # In a regular repository - get the working tree root
        set repo_root (git rev-parse --show-toplevel 2>/dev/null)
        if test -z "$repo_root"
            return 1
        end
    end
    
    # Output the info (caller can capture this)
    echo "$git_dir_resolved|$repo_root|$is_bare"
end

# Helper function to clean up stale index.lock files
function gg_cleanup_stale_locks --description "Remove stale Git index.lock files"
    set -l verbose $argv[1]
    
    # Get repository info
    set -l repo_info (gg_get_repo_info)
    if test $status -ne 0
        return 0  # Not in a git repo, nothing to clean
    end
    
    set -l parts (string split "|" $repo_info)
    set -l git_dir_resolved $parts[1]
    set -l repo_root $parts[2]
    set -l is_bare $parts[3]
    
    # Check for index.lock in the main git directory
    set -l lock_file "$git_dir_resolved/index.lock"
    
    if test -f "$lock_file"
        # Get file age in seconds (5 minutes = 300 seconds)
        set -l current_time (date +%s)
        set -l file_time (stat -f %m "$lock_file" 2>/dev/null; or stat -c %Y "$lock_file" 2>/dev/null)
        
        if test -n "$file_time"
            set -l age (math "$current_time - $file_time")
            
            if test $age -gt 300  # 5 minutes
                test "$verbose" = true; and echo "[CLEANUP] Removing stale index.lock (age: $age"s")"
                rm -f "$lock_file"
                return 0
            else
                test "$verbose" = true; and echo "[WAIT] index.lock exists but is recent (age: $age"s"), waiting..."
                sleep 1
                return 1  # Signal that we should retry
            end
        else
            # Can't determine age, remove it to be safe
            test "$verbose" = true; and echo "[CLEANUP] Removing index.lock (unknown age)"
            rm -f "$lock_file"
        end
    end
    
    return 0
end

# Helper function to generate next available grove name (wt1, wt2, etc.)
function gg_generate_name --description "Generate next available grove name"
    # Get repository info
    set -l repo_info (gg_get_repo_info)
    if test $status -ne 0
        echo "wt1"  # fallback if not in git repo
        return
    end
    
    set -l parts (string split "|" $repo_info)
    set -l git_dir_resolved $parts[1]
    set -l repo_root $parts[2]
    set -l is_bare $parts[3]
    
    # Determine expected parent directory for grove worktrees
    set -l expected_parent (path resolve "$git_dir_resolved")
    
    # Find highest existing wt number by scanning the expected parent directory
    set -l highest_num 0
    if test -d "$expected_parent"
        for dir in "$expected_parent"/*
            if test -d "$dir"
                set -l grove_name (basename "$dir")
                # Check if it matches wt[number] pattern
                if string match -qr '^wt\d+$' "$grove_name"
                    set -l num (echo "$grove_name" | string replace 'wt' '')
                    if test "$num" -gt "$highest_num"
                        set highest_num "$num"
                    end
                end
            end
        end
    end
    
    # Return next number
    set -l next_num (math "$highest_num + 1")
    echo "wt$next_num"
end

function gg --description "Git Grove - Manage git worktrees as a pool of detached workspaces"
    # Setup cleanup on exit
    function __gg_cleanup_on_exit --on-signal INT --on-signal TERM
        # Clean up any temporary files
        rm -f /tmp/gg_*.log
        
        # Try to clean up any stale locks (suppress output)
        gg_cleanup_stale_locks false >/dev/null 2>&1
    end
    
    # Define help function
    function __gg_help
        echo "Git Grove - Manage git worktrees as a pool of detached workspaces"
        echo "---"
        echo ""
        echo "USAGE:"
        echo "  gg [options]                     - Interactive worktree selection with fzf"
        echo "  gg add [name] [options]          - Create new detached worktree (auto-names wt1, wt2, etc.)"
        echo "  gg remove <name> [options]       - Remove detached worktree"
        echo "  gg list [options]                - List all worktrees"
        echo "  gg detach [name] [options]       - Detach worktree from branch (also: gg d)"
        echo "  gg go [branch] [options]         - Smart branch switcher (find or checkout branch)"
        echo "  gg pool                          - Show pool status overview"
        echo "  gg init                          - Create .gg_hook.fish template"
        echo ""
        echo "GLOBAL OPTIONS:"
        echo "  -h, --help                       - Show this help message"
        echo "  -v, --verbose                    - Enable verbose output"
        echo "  -q, --quiet                      - Suppress informational output"
        echo ""
        echo "CORE CONCEPTS:"
        echo "  • Worktrees default to detached HEAD state"
        echo "  • Only detached grove worktrees can be removed"
        echo "  • Use detach to manage branch associations"
        echo "  • Grove maintains a pool of temporary workspaces"
        echo ""
        echo "EXAMPLES:"
        echo "  gg                               - Select worktree interactively"
        echo "  gg add                           - Create wt1 (or next available) from main"
        echo "  gg add hotfix --from v1.2.3      - Create named grove from tag"
        echo "  gg go                            - Switch to first detached worktree"
        echo "  gg go feature/new                - Find or checkout feature/new branch"
    end

    # Define subcommand help functions
    function __gg_add_help
        echo "gg add - Create new detached worktree"
        echo "---"
        echo ""
        echo "USAGE:"
        echo "  gg add [name] [options]"
        echo ""
        echo "OPTIONS:"
        echo "  --from <ref>          Base commit/branch/tag (default: main)"
        echo "  -b, --branch <name>   Create and checkout branch (not detached)"
        echo "  --sync                Sync staged/modified/untracked files"
        echo "  --no-hook             Skip hook execution (.gg_hook.fish)"
        echo "  -h, --help            Show this help message"
        echo ""
        echo "EXAMPLES:"
        echo "  gg add                            - Create wt1, wt2, etc. from main"
        echo "  gg add hotfix                     - Create named grove from main"
        echo "  gg add experiment --from develop  - Create detached from develop"
        echo "  gg add --sync                     - Create with current changes"
    end

    function __gg_remove_help
        echo "gg remove - Remove detached worktree"
        echo "---"
        echo ""
        echo "USAGE:"
        echo "  gg remove [<name>] [options]"
        echo ""
        echo "OPTIONS:"
        echo "  -h, --help            Show this help message"
        echo ""
        echo "SAFETY:"
        echo "  Only detached grove worktrees can be removed."
        echo "  Use 'gg detach <name>' first if worktree has a branch."
        echo ""
        echo "EXAMPLES:"
        echo "  gg remove                         - Interactive selection"
        echo "  gg remove hotfix                  - Remove specific detached worktree"
    end



    function __gg_detach_help
        echo "gg detach - Detach worktree from branch"
        echo "---"
        echo ""
        echo "USAGE:"
        echo "  gg detach [grove_name] [options]"
        echo ""
        echo "OPTIONS:"
        echo "  -h, --help            Show this help message"
        echo ""
        echo "BEHAVIOR:"
        echo "  • Detaches HEAD from the current branch using 'git checkout --detach'"
        echo "  • Only works with grove worktrees that have attached branches"
        echo "  • The branch itself is preserved (not deleted)"
        echo "  • Makes the worktree safe for removal with 'gg remove' or 'gg clean'"
        echo ""
        echo "SAFETY:"
        echo "  Detached grove worktrees can be safely removed since they're not tied"
        echo "  to any branch. This prevents accidental loss of branch work."
        echo ""
        echo "EXAMPLES:"
        echo "  gg detach                         - Detach current worktree from its branch"
        echo "  gg detach hotfix                  - Detach specific grove from its branch"
        echo "  gg d                              - Detach current worktree (alias)"
    end

    function __gg_pool_help
        echo "gg pool - Show pool status overview"
        echo "---"
        echo ""
        echo "USAGE:"
        echo "  gg pool [options]"
        echo ""
        echo "OPTIONS:"
        echo "  -h, --help            Show this help message"
        echo ""
        echo "DESCRIPTION:"
        echo "  Display comprehensive statistics about the grove worktree pool:"
        echo "  • Total number of grove worktrees vs regular worktrees"
        echo "  • Number of detached vs attached worktrees"
        echo "  • Disk usage analysis"
        echo "  • Age distribution (< 7 days, 7-30 days, > 30 days old)"
        echo "  • Cleanup suggestions for old groves"
        echo ""
        echo "EXAMPLES:"
        echo "  gg pool                           - Show pool overview"
        echo "  gg pool --verbose                 - Show detailed grove listing"
    end

    function __gg_init_help
        echo "gg init - Create .gg_hook.fish template"
        echo "---"
        echo ""
        echo "USAGE:"
        echo "  gg init [options]"
        echo ""
        echo "OPTIONS:"
        echo "  -h, --help            Show this help message"
        echo ""
        echo "DESCRIPTION:"
        echo "  Creates a .gg_hook.fish template file in the current directory."
        echo "  This hook is executed after 'gg add' commands in the new grove"
        echo "  worktree directory and can be used to:"
        echo "  • Copy configuration files (.env, .vscode, etc.)"
        echo "  • Create symlinks to large directories (node_modules, vendor)"
        echo "  • Install dependencies"
        echo "  • Set up grove-specific environment"
        echo ""
        echo "TEMPLATE VARIABLES:"
        echo "  The hook has access to grove-specific environment variables:"
        echo "  • \$GG_WORKTREE_PATH - Path to the grove worktree"
        echo "  • \$GG_GROVE_NAME    - Name of the grove"
        echo "  • \$GG_BASE_REF      - Base commit/branch/tag"
        echo "  • \$GG_PROJECT_ROOT  - Original project root path"
        echo "  • \$GG_TIMESTAMP     - Grove creation timestamp"
        echo "  • \$GG_IS_DETACHED   - \"true\" if detached, \"false\" if attached"
        echo ""
        echo "EXAMPLES:"
        echo "  gg init                           - Create hook template"
    end

    function __gg_go_help
        echo "gg go - Smart branch switcher for grove worktrees"
        echo "---"
        echo ""
        echo "USAGE:"
        echo "  gg go [branch_name] [options]"
        echo ""
        echo "OPTIONS:"
        echo "  -h, --help            Show this help message"
        echo ""
        echo "DESCRIPTION:"
        echo "  Smart branch switcher that works with the grove worktree pool."
        echo "  With branch name: finds existing worktree with that branch, or"
        echo "  checks out the branch in a detached worktree."
        echo "  Without branch name: switches to first detached worktree."
        echo ""
        echo "BEHAVIOR:"
        echo "  1. If branch specified: look for worktree with that branch"
        echo "  2. If found: switch to that worktree"
        echo "  3. If not found: checkout branch in first detached worktree"
        echo "  4. If no detached worktrees: fail"
        echo ""
        echo "EXAMPLES:"
        echo "  gg go                             - Switch to first detached worktree"
        echo "  gg go feature/new                 - Find or checkout feature/new branch"
        echo "  gg go main                        - Switch to worktree with main branch"
    end


    # Parse global options - stop at first non-option argument
    argparse -s 'h/help' 'v/verbose' 'q/quiet' -- $argv
    or return 1

    # Handle help flag
    if set -ql _flag_help
        __gg_help
        return 0
    end

    # Set verbosity
    set -l verbose (set -ql _flag_verbose; and echo true; or echo false)
    set -l quiet (set -ql _flag_quiet; and echo true; or echo false)

    # Get subcommand
    set -l cmd $argv[1]
    set -e argv[1]

    # Main command logic
    switch "$cmd"
        case "" # Interactive selection
            __gg_interactive -- $verbose $quiet

        case add
            __gg_add -- $argv $verbose $quiet

        case remove rm
            __gg_remove -- $argv $verbose $quiet

        case list ls
            __gg_list -- $argv $verbose $quiet

        case d detach
            __gg_detach -- $argv $verbose $quiet

        case pool
            __gg_pool -- $argv $verbose $quiet

        case init
            __gg_init -- $argv $verbose $quiet

        case go
            __gg_go -- $argv $verbose $quiet

        case help
            __gg_help
            return 0

        case '*'
            echo "Error: Unknown command '$cmd'" >&2
            echo "Run 'gg --help' for usage information." >&2
            return 1
    end
end

# Interactive worktree selection with fzf
function __gg_interactive
    # Parse arguments after --
    set -l verbose $argv[2]
    set -l quiet $argv[3]
    
    # Check if fzf is available
    if not command -sq fzf
        echo "Error: fzf is not installed. Please install fzf to use interactive mode." >&2
        return 1
    end

    # Get repository info
    set -l repo_info (gg_get_repo_info)
    if test $status -ne 0
        echo "Error: Not in a Git repository" >&2
        return 1
    end
    
    set -l parts (string split "|" $repo_info)
    set -l git_dir_resolved $parts[1]
    set -l repo_root $parts[2]
    set -l is_bare $parts[3]

    # Get worktree list
    set -l grove_worktrees (git worktree list 2>/dev/null)
    if test -z "$grove_worktrees"
        echo "Error: No Git repository found or no worktrees exist." >&2
        return 1
    end

    # Create display list with grove names and HEAD states
    set -l grove_items
    for worktree in $grove_worktrees
        set -l path (echo $worktree | string split -f1 ' ')
        set -l resolved_path (path resolve $path)
        
        # Skip bare repositories (they show as "(bare)" in git worktree list)
        # But don't skip detached grove worktrees
        if string match -q "*(bare)" $worktree
            continue
        end
        
        # Determine grove name and HEAD state
        set -l grove_name
        set -l head_state
        
        # Check if this is a grove worktree (starts with "wt" and is sibling to repo)
        set -l worktree_name (basename $resolved_path)
        set -l worktree_parent (dirname $resolved_path)
        
        # Determine expected parent directory for grove worktrees
        set -l expected_parent (path resolve "$git_dir_resolved")
        
        if string match -q "wt*" $worktree_name; and test "$worktree_parent" = "$expected_parent"
            set grove_name $worktree_name
            # Check if HEAD is detached
            set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
            if test -z "$current_branch"
                set head_state "🔸 detached"
            else
                set head_state "🔗 $current_branch"
            end
        else
            # This is a regular worktree (probably main repo)
            set grove_name (basename $resolved_path)
            set -l branch_match (echo $worktree | string match -r '\[([^\]]+)\]')
            if test -n "$branch_match"
                set -l branch (echo $branch_match | string split -f2 '[' | string trim -c ']')
                set head_state "🔗 $branch"
            else
                set head_state "🔸 detached"
            end
        end
        
        set -a grove_items "$grove_name|$head_state|$resolved_path"
    end
    
    # Select grove with fzf
    set -l selected_item (printf '%s\n' $grove_items | fzf \
        --delimiter='|' \
        --with-nth=1,2 \
        --preview-window="right:70%:wrap" \
        --preview='
            set -l parts (string split "|" {})
            set -l grove_name $parts[1]
            set -l head_state $parts[2] 
            set -l worktree_path $parts[3]
            
            # Get current directory for comparison
            set -l current_dir (pwd)
            set -l is_current (test "$current_dir" = "$worktree_path"; and echo "👈 CURRENT" ; or echo "")
            
            echo "Grove Worktree Information"
            echo "---"
            printf "Grove:   %s %s\n" $grove_name $is_current
            printf "State:   %s\n" $head_state
            printf "Path:    %s\n" $worktree_path
            echo ""
            
            # Get stats
            set -l total_changes (git -C "$worktree_path" status --porcelain 2>/dev/null | wc -l | string trim)
            set -l staged_count (git -C "$worktree_path" diff --cached --numstat 2>/dev/null | wc -l | string trim)
            set -l modified_count (git -C "$worktree_path" diff --numstat 2>/dev/null | wc -l | string trim)
            set -l untracked_count (git -C "$worktree_path" ls-files --others --exclude-standard 2>/dev/null | wc -l | string trim)
            
            echo "Working Tree Status"
            printf "Staged: %s  Modified: %s  Untracked: %s  Total: %s\n" $staged_count $modified_count $untracked_count $total_changes
            echo ""
            
            echo "Changed Files"
            set -l changes (git -C "$worktree_path" status --porcelain 2>/dev/null)
            if test -z "$changes"
                echo "Working tree is clean"
            else
                set -l count 0
                for change in $changes
                    set count (math $count + 1)
                    if test $count -gt 10
                        printf "... and %d more files\n" (math (count $changes) - 10)
                        break
                    end
                    
                    set -l file_status (string sub -l 2 -- $change)
                    set -l file (string sub -s 4 -- $change)
                    
                    switch $file_status
                        case "M " " M" "MM"
                            echo "M  $file"
                        case "A " "AM"
                            echo "A  $file"
                        case "D " " D"
                            echo "D  $file"
                        case "R "
                            echo "R  $file"
                        case "??"
                            echo "?  $file"
                        case "*"
                            echo "[$file_status] $file"
                    end
                end
            end
            echo ""
            
            echo "Recent Commits"
            set -l commits (git -C "$worktree_path" log --oneline --color=always -8 2>/dev/null)
            if test -n "$commits"
                for commit in $commits
                    echo "$commit"
                end
            else
                echo "No commits yet"
            end
        ' \
        --header="Git Grove - Worktree Pool    ↑/↓ Navigate  ⏎ Select  ^C Cancel
🔸 = detached HEAD    🔗 = attached branch" \
        --border=rounded \
        --height=80% \
        --layout=reverse \
        --prompt="grove › " \
        --ansi)
    
    if test -n "$selected_item"
        set -l parts (string split "|" $selected_item)
        set -l worktree_path $parts[3]
        
        if test -d "$worktree_path"
            cd "$worktree_path"
            test "$verbose" = true; and echo "Switched to grove: $worktree_path"
            return 0
        else
            echo "Error: Directory not found: $worktree_path" >&2
            return 1
        end
    end
end

# Add new detached worktree
function __gg_add
    # The first argument is always "--", followed by actual arguments, then verbose and quiet
    set -l actual_argv $argv[2..-3]  # Skip first "--" and last two (verbose, quiet)
    set -l verbose $argv[-2]
    set -l quiet $argv[-1]
    
    # Parse add-specific options
    argparse 'from=' 'b/branch=' 'sync' 'no-hook' 'h/help' -- $actual_argv
    or return 1
    
    # Handle help flag
    if set -ql _flag_help
        __gg_add_help
        return 0
    end
    
    # Get grove name from remaining arguments after argparse
    set -l grove_name $argv[1]
    
    # If no grove name provided, auto-generate wt1, wt2, etc.
    if test -z "$grove_name"
        set grove_name (gg_generate_name)
        test "$verbose" = true; and echo "[INFO] Auto-generated grove name: $grove_name"
    else
        # Validate grove name (no special characters that would break paths)
        if not string match -qr '^[a-zA-Z0-9._-]+$' "$grove_name"
            echo "Error: Grove name must contain only letters, numbers, dots, underscores, and hyphens" >&2
            return 1
        end
    end
    
    # Store current directory and current branch before changing directory
    set -l original_dir $PWD
    set -l current_branch (git branch --show-current 2>/dev/null)
    
    # Get repository info (handles both bare and regular repos)
    set -l repo_info (gg_get_repo_info)
    if test $status -ne 0
        echo "Error: Not in a Git repository" >&2
        return 1
    end
    
    set -l parts (string split "|" $repo_info)
    set -l git_dir_resolved $parts[1]
    set -l repo_root $parts[2]
    set -l is_bare $parts[3]
    
    if test "$is_bare" = "true"
        test "$verbose" = true; and echo "Working in bare repository: $repo_root"
    else
        cd "$repo_root"
        test "$verbose" = true; and echo "Changed to repository root: $repo_root"
    end
    
    # Determine worktree path
    set -l worktree_path
    if test "$is_bare" = "true"
        # For bare repos, create worktrees at top level of the bare repo
        set worktree_path "$git_dir_resolved/$grove_name"
    else
        # For regular repos, create worktrees inside .git directory
        set worktree_path "$git_dir_resolved/$grove_name"
    end
    
    # Check if worktree already exists
    if test -d "$worktree_path"
        echo "Error: Grove worktree already exists at: $worktree_path" >&2
        return 1
    end
    
    # Get base reference (default to main)
    set -l base_ref
    if set -ql _flag_from
        set base_ref $_flag_from
        # Verify the reference exists
        if not git rev-parse --verify "$base_ref" &>/dev/null
            echo "Error: Reference '$base_ref' does not exist" >&2
            return 1
        end
    else
        # Default to main branch
        set base_ref "main"
        # Check if main exists, otherwise try master
        if not git rev-parse --verify main &>/dev/null
            if git rev-parse --verify master &>/dev/null
                set base_ref "master"
            else
                # Fallback to HEAD
                set base_ref "HEAD"
            end
        end
    end
    
    # Check for unstaged changes before creating worktree (only in non-bare repos)
    set -l should_sync_changes false
    set -l has_unstaged_changes ""
    
    if test "$is_bare" != "true"
        set has_unstaged_changes (git status --porcelain 2>/dev/null)
    end
    
    # Only sync changes if --sync flag is explicitly provided and not in bare repo
    if set -ql _flag_sync
        if test "$is_bare" = "true"
            test "$verbose" = true; and echo "[INFO] Ignoring --sync flag in bare repository (no working tree to sync)"
        else
            set should_sync_changes true
            test "$verbose" = true; and echo "[INFO] Syncing changes (--sync flag provided)"
        end
    else if test -n "$has_unstaged_changes"
        test "$verbose" = true; and echo "[INFO] Skipping unstaged changes sync (use --sync to include changes)"
    end
    
    # Clean up any stale locks before creating worktree
    set -l cleanup_attempts 0
    while test $cleanup_attempts -lt 3
        if gg_cleanup_stale_locks "$verbose"
            break
        end
        set cleanup_attempts (math $cleanup_attempts + 1)
        test "$verbose" = true; and echo "[RETRY] Waiting for index lock to clear (attempt $cleanup_attempts/3)"
        sleep 2
    end
    
    # Create worktree - key difference: create detached by default
    test "$quiet" = false; and echo "Creating grove '$grove_name' from $base_ref..."
    
    if set -ql _flag_branch
        # User wants to create with a branch (not detached)
        set -l branch_name $_flag_branch
        test "$quiet" = false; and echo "[INFO] Creating with branch '$branch_name' (not detached)"
        if git worktree add -b "$branch_name" "$worktree_path" "$base_ref" &>/tmp/gg_add.log
            test "$quiet" = false; and echo "[OK] Created grove at: $worktree_path"
            test "$quiet" = false; and echo "     Branch: $branch_name (based on $base_ref)"
        else
            echo "Error: Failed to create grove worktree" >&2
            test "$verbose" = true; and cat /tmp/gg_add.log >&2
            rm -f /tmp/gg_add.log
            # Clean up any locks that might have been left
            gg_cleanup_stale_locks false >/dev/null 2>&1
            return 1
        end
    else
        # Default: create detached worktree
        if git worktree add --detach "$worktree_path" "$base_ref" &>/tmp/gg_add.log
            test "$quiet" = false; and echo "[OK] Created detached grove at: $worktree_path"
            test "$quiet" = false; and echo "     Detached at: $base_ref"
        else
            echo "Error: Failed to create grove worktree" >&2
            test "$verbose" = true; and cat /tmp/gg_add.log >&2
            rm -f /tmp/gg_add.log
            # Clean up any locks that might have been left
            gg_cleanup_stale_locks false >/dev/null 2>&1
            return 1
        end
    end
    
    # Store project root (use repo_root which handles bare repos)
    set -l project_root $repo_root
    
    # Sync all changes (staged, unstaged, and untracked) only if should_sync_changes is true
    if test "$should_sync_changes" = true
        test "$quiet" = false; and echo "[SYNC] Syncing all changes..."
        
        # Get list of files
        set -l staged_files (git diff --cached --name-only)
        set -l modified_files (git diff --name-only)
        set -l untracked_files (git ls-files --others --exclude-standard)
        
        # Copy staged files
        for file in $staged_files
            if test -f "$repo_root/$file"
                set -l dir_path (dirname "$worktree_path/$file")
                mkdir -p "$dir_path"
                cp "$repo_root/$file" "$worktree_path/$file"
                test "$verbose" = true; and echo "       Copied staged: $file"
            end
        end
        
        # Copy modified files (unstaged changes)
        for file in $modified_files
            if test -f "$repo_root/$file"
                set -l dir_path (dirname "$worktree_path/$file")
                mkdir -p "$dir_path"
                cp "$repo_root/$file" "$worktree_path/$file"
                test "$verbose" = true; and echo "       Copied modified: $file"
            end
        end
        
        # Copy untracked files
        for file in $untracked_files
            if test -f "$repo_root/$file"
                set -l dir_path (dirname "$worktree_path/$file")
                mkdir -p "$dir_path"
                cp "$repo_root/$file" "$worktree_path/$file"
                test "$verbose" = true; and echo "       Copied untracked: $file"
            end
        end
        
        test "$quiet" = false; and echo "[OK] Synced all changes"
    end
    
    # Change to new worktree
    cd "$worktree_path"
    
    # Execute hook if exists and not disabled
    if not set -ql _flag_no_hook; and test -f "$project_root/.gg_hook.fish"
        test "$quiet" = false; and echo "[HOOK] Executing .gg_hook.fish..."
        
        # Set environment variables for hook
        set -gx GG_WORKTREE_PATH "$worktree_path"
        set -gx GG_GROVE_NAME "$grove_name"
        set -gx GG_BASE_REF "$base_ref"
        set -gx GG_PROJECT_ROOT "$project_root"
        set -gx GG_TIMESTAMP (date +"%Y-%m-%d %H:%M:%S")
        set -gx GG_IS_DETACHED (set -ql _flag_branch; and echo "false"; or echo "true")
        
        source "$project_root/.gg_hook.fish"
        set -l hook_status $status
        
        # Clean up environment variables
        set -e GG_WORKTREE_PATH
        set -e GG_GROVE_NAME
        set -e GG_BASE_REF
        set -e GG_PROJECT_ROOT
        set -e GG_TIMESTAMP
        set -e GG_IS_DETACHED
        
        if test $hook_status -ne 0
            echo "[WARN] Hook execution failed with status $hook_status" >&2
        else
            test "$quiet" = false; and echo "[OK] Hook executed successfully"
        end
    end
    
    test "$quiet" = false; and echo "[PWD] Now in: $worktree_path"
    
    # Always clean up temp files
    rm -f /tmp/gg_add.log
end

function __gg_remove
    # The first argument is always "--", followed by actual arguments, then verbose and quiet
    set -l actual_argv $argv[2..-3]  # Skip first "--" and last two (verbose, quiet)
    set -l verbose $argv[-2]
    set -l quiet $argv[-1]
    
    # Parse remove-specific options
    argparse 'h/help' -- $actual_argv
    or return 1
    
    # Handle help flag
    if set -ql _flag_help
        __gg_remove_help
        return 0
    end
    
    set -l grove_name $argv[1]
    
    # Get repository info (handles both bare and regular repos)
    set -l repo_info (gg_get_repo_info)
    if test $status -ne 0
        echo "Error: Not in a Git repository" >&2
        return 1
    end
    
    set -l git_dir_resolved (string split '|' $repo_info)[1]
    set -l repo_root (string split '|' $repo_info)[2]
    set -l is_bare (string split '|' $repo_info)[3]
    
    # If no grove name provided, use fzf for interactive selection
    if test -z "$grove_name"
        # Check if fzf is available
        if not command -sq fzf
            echo "Error: Grove name required or install fzf for interactive selection" >&2
            echo "Usage: gg remove <grove_name>" >&2
            return 1
        end
        
        # Get all grove worktrees that are detached (safety: only show detached for removal)
        set -l detached_groves
        
        # Determine expected parent directory for grove worktrees
        set -l expected_parent (path resolve "$git_dir_resolved")
        
        # Get worktree list and find detached grove worktrees
        set -l grove_worktrees (git worktree list 2>/dev/null)
        for worktree in $grove_worktrees
            # Skip bare repositories
            if string match -q "*(bare)" $worktree
                continue
            end
            
            set -l path (string split ' ' $worktree)[1]
            set -l resolved_path (path resolve $path)
            set -l worktree_name (basename $resolved_path)
            set -l worktree_parent (dirname $resolved_path)
            
            # Check if this is a grove worktree
            if string match -q "wt*" $worktree_name; and test "$worktree_parent" = "$expected_parent"
                # Check if HEAD is detached
                set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                if test -z "$current_branch"
                    set -a detached_groves "$worktree_name|$resolved_path"
                end
            end
        end
        
        if test -z "$detached_groves"
            echo "No detached grove worktrees found (only detached groves can be removed)" >&2
            return 1
        end
        
        # Select grove with fzf
        set -l selected_grove (printf '%s\n' $detached_groves | fzf \
            --delimiter='|' \
            --with-nth=1 \
            --preview-window="right:70%:wrap" \
            --preview='
                set -l parts (string split "|" {})
                set -l grove_name $parts[1]
                set -l worktree_path $parts[2]
                
                echo "Grove Removal Preview"
                echo "---"
                printf "Grove:   %s\n" $grove_name
                printf "Status:  %s\n" "🔸 detached"
                printf "Path:    %s\n" $worktree_path
                echo ""
                
                # Get stats
                set -l total_changes (git -C "$worktree_path" status --porcelain 2>/dev/null | wc -l | string trim)
                set -l staged_count (git -C "$worktree_path" diff --cached --numstat 2>/dev/null | wc -l | string trim)
                set -l modified_count (git -C "$worktree_path" diff --numstat 2>/dev/null | wc -l | string trim)
                set -l untracked_count (git -C "$worktree_path" ls-files --others --exclude-standard 2>/dev/null | wc -l | string trim)
                
                echo "Working Tree Status"
                printf "Staged: %s  Modified: %s  Untracked: %s  Total: %s\n" $staged_count $modified_count $untracked_count $total_changes
                echo ""
                
                echo "Changed Files"
                set -l changes (git -C "$worktree_path" status --porcelain 2>/dev/null)
                if test -z "$changes"
                    echo "Working tree is clean"
                else
                    set -l count 0
                    for change in $changes
                        set count (math $count + 1)
                        if test $count -gt 8
                            printf "... and %d more files\n" (math (count $changes) - 8)
                            break
                        end
                        
                        set -l file_status (string sub -l 2 -- $change)
                        set -l file (string sub -s 4 -- $change)
                        
                        switch $file_status
                            case "M " " M" "MM"
                                echo "M  $file"
                            case "A " "AM"
                                echo "A  $file"
                            case "D " " D"
                                echo "D  $file"
                            case "R "
                                echo "R  $file"
                            case "??"
                                echo "?  $file"
                            case "*"
                                echo "[$file_status] $file"
                        end
                    end
                end
                echo ""
                
                echo "⚠️  WARNING: This will permanently remove the grove and all changes!"
            ' \
            --header="Remove Grove Worktree    ↑/↓ Navigate  ⏎ Remove  ^C Cancel
🔸 = detached (removable)" \
            --border=rounded \
            --height=80% \
            --layout=reverse \
            --prompt="remove › " \
            --ansi)
        
        if test -z "$selected_grove"
            echo "Cancelled"
            return 0
        end
        
        # Parse selected grove info
        set -l parts (string split "|" $selected_grove)
        set grove_name $parts[1]
    end
    
    # Find worktree by grove name
    set -l worktree_path
    
    # Determine expected parent directory for grove worktrees
    set -l expected_parent (path resolve "$git_dir_resolved")
    
    # Get worktree list and find the grove worktree
    set -l grove_worktrees (git worktree list 2>/dev/null)
    for worktree in $grove_worktrees
        # Skip bare repositories
        if string match -q "*(bare)" $worktree
            continue
        end
        
        set -l path (string split ' ' $worktree)[1]
        set -l resolved_path (path resolve $path)
        set -l worktree_name (basename $resolved_path)
        set -l worktree_parent (dirname $resolved_path)
        
        # Check if this is the grove we're looking for
        if string match -q "wt*" $worktree_name; and test "$worktree_parent" = "$expected_parent"; and test "$worktree_name" = "$grove_name"
            set worktree_path "$resolved_path"
            break
        end
    end
    
    if test -z "$worktree_path"
        echo "Error: Grove '$grove_name' not found" >&2
        return 1
    end
    
    # Safety check: Verify the worktree is detached
    set -l current_branch (git -C "$worktree_path" branch --show-current 2>/dev/null)
    if test -n "$current_branch"
        echo "Error: Grove '$grove_name' is attached to branch '$current_branch'" >&2
        echo "Safety rule: Only detached grove worktrees can be removed." >&2
        echo "Use 'gg detach $grove_name' first to detach from the branch." >&2
        return 1
    end
    
    # Show what will be removed
    echo "Grove to remove: $grove_name"
    echo "Path: $worktree_path"
    echo "Status: 🔸 detached (safe to remove)"
    
    # Show working tree status if there are changes
    set -l changes (git -C "$worktree_path" status --porcelain 2>/dev/null)
    if test -n "$changes"
        set -l change_count (echo "$changes" | wc -l | string trim)
        echo "⚠️  Warning: $change_count uncommitted changes will be lost"
    end
    echo ""
    
    # Confirmation
    read -l -P "Remove this grove permanently? (y/N) " confirm
    if not string match -qi 'y' $confirm
        echo "Cancelled"
        return 0
    end
    
    # Clean up any stale locks before removing worktree
    gg_cleanup_stale_locks "$verbose" >/dev/null
    
    # Remove worktree
    test "$quiet" = false; and echo "Removing grove worktree..."
    if git worktree remove --force "$worktree_path" &>/tmp/gg_remove.log
        test "$quiet" = false; and echo "[OK] Removed grove worktree: $worktree_path"
        
        # Remove the directory if it still exists (shouldn't, but just to be safe)
        if test -d "$worktree_path"
            rm -rf "$worktree_path"
            test "$verbose" = true; and echo "[CLEANUP] Removed leftover directory"
        end
        
        test "$quiet" = false; and echo "[OK] Grove '$grove_name' removed successfully"
    else
        echo "Error: Failed to remove grove worktree" >&2
        test "$verbose" = true; and cat /tmp/gg_remove.log >&2
        rm -f /tmp/gg_remove.log
        # Clean up any locks that might have been left
        gg_cleanup_stale_locks false >/dev/null 2>&1
        return 1
    end
    
    # Always clean up temp files
    rm -f /tmp/gg_remove.log
end

function __gg_list
    # The first argument is always "--", followed by actual arguments, then verbose and quiet
    set -l actual_argv $argv[2..-3]  # Skip first "--" and last two (verbose, quiet)
    set -l verbose $argv[-2]
    set -l quiet $argv[-1]
    
    # Parse list-specific options
    argparse 'h/help' -- $actual_argv
    or return 1
    
    # Handle help flag - show basic help since __gg_list_help doesn't exist yet
    if set -ql _flag_help
        echo "╭──────────────────────────────────────────────────────────╮"
        echo "│ gg list - List all worktrees                                  │"
        echo "╰──────────────────────────────────────────────────────────╯"
        echo ""
        echo "USAGE:"
        echo "  gg list [options]"
        echo ""
        echo "OPTIONS:"
        echo "  -h, --help            Show this help message"
        echo ""
        echo "DESCRIPTION:"
        echo "  Display all grove worktrees and regular worktrees with their"
        echo "  grove names, HEAD state (detached/branch), paths, working"
        echo "  tree status, and last commits."
        return 0
    end
    
    # Get repository info
    set -l repo_info (gg_get_repo_info)
    if test $status -ne 0
        echo "Error: Not in a Git repository" >&2
        return 1
    end
    
    set -l parts (string split "|" $repo_info)
    set -l git_dir_resolved $parts[1]
    set -l repo_root $parts[2]
    set -l is_bare $parts[3]
    
    # Get all worktrees
    set -l grove_worktrees (git worktree list 2>/dev/null)
    if test -z "$grove_worktrees"
        echo "No Git repository found or no worktrees exist." >&2
        return 1
    end
    
    test "$quiet" = false; and echo "Git Grove - Worktree Pool Status"
    test "$quiet" = false; and echo "---"
    test "$quiet" = false; and echo ""
    
    # Process each worktree
    for worktree in $grove_worktrees
        set -l path (echo $worktree | string split -f1 ' ')
        set -l resolved_path (path resolve $path)
        
        # Skip bare repositories (they show as "(bare)" in git worktree list)
        # But don't skip detached grove worktrees
        if string match -q "*(bare)" $worktree
            continue
        end
        
        # Determine grove name and HEAD state
        set -l grove_name
        set -l head_state
        set -l is_grove false
        
        # Check if this is a grove worktree (starts with "wt" and is sibling to repo)
        set -l worktree_name (basename $resolved_path)
        set -l worktree_parent (dirname $resolved_path)
        
        # Determine expected parent directory for grove worktrees
        set -l expected_parent (path resolve "$git_dir_resolved")
        
        if string match -q "wt*" $worktree_name; and test "$worktree_parent" = "$expected_parent"
            set is_grove true
            set grove_name $worktree_name
            
            # Check if HEAD is detached
            set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
            if test -z "$current_branch"
                set head_state "🔸 detached"
            else
                set head_state "🔗 $current_branch"
            end
        else
            # This is a regular worktree (probably main repo)
            set grove_name (basename $resolved_path)
            # Parse branch from worktree list output  
            set -l branch_match (echo $worktree | string match -r '\[([^\]]+)\]')
            if test -n "$branch_match"
                set -l branch (echo $branch_match | string replace -r '.*\[([^\]]+)\].*' '$1')
                set head_state "🔗 $branch"
            else
                set head_state "🔸 detached"
            end
        end
        
        # Get working tree status
        set -l changes (git -C "$resolved_path" status --porcelain 2>/dev/null)
        set -l change_count (echo "$changes" | wc -l | string trim)
        set -l status_text
        if test -z "$changes"
            set status_text "clean"
        else
            # Count different types of changes
            set -l staged_count (git -C "$resolved_path" diff --cached --numstat 2>/dev/null | wc -l | string trim)
            set -l modified_count (git -C "$resolved_path" diff --numstat 2>/dev/null | wc -l | string trim)
            set -l untracked_count (git -C "$resolved_path" ls-files --others --exclude-standard 2>/dev/null | wc -l | string trim)
            
            if test $change_count -eq 1
                set status_text "$change_count change"
            else
                set status_text "$change_count changes"
            end
            
            # Add breakdown if verbose
            if test "$verbose" = true
                set status_text "$status_text (S:$staged_count M:$modified_count U:$untracked_count)"
            end
        end
        
        # Get last commit (hash + message)
        set -l last_commit (git -C "$resolved_path" log -1 --format="%h %s" 2>/dev/null)
        if test -z "$last_commit"
            set last_commit "No commits"
        end
        
        # Output format
        if test "$is_grove" = true
            echo "Grove: $grove_name"
        else
            echo "Worktree: $grove_name"
        end
        echo "  State: $head_state"
        echo "  Path: $resolved_path"
        echo "  Status: $status_text"
        echo "  Last: $last_commit"
        
        # Show detailed file changes if verbose and there are changes
        if test "$verbose" = true -a -n "$changes"
            echo "  Files:"
            set -l file_count 0
            for change in $changes
                set file_count (math $file_count + 1)
                if test $file_count -gt 8
                    set -l remaining (math (echo "$changes" | wc -l | string trim) - 8)
                    echo "    ... and $remaining more files"
                    break
                end
                
                set -l file_status (string sub -l 2 -- $change)
                set -l file (string sub -s 4 -- $change)
                
                switch $file_status
                    case "M " " M" "MM"
                        echo "    M  $file"
                    case "A " "AM"  
                        echo "    A  $file"
                    case "D " " D"
                        echo "    D  $file"
                    case "R "
                        echo "    R  $file"
                    case "??"
                        echo "    ?  $file"
                    case "*"
                        echo "    $file_status $file"
                end
            end
        end
        
        echo ""
    end
    
    # Show summary
    if test "$quiet" = false
        set -l total_grove_worktrees (echo "$grove_worktrees" | wc -l | string trim)
        set -l grove_count 0
        set -l regular_count 0
        set -l detached_count 0
        set -l attached_count 0
        
        # Determine expected parent directory for grove worktrees
        set -l expected_parent (path resolve "$git_dir_resolved")
        
        for worktree in $grove_worktrees
            set -l path (echo $worktree | string split -f1 ' ')
            set -l resolved_path (path resolve $path)
            set -l worktree_name (basename $resolved_path)
            set -l worktree_parent (dirname $resolved_path)
            
            if string match -q "wt*" $worktree_name; and test "$worktree_parent" = "$expected_parent"
                set grove_count (math $grove_count + 1)
                set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                if test -z "$current_branch"
                    set detached_count (math $detached_count + 1)
                else
                    set attached_count (math $attached_count + 1)
                end
            else
                set regular_count (math $regular_count + 1)
                # Check if regular worktree is attached
                set -l branch_match (echo $worktree | string match -r '\[([^\]]+)\]')
                if test -n "$branch_match"
                    set attached_count (math $attached_count + 1)
                else
                    set detached_count (math $detached_count + 1)
                end
            end
        end
        
        echo "---"
        echo "Summary: $total_grove_worktrees total ($grove_count grove, $regular_count regular) • $detached_count detached, $attached_count attached"
    end
end



function __gg_detach
    # Parse arguments: -- actual_args... verbose quiet
    set -l actual_argv $argv[2..-3]
    set -l verbose $argv[-2]
    set -l quiet $argv[-1]
    
    # Parse detach-specific options
    argparse 'h/help' -- $actual_argv
    or return 1
    
    # Handle help flag
    if set -ql _flag_help
        __gg_detach_help
        return 0
    end
    
    # Get grove name from remaining arguments after argparse
    set -l grove_name $argv[1]
    set -l current_dir $PWD
    
    # Get repository information first
    set -l repo_info (gg_get_repo_info)
    if test -z "$repo_info"
        echo "Error: Not in a Git repository" >&2
        return 1
    end
    
    set -l git_dir_resolved (string split '|' $repo_info)[1]
    set -l repo_root (string split '|' $repo_info)[2]
    set -l is_bare (string split '|' $repo_info)[3]
    
    # If no grove name provided, try to detach current worktree
    if test -z "$grove_name"
        # Determine expected parent directory for grove worktrees
        set -l expected_parent (path resolve "$git_dir_resolved")
        
        # Check if we're in a grove worktree
        set -l worktree_name (basename $current_dir)
        set -l worktree_parent (dirname $current_dir)
        
        if string match -q "wt*" $worktree_name; and test "$worktree_parent" = "$expected_parent"
            set grove_name $worktree_name
            test "$verbose" = true; and echo "[INFO] Auto-detected current grove: $grove_name"
        else
            # Check if current directory is a worktree that can be detached
            set -l current_branch (git branch --show-current 2>/dev/null)
            if test -n "$current_branch"
                test "$quiet" = false; and echo "Detaching current worktree from branch '$current_branch'..."
                if git checkout --detach &>/tmp/gg_detach.log
                    test "$quiet" = false; and echo "[OK] Detached current worktree from branch '$current_branch'"
                    rm -f /tmp/gg_detach.log
                    return 0
                else
                    echo "Error: Failed to detach current worktree" >&2
                    test "$verbose" = true; and cat /tmp/gg_detach.log >&2
                    rm -f /tmp/gg_detach.log
                    # Clean up any locks that might have been left
                    gg_cleanup_stale_locks false >/dev/null 2>&1
                    return 1
                end
            else
                echo "Error: Current worktree is already detached" >&2
                return 1
            end
        end
    end
    # Find worktree by grove name using git worktree list
    set -l worktree_path
    set -l grove_worktrees (git worktree list 2>/dev/null)
    
    set -l expected_parent (path resolve "$git_dir_resolved")
    
    for worktree in $grove_worktrees
        # Skip bare repositories
        if string match -q "*(bare)" $worktree
            continue
        end
        
        set -l path (string split ' ' $worktree)[1]
        set -l resolved_path (path resolve $path)
        set -l worktree_name (basename $resolved_path)
        set -l worktree_parent (dirname $resolved_path)
        
        # Check if this is the grove we're looking for
        if string match -q "wt*" $worktree_name; and test "$worktree_parent" = "$expected_parent"; and test "$worktree_name" = "$grove_name"
            set worktree_path "$resolved_path"
            break
        end
    end
    
    if test -z "$worktree_path"
        echo "Error: Grove '$grove_name' not found" >&2
        return 1
    end
    
    # Verify the worktree currently has an attached branch
    set -l current_branch (git -C "$worktree_path" branch --show-current 2>/dev/null)
    if test -z "$current_branch"
        echo "Error: Grove '$grove_name' is already detached" >&2
        echo "Current state: HEAD is detached" >&2
        return 1
    end
    
    # Show current state before detaching
    test "$quiet" = false; and echo "[DETACH] Detaching grove '$grove_name' from branch '$current_branch'..."
    
    # Clean up any stale locks before checkout operation
    gg_cleanup_stale_locks "$verbose" >/dev/null
    
    # Change to worktree directory for git operations
    set -l original_dir $PWD
    cd "$worktree_path"
    
    # Get current commit to detach to
    set -l current_commit (git rev-parse HEAD 2>/dev/null)
    if test -z "$current_commit"
        echo "Error: Unable to determine current commit" >&2
        cd "$original_dir"
        return 1
    end
    
    # Detach HEAD from the branch
    if git checkout --detach &>/tmp/gg_detach.log
        test "$quiet" = false; and echo "[OK] Successfully detached from branch '$current_branch'"
        test "$quiet" = false; and echo "     HEAD is now at: $(git rev-parse --short HEAD) $(git log -1 --format='%s' 2>/dev/null)"
    else
        echo "Error: Failed to detach from branch '$current_branch'" >&2
        test "$verbose" = true; and cat /tmp/gg_detach.log >&2
        rm -f /tmp/gg_detach.log
        # Clean up any locks that might have been left
        gg_cleanup_stale_locks false >/dev/null 2>&1
        cd "$original_dir"
        return 1
    end
    
    # Return to original directory
    cd "$original_dir"
    
    test "$quiet" = false; and echo "[INFO] Grove '$grove_name' is now detached (safe for removal)"
    test "$quiet" = false; and echo "       Path: $worktree_path"
    test "$verbose" = true; and echo "       Previous branch '$current_branch' is preserved"
    
    # Always clean up temp files
    rm -f /tmp/gg_detach.log
end

function __gg_pool
    # Parse arguments: -- actual_args... verbose quiet
    set -l actual_argv $argv[2..-3]
    set -l verbose $argv[-2]
    set -l quiet $argv[-1]
    
    # Parse pool-specific options
    argparse 'h/help' -- $actual_argv
    or return 1
    
    # Handle help flag
    if set -ql _flag_help
        __gg_pool_help
        return 0
    end
    
    # Get repository information
    set -l repo_info (gg_get_repo_info)
    if test -z "$repo_info"
        echo "Error: Not in a Git repository" >&2
        return 1
    end
    
    set -l git_dir_resolved (string split '|' $repo_info)[1]
    set -l repo_root (string split '|' $repo_info)[2]
    set -l is_bare (string split '|' $repo_info)[3]
    
    test "$quiet" = false; and echo "Git Grove - Worktree Pool Status"
    test "$quiet" = false; and echo "---"
    test "$quiet" = false; and echo ""
    
    # Get all worktrees
    set -l grove_worktrees (git worktree list 2>/dev/null)
    if test -z "$grove_worktrees"
        echo "No Git repository found or no worktrees exist." >&2
        return 1
    end
    
    # Initialize counters
    set -l total_grove_worktrees (echo "$grove_worktrees" | wc -l | string trim)
    set -l grove_count 0
    set -l regular_count 0
    set -l detached_count 0
    set -l attached_count 0
    set -l grove_disk_usage 0
    set -l age_week_count 0
    set -l age_month_count 0
    set -l age_old_count 0
    
    # Calculate current time for age analysis
    set -l current_time (date +%s)
    set -l week_cutoff (math "$current_time - (7 * 86400)")
    set -l month_cutoff (math "$current_time - (30 * 86400)")
    
    # Determine expected parent directory for grove worktrees
    set -l expected_parent (path resolve "$git_dir_resolved")
    
    # Process each worktree
    for worktree in $grove_worktrees
        set -l path (echo $worktree | string split -f1 ' ')
        set -l resolved_path (path resolve $path)
        
        # Skip bare repositories (they show as "(bare)" in git worktree list)
        # But don't skip detached grove worktrees
        if string match -q "*(bare)" $worktree
            continue
        end
        
        # Check if this is a grove worktree
        set -l worktree_name (basename $resolved_path)
        set -l worktree_parent (dirname $resolved_path)
        
        if string match -q "wt*" $worktree_name; and test "$worktree_parent" = "$expected_parent"
            set grove_count (math $grove_count + 1)
            
            # Check if HEAD is detached
            set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
            if test -z "$current_branch"
                set detached_count (math $detached_count + 1)
            else
                set attached_count (math $attached_count + 1)
            end
            
            # Get grove disk usage
            set -l dir_size (du -sk "$resolved_path" 2>/dev/null | string split -f1)
            if test -n "$dir_size"
                set grove_disk_usage (math "$grove_disk_usage + $dir_size")
            end
            
            # Get age information
            set -l last_activity_date (git -C "$resolved_path" log -1 --format=%ct 2>/dev/null)
            if test -z "$last_activity_date"
                # If no commits, use directory modification time
                set last_activity_date (stat -f %m "$resolved_path" 2>/dev/null; or stat -c %Y "$resolved_path" 2>/dev/null)
            end
            
            if test -n "$last_activity_date"
                if test "$last_activity_date" -lt "$month_cutoff"
                    set age_old_count (math $age_old_count + 1)
                else if test "$last_activity_date" -lt "$week_cutoff"
                    set age_month_count (math $age_month_count + 1)
                else
                    set age_week_count (math $age_week_count + 1)
                end
            end
        else
            set regular_count (math $regular_count + 1)
            # Check if regular worktree is attached
            set -l branch_match (echo $worktree | string match -r '\[([^\]]+)\]')
            if test -n "$branch_match"
                set attached_count (math $attached_count + 1)
            else
                set detached_count (math $detached_count + 1)
            end
        end
    end
    
    # Display statistics
    test "$quiet" = false; and echo "Worktree Overview"
    test "$quiet" = false; and printf "Total Worktrees:    %s   (%s grove, %s regular)\n" $total_grove_worktrees $grove_count $regular_count
    test "$quiet" = false; and printf "Branch Status:      %s   (%s detached, %s attached)\n" $total_grove_worktrees $detached_count $attached_count
    test "$quiet" = false; and echo ""
    
    # Grove-specific statistics
    if test $grove_count -gt 0
        test "$quiet" = false; and echo "Grove Pool Statistics"
        
        # Convert KB to human readable format
        set -l disk_usage_mb (math "$grove_disk_usage / 1024")
        set -l disk_usage_display
        if test $disk_usage_mb -gt 1024
            set -l disk_usage_gb (math "$disk_usage_mb / 1024")
            set disk_usage_display "{$disk_usage_gb}GB"
        else
            set disk_usage_display "{$disk_usage_mb}MB"
        end
        
        test "$quiet" = false; and printf "Disk Usage:         %s (~%sKB total)\n" $disk_usage_display $grove_disk_usage
        test "$quiet" = false; and printf "Age Distribution:   < 7 days: %s  |  7-30 days: %s  |  > 30 days: %s\n" $age_week_count $age_month_count $age_old_count
        
        if test $age_old_count -gt 0
            test "$quiet" = false; and printf "💡 Cleanup Suggestion: %s grove(s) > 30 days old (use 'gg clean')\n" $age_old_count
        end
    else
        test "$quiet" = false; and echo "Grove Pool Status"
        test "$quiet" = false; and echo "No grove worktrees found."
        test "$quiet" = false; and echo ""
        test "$quiet" = false; and echo "Use 'gg add <name>' to create your first grove worktree."
    end
    
    # Show grove details if verbose
    if test "$verbose" = true -a $grove_count -gt 0
        test "$quiet" = false; and echo ""
        test "$quiet" = false; and echo "Grove Contents:"
        
        # List grove worktrees if verbose - scan through worktrees again
        for worktree in $grove_worktrees
            # Skip bare repositories
            if string match -q "*(bare)" $worktree
                continue
            end
            
            set -l path (string split ' ' $worktree)[1]
            set -l resolved_path (path resolve $path)
            set -l worktree_name (basename $resolved_path)
            set -l worktree_parent (dirname $resolved_path)
            
            # Check if this is a grove worktree
            if string match -q "wt*" $worktree_name; and test "$worktree_parent" = "$expected_parent"
                # Check if HEAD is detached
                set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                set -l state_indicator
                if test -z "$current_branch"
                    set state_indicator "🔸"
                else
                    set state_indicator "🔗 $current_branch"
                end
                
                # Get last commit info
                set -l last_commit (git -C "$resolved_path" log -1 --format="%h %s" 2>/dev/null)
                if test -z "$last_commit"
                    set last_commit "No commits"
                end
                
                test "$quiet" = false; and echo "  - $worktree_name $state_indicator"
                test "$quiet" = false; and echo "    Last: $last_commit"
            end
        end
    end
end

function __gg_init
    # Parse arguments: -- actual_args... verbose quiet
    set -l actual_argv $argv[2..-3]
    set -l verbose $argv[-2]
    set -l quiet $argv[-1]
    
    # Parse init-specific options
    argparse 'h/help' -- $actual_argv
    or return 1
    
    # Handle help flag
    if set -ql _flag_help
        __gg_init_help
        return 0
    end
    
    # Check if .gg_hook.fish already exists
    if test -f ".gg_hook.fish"
        echo "Error: .gg_hook.fish already exists" >&2
        echo "Remove it first if you want to recreate it." >&2
        return 1
    end
    
    # Create the .gg_hook.fish template
    echo '#!/usr/bin/env fish
# .gg_hook.fish - Executed after \'gg add\' command in grove worktree directory
#
# Available environment variables:
# - $GG_WORKTREE_PATH : Path to the new grove worktree (current directory)
# - $GG_GROVE_NAME    : Name of the grove
# - $GG_BASE_REF      : Base commit/branch/tag used for creation
# - $GG_PROJECT_ROOT  : Path to the original project root
# - $GG_TIMESTAMP     : Timestamp of grove creation
# - $GG_IS_DETACHED   : "true" if detached HEAD, "false" if attached to branch

# Example: Show grove creation info
echo "[HOOK] Grove hook executing..."
echo "   Grove: $GG_GROVE_NAME (from $GG_BASE_REF)"
echo "   Location: $GG_WORKTREE_PATH"
echo "   Detached: $GG_IS_DETACHED"

# Files and directories to copy from project root
set -l copy_items \
    ".env" \
    ".env.local" \
    ".env.development" \
    ".claude" \
    "node_modules" \
    "vendor" \
    ".vscode" \
    ".idea"

# Copy items if they exist
for item in $copy_items
    set -l source "$GG_PROJECT_ROOT/$item"
    set -l target "$GG_WORKTREE_PATH/$item"
    
    if test -e "$source"
        # Skip if target already exists
        if test -e "$target"
            echo "       [SKIP] $item (already exists)"
            continue
        end
        
        # Determine copy method based on type and name
        if test -d "$source"
            switch $item
                case "node_modules" "vendor" ".git" "build" "dist" "target"
                    # Create symlink for large directories
                    ln -s "$source" "$target"
                    echo "       [LINK] $item"
                case \'*\'
                    # Copy directory
                    cp -r "$source" "$target"
                    echo "       [COPY] $item/"
            end
        else
            # Copy file
            cp "$source" "$target"
            echo "       [COPY] $item"
        end
    end
end

# Grove-specific initialization examples
# Uncomment and modify as needed:

# Install dependencies (if not linked)
# if not test -L "node_modules"
#     echo "[INSTALL] Installing dependencies..."
#     npm install
# end

# Run setup script
# if test -x "./scripts/setup.sh"
#     echo "[SETUP] Running setup script..."
#     ./scripts/setup.sh
# end

# Create grove-specific config
# echo "GROVE_NAME=$GG_GROVE_NAME" >> .env.local
# echo "BASE_REF=$GG_BASE_REF" >> .env.local

# Initialize development environment based on grove type
# if string match -q "*experiment*" "$GG_GROVE_NAME"
#     echo "[EXPERIMENT] Setting up experimental environment..."
#     # Add experimental flags or config
# else if string match -q "*hotfix*" "$GG_GROVE_NAME"
#     echo "[HOTFIX] Setting up hotfix environment..."
#     # Add production-like settings
# end

# Grove cleanup hooks (for when grove is removed)
# You can add cleanup logic in separate files:
# - .gg_cleanup.fish (executed before grove removal)
# - Use grove naming patterns for conditional behavior

echo "[OK] Grove hook completed successfully"' > .gg_hook.fish
    
    # Make the hook executable
    chmod +x .gg_hook.fish
    
    test "$quiet" = false; and echo "[OK] Created .gg_hook.fish template"
    test "$verbose" = true; and echo "Edit this file to customize grove initialization"
    
    # Add to .gitignore if not already there
    if test -f .gitignore
        if not grep -q "^\.gg_hook\.fish\$" .gitignore
            echo ".gg_hook.fish" >> .gitignore
            test "$quiet" = false; and echo "[OK] Added .gg_hook.fish to .gitignore"
        else
            test "$verbose" = true; and echo "[INFO] .gg_hook.fish already in .gitignore"
        end
    else
        # Create .gitignore if it doesn't exist
        echo ".gg_hook.fish" > .gitignore
        test "$quiet" = false; and echo "[OK] Created .gitignore and added .gg_hook.fish"
    end
end

function __gg_go
    # Parse arguments: -- actual_args... verbose quiet
    set -l actual_argv $argv[2..-3]
    set -l verbose $argv[-2]
    set -l quiet $argv[-1]
    
    # Parse go-specific options
    argparse 'h/help' -- $actual_argv
    or return 1
    
    # Handle help flag
    if set -ql _flag_help
        __gg_go_help
        return 0
    end
    
    # Get branch name from remaining arguments
    set -l target_branch $argv[1]
    
    # Get all worktrees
    set -l grove_worktrees (git worktree list 2>/dev/null)
    if test -z "$grove_worktrees"
        echo "Error: No Git repository found or no worktrees exist." >&2
        return 1
    end
    
    set -l target_worktree
    
    if test -n "$target_branch"
        # Branch specified - look for worktree with that branch
        test "$verbose" = true; and echo "Looking for worktree with branch '$target_branch'..."
        
        for worktree in $grove_worktrees
            set -l worktree_path (echo $worktree | string split -f1 ' ')
            set -l resolved_path (path resolve $worktree_path)
            
            # Skip bare repositories
            if string match -q "*(bare)" $worktree
                continue
            end
            
            # Check if directory exists and has the target branch
            if test -d "$resolved_path"
                set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                if test "$current_branch" = "$target_branch"
                    set target_worktree $worktree
                    test "$verbose" = true; and echo "Found worktree with branch '$target_branch': $resolved_path"
                    break
                end
            end
        end
        
        # If not found, try to checkout branch in first detached worktree
        if test -z "$target_worktree"
            test "$verbose" = true; and echo "Branch '$target_branch' not found in any worktree, looking for detached worktree..."
            
            for worktree in $grove_worktrees
                set -l worktree_path (echo $worktree | string split -f1 ' ')
                set -l resolved_path (path resolve $worktree_path)
                
                # Skip bare repositories
                if string match -q "*(bare)" $worktree
                    continue
                end
                
                # Check if this is a detached worktree
                if test -d "$resolved_path"
                    set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                    if test -z "$current_branch"
                        # This is detached - try to checkout the branch here
                        test "$quiet" = false; and echo "Checking out branch '$target_branch' in detached worktree..."
                        
                        # Clean up any stale locks before checkout operation
                        gg_cleanup_stale_locks "$verbose" >/dev/null
                        
                        if git -C "$resolved_path" checkout "$target_branch" &>/tmp/gg_go.log
                            set target_worktree $worktree
                            test "$quiet" = false; and echo "[OK] Checked out '$target_branch' in $resolved_path"
                            rm -f /tmp/gg_go.log
                            break
                        else
                            echo "Error: Failed to checkout branch '$target_branch'" >&2
                            test "$verbose" = true; and cat /tmp/gg_go.log >&2
                            rm -f /tmp/gg_go.log
                            # Clean up any locks that might have been left
                            gg_cleanup_stale_locks false >/dev/null 2>&1
                            return 1
                        end
                    end
                end
            end
            
            # If still no target, we need a detached worktree
            if test -z "$target_worktree"
                echo "Error: Branch '$target_branch' not found and no detached worktrees available" >&2
                echo "Use 'gg add' to create a detached worktree, or 'gg detach' to detach an existing one" >&2
                return 1
            end
        end
    else
        # No branch specified - find first detached worktree
        test "$verbose" = true; and echo "Looking for first detached worktree..."
        
        set -l has_attached_worktrees false
        
        for worktree in $grove_worktrees
            set -l worktree_path (echo $worktree | string split -f1 ' ')
            set -l resolved_path (path resolve $worktree_path)
            
            # Skip bare repositories
            if string match -q "*(bare)" $worktree
                continue
            end
            
            # Check if directory exists and has a working tree
            if test -d "$resolved_path"
                set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                if test -z "$current_branch"
                    # This is detached - use it!
                    set target_worktree $worktree
                    test "$verbose" = true; and echo "Found detached worktree: $resolved_path"
                    break
                else
                    # This is attached - note it for error message
                    set has_attached_worktrees true
                end
            end
        end
        
        # Fail if no detached worktree found
        if test -z "$target_worktree"
            if test "$has_attached_worktrees" = true
                echo "Error: No detached worktrees found (only attached worktrees exist)" >&2
                echo "Use 'gg detach' to detach a worktree, or use 'gg' for interactive selection" >&2
            else
                echo "Error: No valid worktrees found" >&2
            end
            return 1
        end
    end
    
    # Extract path from the target worktree
    set -l worktree_path (echo $target_worktree | string split -f1 ' ')
    set -l resolved_path (path resolve $worktree_path)
    
    if test -d "$resolved_path"
        cd "$resolved_path"
        
        # Determine if this is a grove or regular worktree for the message
        set -l worktree_type "worktree"
        set -l grove_name (basename $resolved_path)
        
        # Check if this is a grove worktree using the new detection logic
        # We need repo info for this check
        set -l repo_info (gg_get_repo_info)
        if test $status -eq 0
            set -l parts (string split "|" $repo_info)
            set -l git_dir_resolved $parts[1]
            set -l repo_root $parts[2]
            set -l is_bare $parts[3]
            
            # Determine expected parent directory for grove worktrees
            set -l expected_parent (path resolve "$git_dir_resolved")
            
            set -l worktree_name (basename $resolved_path)
            set -l worktree_parent (dirname $resolved_path)
            
            if string match -q "wt*" $worktree_name; and test "$worktree_parent" = "$expected_parent"
                set worktree_type "grove"
                set grove_name $worktree_name
            end
        end
        
        test "$quiet" = false; and echo "Switched to: $resolved_path"
        test "$verbose" = true; and echo "Type: $worktree_type ($grove_name)"
        return 0
    else
        echo "Error: Directory not found: $resolved_path" >&2
        return 1
    end
end

