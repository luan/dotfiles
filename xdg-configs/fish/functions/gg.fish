# Git Grove - Manage git worktrees as a pool of detached workspaces

function gg_get_repo_info --description "Get repository information for bare and regular repos"
    set -l git_common_dir (git rev-parse --git-common-dir 2>/dev/null)
    test -z "$git_common_dir"; and return 1

    set -l git_dir_resolved (path resolve $git_common_dir)
    set -l is_bare (git rev-parse --is-bare-repository 2>/dev/null)
    set -l repo_root

    if test "$is_bare" = true
        set repo_root $git_dir_resolved
    else
        set repo_root (git rev-parse --show-toplevel 2>/dev/null)
        test -z "$repo_root"; and return 1
    end

    echo "$git_dir_resolved|$repo_root|$is_bare"
end

function __gg_is_grove --argument-names worktree_path git_dir
    set -l parent (dirname $worktree_path)
    set -l expected (path resolve $git_dir)

    # Fast path: parent must be the git common dir
    test "$parent" = "$expected"; or return 1

    # Verify path is actually a registered worktree (not a random subdir)
    set -l resolved (path resolve $worktree_path)
    for wt_line in (git worktree list --porcelain 2>/dev/null)
        set -l wt_path (string replace -r '^worktree ' '' -- $wt_line)
        test "$wt_path" = "$wt_line"; and continue
        if test (path resolve $wt_path) = "$resolved"
            return 0
        end
    end
    return 1
end

function __gg_cleanup_stale_lock --argument-names lock_path
    # Only clean truly stale locks (>60 seconds old)
    # This avoids racing with active git operations
    test -f "$lock_path"; or return 1

    set -l file_time (stat -f %m "$lock_path" 2>/dev/null)
    test -z "$file_time"; and return 1

    set -l age (math (date +%s) - $file_time)
    if test $age -gt 60
        rm -f "$lock_path" 2>/dev/null
        return 0
    end
    return 1
end

function __gg_try_cleanup_locks --argument-names worktree_path git_dir
    # Clean stale locks for a worktree before operating on it
    __gg_cleanup_stale_lock "$worktree_path/index.lock"
    __gg_cleanup_stale_lock "$worktree_path/.git/index.lock"
    test -n "$git_dir"; and __gg_cleanup_stale_lock "$git_dir/index.lock"
end

function __gg_has_claude_history --argument-names worktree_path
    # Claude sanitizes project paths: /a/b/c -> -a-b-c
    set -l sanitized (string replace -a / - $worktree_path)
    set -l project_dir "$HOME/.claude/projects/$sanitized"

    test -d "$project_dir"; or return 1

    # Check for any .jsonl session files
    set -l session_files $project_dir/*.jsonl
    test -n "$session_files"; and test -e "$session_files[1]"
end

function __gg_claude_cmd --argument-names worktree_path
    if __gg_has_claude_history "$worktree_path"
        echo "claude --continue"
    else
        echo "claude"
    end
end

function __gg_tmux_session --argument-names grove_name worktree_path
    # Derive session name from repo basename + grove name
    # e.g., for /Users/me/src/arc.git/wt3, repo is "arc", grove is "wt3" -> "arc/wt3"
    set -l git_common_dir (git -C "$worktree_path" rev-parse --git-common-dir 2>/dev/null)
    set -l repo_basename
    if test -n "$git_common_dir"
        set repo_basename (basename (path resolve "$git_common_dir") | string replace -r '\.git$' '')
    else
        set repo_basename (basename "$worktree_path")
    end

    set -l session_name "$repo_basename/$grove_name"
    # tmux session names can't contain dots or colons
    set session_name (string replace -a '.' '_' "$session_name")
    set session_name (string replace -a ':' '_' "$session_name")

    # If session exists, just switch to it
    if tmux has-session -t "$session_name" 2>/dev/null
        if test -n "$TMUX"
            tmux switch-client -t "$session_name"
        else
            tmux attach -t "$session_name"
        end
        return 0
    end

    # Create session with 3 windows: ai, vi, sh
    tmux new-session -d -s "$session_name" -n "ai" -c "$worktree_path"
    tmux new-window -t "$session_name" -n "vi" -c "$worktree_path"
    tmux new-window -t "$session_name" -n "sh" -c "$worktree_path"

    # Launch claude and nvim via send-keys (matches new-project-session.sh)
    set -l claude_cmd (__gg_claude_cmd "$worktree_path")
    tmux send-keys -t "$session_name:ai" "$claude_cmd" Enter
    tmux send-keys -t "$session_name:vi" "nvim" Enter

    # Focus the ai window
    tmux select-window -t "$session_name:ai"

    # Switch or attach
    if test -n "$TMUX"
        tmux switch-client -t "$session_name"
    else
        tmux attach -t "$session_name"
    end
end

function gg_generate_name --description "Generate next available grove name"
    set -l repo_info (gg_get_repo_info)
    if test $status -ne 0
        echo wt1
        return
    end

    set -l git_dir_resolved (string split "|" $repo_info)[1]
    set -l expected_parent (path resolve "$git_dir_resolved")
    set -l highest_num 0

    if test -d "$expected_parent"
        for dir in "$expected_parent"/*
            if test -d "$dir"
                set -l grove_name (basename "$dir")
                if string match -qr '^wt\d+$' "$grove_name"
                    set -l num (string replace 'wt' '' "$grove_name")
                    test "$num" -gt "$highest_num"; and set highest_num "$num"
                end
            end
        end
    end

    echo "wt"(math "$highest_num + 1")
end

function __gg_help
    echo "Git Grove - Manage git worktrees as a pool of detached workspaces"
    echo ---
    echo ""
    echo "USAGE:"
    echo "  gg [options]                     - Interactive worktree selection with fzf"
    echo "  gg add [name] [options]          - Create new detached worktree"
    echo "  gg remove <name> [options]       - Remove detached worktree"
    echo "  gg list [options]                - List all worktrees"
    echo "  gg detach [name] [options]       - Detach worktree from branch (also: gg d)"
    echo "  gg go [branch] [options]         - Smart branch switcher"
    echo "  gg pool                          - Show pool status overview"
    echo "  gg init                          - Create .gg_hook.fish template"
    echo ""
    echo "GLOBAL OPTIONS:"
    echo "  -h, --help                       - Show this help message"
    echo "  -v, --verbose                    - Enable verbose output"
    echo "  -q, --quiet                      - Suppress informational output"
    echo "  --tmux                           - Create tmux session (auto when in tmux)"
    echo "  --no-tmux                        - Don't create tmux session"
    echo ""
    echo "EXAMPLES:"
    echo "  gg                               - Select worktree interactively"
    echo "  gg add                           - Create wt1 (or next available)"
    echo "  gg add hotfix --from v1.2.3      - Create named grove from tag"
    echo "  gg go feature/new                - Find or checkout feature/new branch"
end

function __gg_add_help
    echo "gg add - Create new detached worktree"
    echo ---
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
end

function __gg_remove_help
    echo "gg remove - Remove grove worktree"
    echo ---
    echo ""
    echo "USAGE:"
    echo "  gg remove [<name>] [options]"
    echo ""
    echo "BEHAVIOR:"
    echo "  If the grove is attached to a branch, it will be auto-detached"
    echo "  before removal. Confirmation prompt shows the branch name."
end

function __gg_detach_help
    echo "gg detach - Detach worktree from branch"
    echo ---
    echo ""
    echo "USAGE:"
    echo "  gg detach [grove_name] [options]"
    echo ""
    echo "BEHAVIOR:"
    echo "  Detaches HEAD from the current branch."
    echo "  Makes the worktree safe for removal with 'gg remove'."
end

function __gg_pool_help
    echo "gg pool - Show pool status overview"
    echo ---
    echo ""
    echo "USAGE:"
    echo "  gg pool [options]"
    echo ""
    echo "DESCRIPTION:"
    echo "  Display statistics about the grove worktree pool."
end

function __gg_init_help
    echo "gg init - Create .gg_hook.fish template"
    echo ---
    echo ""
    echo "DESCRIPTION:"
    echo "  Creates a .gg_hook.fish template for post-create setup."
end

function __gg_go_help
    echo "gg go - Smart branch switcher"
    echo ---
    echo ""
    echo "USAGE:"
    echo "  gg go [branch_name]"
    echo ""
    echo "DESCRIPTION:"
    echo "  With branch: finds worktree with that branch or checks it out."
    echo "  Without branch: switches to first detached worktree."
end

function __gg_worktree_mtime --argument-names path --description "Get directory mtime as epoch seconds"
    stat -f %m "$path" 2>/dev/null; or echo 0
end

function __gg_format_relative_time --argument-names epoch --description "Format epoch as relative time string"
    set -l now (date +%s)
    set -l diff (math $now - $epoch)

    if test $diff -lt 60
        echo "just now"
    else if test $diff -lt 3600
        echo (math "floor($diff / 60)")"m ago"
    else if test $diff -lt 86400
        echo (math "floor($diff / 3600)")"h ago"
    else if test $diff -lt 604800
        echo (math "floor($diff / 86400)")"d ago"
    else
        echo (math "floor($diff / 604800)")"w ago"
    end
end

function __gg_interactive --argument-names verbose quiet use_tmux
    set -e argv[1..3]
    if not command -sq fzf
        echo "Error: fzf is not installed" >&2
        return 1
    end

    set -l repo_info (gg_get_repo_info)
    test $status -ne 0; and echo "Error: Not in a Git repository" >&2; and return 1

    set -l parts (string split "|" $repo_info)
    set -l git_dir_resolved $parts[1]

    set -l grove_worktrees (git worktree list 2>/dev/null)
    test -z "$grove_worktrees"; and echo "Error: No worktrees exist" >&2; and return 1

    # Build items with mtime prefix for sorting (decorate-sort-undecorate)
    set -l decorated_items
    for worktree in $grove_worktrees
        set -l path (string split -f1 ' ' $worktree)
        set -l resolved_path (path resolve $path)

        string match -q "*(bare)" $worktree; and continue

        set -l worktree_name (basename $resolved_path)
        set -l head_state

        if __gg_is_grove "$resolved_path" "$git_dir_resolved"
            set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
            if test -z "$current_branch"
                set head_state "🔸 detached"
            else
                set head_state "🔗 $current_branch"
            end
        else
            set -l branch_match (string match -r '\[([^\]]+)\]' $worktree)
            if test -n "$branch_match"
                set -l branch (string replace -r '.*\[([^\]]+)\].*' '$1' $branch_match)
                set head_state "🔗 $branch"
            else
                set head_state "🔸 detached"
            end
        end

        set -l mtime (__gg_worktree_mtime "$resolved_path")
        set -l relative_time (__gg_format_relative_time $mtime)
        set -a decorated_items "$mtime|$worktree_name|$head_state|$resolved_path|$relative_time"
    end

    # Sort by mtime descending, then strip mtime prefix
    set -l grove_items
    for item in (printf '%s\n' $decorated_items | sort -t'|' -k1 -nr)
        set -l fields (string split "|" $item)
        set -a grove_items "$fields[2]|$fields[3]|$fields[4]|$fields[5]"
    end

    set -l selected_item (printf '%s\n' $grove_items | fzf \
        --delimiter='|' \
        --with-nth=1,2,4 \
        --preview-window="right:70%:wrap" \
        --preview='
            set -l parts (string split "|" {})
            set -l grove_name $parts[1]
            set -l head_state $parts[2]
            set -l worktree_path $parts[3]
            set -l relative_time $parts[4]

            echo "Grove: $grove_name"
            echo "State: $head_state"
            echo "Path:  $worktree_path"
            echo "Last:  $relative_time"
            echo ""
            echo "Changes:"
            GIT_OPTIONAL_LOCKS=0 git -C "$worktree_path" status --porcelain 2>/dev/null | head -10
            echo ""
            echo "Recent Commits:"
            GIT_OPTIONAL_LOCKS=0 git -C "$worktree_path" log --oneline -5 2>/dev/null
        ' \
        --header="Git Grove    ⏎ Select  ^C Cancel" \
        --border=rounded \
        --height=80% \
        --layout=reverse \
        --prompt="grove › ")

    if test -n "$selected_item"
        set -l parts (string split "|" $selected_item)
        set -l worktree_path $parts[3]

        if test -d "$worktree_path"
            __gg_try_cleanup_locks "$worktree_path" "$git_dir_resolved"
            if test "$use_tmux" = true
                set -l grove_name (basename "$worktree_path")
                __gg_tmux_session "$grove_name" "$worktree_path"
            else
                cd "$worktree_path"
            end
            test "$verbose" = true; and echo "Switched to: $worktree_path"
            return 0
        else
            echo "Error: Directory not found: $worktree_path" >&2
            return 1
        end
    end
end

function __gg_add --argument-names verbose quiet use_tmux
    set -e argv[1..3]
    argparse 'from=' 'b/branch=' sync no-hook h/help -- $argv
    or return 1

    set -ql _flag_help; and __gg_add_help; and return 0

    set -l grove_name $argv[1]
    if test -z "$grove_name"
        set grove_name (gg_generate_name)
        test "$verbose" = true; and echo "[INFO] Auto-generated name: $grove_name"
    else
        if not string match -qr '^[a-zA-Z0-9._-]+$' "$grove_name"
            echo "Error: Invalid grove name" >&2
            return 1
        end
    end

    set -l original_dir $PWD
    set -l repo_info (gg_get_repo_info)
    test $status -ne 0; and echo "Error: Not in a Git repository" >&2; and return 1

    set -l parts (string split "|" $repo_info)
    set -l git_dir_resolved $parts[1]
    set -l repo_root $parts[2]
    set -l is_bare $parts[3]

    test "$is_bare" != true; and cd "$repo_root"

    set -l worktree_path "$git_dir_resolved/$grove_name"

    if test -d "$worktree_path"
        echo "Error: Grove already exists: $worktree_path" >&2
        return 1
    end

    set -l base_ref
    if set -ql _flag_from
        set base_ref $_flag_from
        if not git rev-parse --verify "$base_ref" &>/dev/null
            echo "Error: Reference '$base_ref' does not exist" >&2
            return 1
        end
    else
        set base_ref main
        if not git rev-parse --verify main &>/dev/null
            if git rev-parse --verify master &>/dev/null
                set base_ref master
            else
                set base_ref HEAD
            end
        end
    end

    test "$quiet" = false; and echo "Creating grove '$grove_name' from $base_ref..."

    if set -ql _flag_branch
        set -l branch_name $_flag_branch
        if not git worktree add -b "$branch_name" "$worktree_path" "$base_ref"
            echo "Error: Failed to create grove" >&2
            return 1
        end
        test "$quiet" = false; and echo "[OK] Created grove with branch '$branch_name'"
    else
        if not git worktree add --detach "$worktree_path" "$base_ref"
            echo "Error: Failed to create grove" >&2
            return 1
        end
        test "$quiet" = false; and echo "[OK] Created detached grove"
    end

    if set -ql _flag_sync; and test "$is_bare" != true
        test "$quiet" = false; and echo "[SYNC] Syncing changes..."
        for file in (git diff --cached --name-only) (git diff --name-only) (git ls-files --others --exclude-standard)
            if test -f "$repo_root/$file"
                set -l dir_path (dirname "$worktree_path/$file")
                mkdir -p "$dir_path"
                cp "$repo_root/$file" "$worktree_path/$file"
            end
        end
    end

    cd "$worktree_path"

    if not set -ql _flag_no_hook; and test -f "$repo_root/.gg_hook.fish"
        test "$quiet" = false; and echo "[HOOK] Executing .gg_hook.fish..."
        set -gx GG_WORKTREE_PATH "$worktree_path"
        set -gx GG_GROVE_NAME "$grove_name"
        set -gx GG_BASE_REF "$base_ref"
        set -gx GG_PROJECT_ROOT "$repo_root"
        set -gx GG_TIMESTAMP (date +"%Y-%m-%d %H:%M:%S")
        set -gx GG_IS_DETACHED (set -ql _flag_branch; and echo "false"; or echo "true")

        source "$repo_root/.gg_hook.fish"

        set -e GG_WORKTREE_PATH GG_GROVE_NAME GG_BASE_REF GG_PROJECT_ROOT GG_TIMESTAMP GG_IS_DETACHED
    end

    if test "$use_tmux" = true
        __gg_tmux_session "$grove_name" "$worktree_path"
    end

    test "$quiet" = false; and echo "[PWD] Now in: $worktree_path"
end

function __gg_remove --argument-names verbose quiet
    set -e argv[1..2]
    argparse h/help -- $argv
    or return 1

    set -ql _flag_help; and __gg_remove_help; and return 0

    set -l grove_name $argv[1]

    set -l repo_info (gg_get_repo_info)
    test $status -ne 0; and echo "Error: Not in a Git repository" >&2; and return 1

    set -l git_dir_resolved (string split '|' $repo_info)[1]
    set -l expected_parent (path resolve "$git_dir_resolved")

    if test -z "$grove_name"
        if not command -sq fzf
            echo "Error: Grove name required or install fzf" >&2
            return 1
        end

        set -l grove_items
        for worktree in (git worktree list 2>/dev/null)
            string match -q "*(bare)" $worktree; and continue

            set -l path (string split -f1 ' ' $worktree)
            set -l resolved_path (path resolve $path)

            if __gg_is_grove "$resolved_path" "$git_dir_resolved"
                set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                set -l label (basename $resolved_path)
                if test -n "$current_branch"
                    set label "$label (🔗 $current_branch)"
                else
                    set label "$label (🔸 detached)"
                end
                set -a grove_items "$label|$resolved_path"
            end
        end

        test -z "$grove_items"; and echo "No groves found" >&2; and return 1

        set -l selected (printf '%s\n' $grove_items | fzf --delimiter='|' --with-nth=1 --header="Select grove to remove")
        test -z "$selected"; and return 0

        set -l selected_path (string split '|' $selected)[2]
        set grove_name (basename $selected_path)
    end

    set -l worktree_path
    for worktree in (git worktree list 2>/dev/null)
        string match -q "*(bare)" $worktree; and continue

        set -l path (string split ' ' $worktree)[1]
        set -l resolved_path (path resolve $path)
        set -l name (basename $resolved_path)

        if __gg_is_grove "$resolved_path" "$git_dir_resolved"; and test "$name" = "$grove_name"
            set worktree_path "$resolved_path"
            break
        end
    end

    test -z "$worktree_path"; and echo "Error: Grove '$grove_name' not found" >&2; and return 1

    set -l current_branch (git -C "$worktree_path" branch --show-current 2>/dev/null)
    if test -n "$current_branch"
        echo "Remove grove '$grove_name' (branch: $current_branch)?"
        read -l -P "This will detach from '$current_branch' and remove. Confirm (y/N) " confirm
        string match -qi y $confirm; or return 0

        if not git -C "$worktree_path" checkout --detach 2>/dev/null
            echo "Error: Failed to detach grove '$grove_name' from '$current_branch'" >&2
            return 1
        end
        test "$quiet" = false; and echo "[OK] Detached from '$current_branch'"
    else
        echo "Remove grove '$grove_name' at $worktree_path?"
        read -l -P "Confirm (y/N) " confirm
        string match -qi y $confirm; or return 0
    end

    if git worktree remove --force "$worktree_path"
        test "$quiet" = false; and echo "[OK] Removed grove '$grove_name'"
    else
        echo "Error: Failed to remove grove" >&2
        return 1
    end
end

function __gg_list --argument-names verbose quiet
    set -e argv[1..2]
    argparse h/help -- $argv
    or return 1

    if set -ql _flag_help
        echo "gg list - List all worktrees"
        return 0
    end

    set -l repo_info (gg_get_repo_info)
    test $status -ne 0; and echo "Error: Not in a Git repository" >&2; and return 1

    set -l git_dir_resolved (string split "|" $repo_info)[1]
    set -l grove_worktrees (git worktree list 2>/dev/null)

    test "$quiet" = false; and echo "Git Grove - Worktree List"
    test "$quiet" = false; and echo ---
    test "$quiet" = false; and echo ""

    set -l grove_count 0
    set -l detached_count 0

    # Collect worktree data with mtime for sorting
    set -l decorated_entries
    for worktree in $grove_worktrees
        string match -q "*(bare)" $worktree; and continue

        set -l path (string split -f1 ' ' $worktree)
        set -l resolved_path (path resolve $path)
        set -l mtime (__gg_worktree_mtime "$resolved_path")

        set -a decorated_entries "$mtime\t$worktree\t$resolved_path"
    end

    # Sort by mtime descending
    set -l sorted_entries (printf '%s\n' $decorated_entries | sort -t\t -k1 -nr)

    for entry in $sorted_entries
        set -l entry_parts (string split \t $entry)
        set -l mtime $entry_parts[1]
        set -l worktree $entry_parts[2]
        set -l resolved_path $entry_parts[3]

        set -l worktree_name (basename $resolved_path)
        set -l is_grove (__gg_is_grove "$resolved_path" "$git_dir_resolved"; and echo true; or echo false)

        set -l head_state
        set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
        if test -z "$current_branch"
            set head_state "🔸 detached"
            set detached_count (math $detached_count + 1)
        else
            set head_state "🔗 $current_branch"
        end

        test "$is_grove" = true; and set grove_count (math $grove_count + 1)

        set -l changes (git -C "$resolved_path" status --porcelain 2>/dev/null | wc -l | string trim)
        set -l status_text (test "$changes" = "0"; and echo "clean"; or echo "$changes changes")
        set -l last_commit (git -C "$resolved_path" log -1 --format="%h %s" 2>/dev/null)
        set -l relative_time (__gg_format_relative_time $mtime)

        if test "$is_grove" = true
            echo "Grove: $worktree_name"
        else
            echo "Worktree: $worktree_name"
        end
        echo "  State: $head_state"
        echo "  Path: $resolved_path"
        echo "  Status: $status_text"
        echo "  Last commit: $last_commit"
        echo "  Last used: $relative_time"
        echo ""
    end

    set -l total (count $grove_worktrees)
    echo ---
    echo "Summary: $total total ($grove_count grove) • $detached_count detached"
end

function __gg_detach --argument-names verbose quiet
    set -e argv[1..2]
    argparse h/help -- $argv
    or return 1

    set -ql _flag_help; and __gg_detach_help; and return 0

    set -l grove_name $argv[1]
    set -l current_dir $PWD

    set -l repo_info (gg_get_repo_info)
    test -z "$repo_info"; and echo "Error: Not in a Git repository" >&2; and return 1

    set -l git_dir_resolved (string split '|' $repo_info)[1]

    if test -z "$grove_name"
        if __gg_is_grove "$current_dir" "$git_dir_resolved"
            set grove_name (basename $current_dir)
        else
            set -l current_branch (git branch --show-current 2>/dev/null)
            if test -n "$current_branch"
                test "$quiet" = false; and echo "Detaching from '$current_branch'..."
                if git checkout --detach
                    test "$quiet" = false; and echo "[OK] Detached"
                    return 0
                else
                    echo "Error: Failed to detach" >&2
                    return 1
                end
            else
                echo "Error: Already detached" >&2
                return 1
            end
        end
    end

    set -l worktree_path
    for worktree in (git worktree list 2>/dev/null)
        string match -q "*(bare)" $worktree; and continue

        set -l path (string split ' ' $worktree)[1]
        set -l resolved_path (path resolve $path)
        set -l name (basename $resolved_path)

        if __gg_is_grove "$resolved_path" "$git_dir_resolved"; and test "$name" = "$grove_name"
            set worktree_path "$resolved_path"
            break
        end
    end

    test -z "$worktree_path"; and echo "Error: Grove '$grove_name' not found" >&2; and return 1

    set -l current_branch (git -C "$worktree_path" branch --show-current 2>/dev/null)
    test -z "$current_branch"; and echo "Error: Grove '$grove_name' is already detached" >&2; and return 1

    test "$quiet" = false; and echo "Detaching grove '$grove_name' from '$current_branch'..."

    set -l original_dir $PWD
    cd "$worktree_path"

    if git checkout --detach
        test "$quiet" = false; and echo "[OK] Detached from '$current_branch'"
    else
        echo "Error: Failed to detach" >&2
        cd "$original_dir"
        return 1
    end

    cd "$original_dir"
end

function __gg_pool --argument-names verbose quiet
    set -e argv[1..2]
    argparse h/help -- $argv
    or return 1

    set -ql _flag_help; and __gg_pool_help; and return 0

    set -l repo_info (gg_get_repo_info)
    test -z "$repo_info"; and echo "Error: Not in a Git repository" >&2; and return 1

    set -l git_dir_resolved (string split '|' $repo_info)[1]
    set -l grove_worktrees (git worktree list 2>/dev/null)

    test "$quiet" = false; and echo "Git Grove - Pool Status"
    test "$quiet" = false; and echo ---
    test "$quiet" = false; and echo ""

    set -l total 0
    set -l grove_count 0
    set -l detached_count 0
    set -l grove_disk_usage 0

    for worktree in $grove_worktrees
        string match -q "*(bare)" $worktree; and continue
        set total (math $total + 1)

        set -l path (string split ' ' $worktree)[1]
        set -l resolved_path (path resolve $path)

        if __gg_is_grove "$resolved_path" "$git_dir_resolved"
            set grove_count (math $grove_count + 1)
            set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
            test -z "$current_branch"; and set detached_count (math $detached_count + 1)

            set -l dir_size (du -sk "$resolved_path" 2>/dev/null | string split \t -f1)
            test -n "$dir_size"; and set grove_disk_usage (math "$grove_disk_usage + $dir_size")
        end
    end

    set -l regular_count (math "$total - $grove_count")
    set -l disk_mb (math "round($grove_disk_usage / 1024)")

    echo "Worktrees:  $total total ($grove_count grove, $regular_count regular)"
    echo "Detached:   $detached_count grove worktrees"
    echo "Disk Usage: ~$disk_mb MB"

    if test "$verbose" = true -a $grove_count -gt 0
        echo ""
        echo "Grove Details:"
        for worktree in $grove_worktrees
            string match -q "*(bare)" $worktree; and continue

            set -l path (string split ' ' $worktree)[1]
            set -l resolved_path (path resolve $path)

            if __gg_is_grove "$resolved_path" "$git_dir_resolved"
                set -l name (basename $resolved_path)
                set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                set -l state (test -z "$current_branch"; and echo "🔸"; or echo "🔗 $current_branch")
                echo "  - $name $state"
            end
        end
    end
end

function __gg_init --argument-names verbose quiet
    set -e argv[1..2]
    argparse h/help -- $argv
    or return 1

    set -ql _flag_help; and __gg_init_help; and return 0

    if test -f ".gg_hook.fish"
        echo "Error: .gg_hook.fish already exists" >&2
        return 1
    end

    echo '#!/usr/bin/env fish
# .gg_hook.fish - Executed after gg add in grove worktree
#
# Available variables:
# - $GG_WORKTREE_PATH : Path to the grove worktree
# - $GG_GROVE_NAME    : Name of the grove
# - $GG_BASE_REF      : Base commit/branch/tag
# - $GG_PROJECT_ROOT  : Original project root
# - $GG_TIMESTAMP     : Creation timestamp
# - $GG_IS_DETACHED   : "true" if detached

echo "[HOOK] Grove: $GG_GROVE_NAME"

set -l copy_items .env .env.local .claude .vscode

for item in $copy_items
    set -l source "$GG_PROJECT_ROOT/$item"
    set -l target "$GG_WORKTREE_PATH/$item"

    if test -e "$source"; and not test -e "$target"
        if test -d "$source"
            switch $item
                case node_modules vendor
                    ln -s "$source" "$target"
                    echo "[LINK] $item"
                case "*"
                    cp -r "$source" "$target"
                    echo "[COPY] $item/"
            end
        else
            cp "$source" "$target"
            echo "[COPY] $item"
        end
    end
end

echo "[OK] Hook completed"' >.gg_hook.fish

    chmod +x .gg_hook.fish
    test "$quiet" = false; and echo "[OK] Created .gg_hook.fish"

    if test -f .gitignore
        if not grep -q '^\\.gg_hook\\.fish$' .gitignore
            echo ".gg_hook.fish" >>.gitignore
            test "$quiet" = false; and echo "[OK] Added to .gitignore"
        end
    else
        echo ".gg_hook.fish" >.gitignore
        test "$quiet" = false; and echo "[OK] Created .gitignore"
    end
end

function __gg_go --argument-names verbose quiet use_tmux
    set -e argv[1..3]
    argparse h/help -- $argv
    or return 1

    set -ql _flag_help; and __gg_go_help; and return 0

    set -l target_branch $argv[1]
    set -l grove_worktrees (git worktree list 2>/dev/null)

    test -z "$grove_worktrees"; and echo "Error: No worktrees exist" >&2; and return 1

    # Get git dir for lock cleanup
    set -l repo_info (gg_get_repo_info)
    set -l git_dir (test -n "$repo_info"; and string split '|' $repo_info)[1]

    set -l target_worktree

    if test -n "$target_branch"
        for worktree in $grove_worktrees
            string match -q "*(bare)" $worktree; and continue

            set -l worktree_path (string split -f1 ' ' $worktree)
            set -l resolved_path (path resolve $worktree_path)

            if test -d "$resolved_path"
                set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                if test "$current_branch" = "$target_branch"
                    set target_worktree "$resolved_path"
                    break
                end
            end
        end

        if test -z "$target_worktree"
            for worktree in $grove_worktrees
                string match -q "*(bare)" $worktree; and continue

                set -l worktree_path (string split -f1 ' ' $worktree)
                set -l resolved_path (path resolve $worktree_path)

                if test -d "$resolved_path"
                    set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                    if test -z "$current_branch"
                        # Clean stale locks before checkout
                        __gg_try_cleanup_locks "$resolved_path" "$git_dir"
                        test "$quiet" = false; and echo "Checking out '$target_branch'..."
                        if git -C "$resolved_path" checkout "$target_branch"
                            set target_worktree "$resolved_path"
                            break
                        else
                            echo "Error: Failed to checkout '$target_branch'" >&2
                            return 1
                        end
                    end
                end
            end
        end

        test -z "$target_worktree"; and echo "Error: No worktree available for '$target_branch'" >&2; and return 1
    else
        for worktree in $grove_worktrees
            string match -q "*(bare)" $worktree; and continue

            set -l worktree_path (string split -f1 ' ' $worktree)
            set -l resolved_path (path resolve $worktree_path)

            if test -d "$resolved_path"
                set -l current_branch (git -C "$resolved_path" branch --show-current 2>/dev/null)
                if test -z "$current_branch"
                    set target_worktree "$resolved_path"
                    break
                end
            end
        end

        test -z "$target_worktree"; and echo "Error: No detached worktrees found" >&2; and return 1
    end

    if test "$use_tmux" = true
        set -l grove_name (basename "$target_worktree")
        __gg_tmux_session "$grove_name" "$target_worktree"
    else
        cd "$target_worktree"
    end
    test "$quiet" = false; and echo "Switched to: $target_worktree"
end

function gg --description "Git Grove - Manage git worktrees"
    argparse -s h/help v/verbose q/quiet tmux no-tmux -- $argv
    or return 1

    set -ql _flag_help; and __gg_help; and return 0

    set -l verbose (set -ql _flag_verbose; and echo true; or echo false)
    set -l quiet (set -ql _flag_quiet; and echo true; or echo false)

    # Determine tmux mode: explicit flag > auto-detect from $TMUX
    set -l use_tmux false
    if set -ql _flag_tmux
        set use_tmux true
    else if set -ql _flag_no_tmux
        set use_tmux false
    else if test -n "$TMUX"
        set use_tmux true
    end

    set -l cmd $argv[1]
    set -e argv[1]

    switch "$cmd"
        case ""
            __gg_interactive $verbose $quiet $use_tmux
        case add
            __gg_add $verbose $quiet $use_tmux $argv
        case remove rm
            __gg_remove $verbose $quiet $argv
        case list ls
            __gg_list $verbose $quiet $argv
        case d detach
            __gg_detach $verbose $quiet $argv
        case pool
            __gg_pool $verbose $quiet $argv
        case init
            __gg_init $verbose $quiet $argv
        case go
            __gg_go $verbose $quiet $use_tmux $argv
        case help
            __gg_help
        case '*'
            echo "Error: Unknown command '$cmd'" >&2
            echo "Run 'gg --help' for usage" >&2
            return 1
    end
end
