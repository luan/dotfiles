function bd-init --description "Initialize beads with dolt backend in stealth mode"
    argparse 'p/prefix=' h/help -- $argv
    or return 1

    if set -ql _flag_help
        echo "bd-init - Initialize beads (dolt, stealth, no hooks)"
        echo ""
        echo "USAGE:"
        echo "  bd-init [-p prefix]"
        echo ""
        echo "OPTIONS:"
        echo "  -p, --prefix <name>   Issue prefix (default: directory name)"
        echo ""
        echo "Runs bd init --backend dolt --stealth --skip-hooks and"
        echo "applies the issue_prefix workaround for dolt backend."
        return 0
    end

    if not command -sq bd
        echo "Error: bd is not installed" >&2
        return 1
    end

    if not git rev-parse --git-dir &>/dev/null
        echo "Error: not a git repository" >&2
        return 1
    end

    if test -d .beads
        echo "Error: .beads already exists" >&2
        return 1
    end

    set -l prefix
    if set -ql _flag_prefix
        set prefix $_flag_prefix
    else
        set prefix (basename $PWD)
    end

    echo n | bd init --backend dolt --stealth --prefix $prefix --skip-hooks
    or return 1

    bd config set issue_prefix $prefix
    or return 1

    echo ""
    echo "Ready. Try: bd create \"My first issue\" -p 1 --description \"...\""
end
