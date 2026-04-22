# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_ct_global_optspecs
	string join \n h/help
end

function __fish_ct_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_ct_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_ct_using_subcommand
	set -l cmd (__fish_ct_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c ct -n "__fish_ct_needs_command" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_needs_command" -f -a "plan" -d 'Plan file operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "p" -d 'Plan file operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "spec" -d 'Spec file operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "s" -d 'Spec file operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "review" -d 'Review file operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "r" -d 'Review file operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "report" -d 'Report file operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "rp" -d 'Report file operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "doc" -d 'Doc file operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "d" -d 'Doc file operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "vault" -d 'Vault repository management'
complete -c ct -n "__fish_ct_needs_command" -f -a "v" -d 'Vault repository management'
complete -c ct -n "__fish_ct_needs_command" -f -a "read" -d 'Read artifact by stem (resolves across all types)'
complete -c ct -n "__fish_ct_needs_command" -f -a "notify" -d 'Handle notification hooks'
complete -c ct -n "__fish_ct_needs_command" -f -a "n" -d 'Handle notification hooks'
complete -c ct -n "__fish_ct_needs_command" -f -a "tool" -d 'Utility tools'
complete -c ct -n "__fish_ct_needs_command" -f -a "o" -d 'Utility tools'
complete -c ct -n "__fish_ct_needs_command" -f -a "sym" -d 'Code indexing and symbol discovery'
complete -c ct -n "__fish_ct_needs_command" -f -a "mcp" -d 'Run the MCP stdio server'
complete -c ct -n "__fish_ct_needs_command" -f -a "apply-patch" -d 'Inspect or prune apply_patch telemetry'
complete -c ct -n "__fish_ct_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from list" -l all -d 'Show artifacts from all projects'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from list" -l archived -d 'Show archived artifacts instead of active'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from list" -l include-dives -d 'Also show dive/ files (spec only)'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from create" -l topic -d 'Artifact topic' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from create" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from create" -l source -d 'Source artifact stem for [[wiki-link]]' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from create" -l tags -d 'Comma-separated tags (e.g. domain/combat,stage/research)' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from create" -l dive -d 'Route to dive/ instead of spec/ (requires --source; spec only)'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from latest" -l include-dives -d 'Also scan dive/ files when finding latest (spec only)'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from archive" -l batch -d 'Batch archive multiple files' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from archive" -l dry-run -d 'Preview what would be archived without acting'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from prune" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be archived'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from comments" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from comments" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from retag" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from list" -l all -d 'Show artifacts from all projects'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from list" -l archived -d 'Show archived artifacts instead of active'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from list" -l include-dives -d 'Also show dive/ files (spec only)'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from create" -l topic -d 'Artifact topic' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from create" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from create" -l source -d 'Source artifact stem for [[wiki-link]]' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from create" -l tags -d 'Comma-separated tags (e.g. domain/combat,stage/research)' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from create" -l dive -d 'Route to dive/ instead of spec/ (requires --source; spec only)'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from latest" -l include-dives -d 'Also scan dive/ files when finding latest (spec only)'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from archive" -l batch -d 'Batch archive multiple files' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from archive" -l dry-run -d 'Preview what would be archived without acting'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from prune" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be archived'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from comments" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from comments" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from retag" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand spec; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from list" -l all -d 'Show artifacts from all projects'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from list" -l archived -d 'Show archived artifacts instead of active'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from list" -l include-dives -d 'Also show dive/ files (spec only)'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from create" -l topic -d 'Artifact topic' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from create" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from create" -l source -d 'Source artifact stem for [[wiki-link]]' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from create" -l tags -d 'Comma-separated tags (e.g. domain/combat,stage/research)' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from create" -l dive -d 'Route to dive/ instead of spec/ (requires --source; spec only)'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from latest" -l include-dives -d 'Also scan dive/ files when finding latest (spec only)'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from archive" -l batch -d 'Batch archive multiple files' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from archive" -l dry-run -d 'Preview what would be archived without acting'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from prune" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be archived'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from comments" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from comments" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from retag" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand s; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from list" -l all -d 'Show artifacts from all projects'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from list" -l archived -d 'Show archived artifacts instead of active'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from list" -l include-dives -d 'Also show dive/ files (spec only)'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from create" -l topic -d 'Artifact topic' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from create" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from create" -l source -d 'Source artifact stem for [[wiki-link]]' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from create" -l tags -d 'Comma-separated tags (e.g. domain/combat,stage/research)' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from create" -l dive -d 'Route to dive/ instead of spec/ (requires --source; spec only)'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from latest" -l include-dives -d 'Also scan dive/ files when finding latest (spec only)'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from archive" -l batch -d 'Batch archive multiple files' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from archive" -l dry-run -d 'Preview what would be archived without acting'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from prune" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be archived'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from comments" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from comments" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from retag" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand review; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from list" -l all -d 'Show artifacts from all projects'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from list" -l archived -d 'Show archived artifacts instead of active'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from list" -l include-dives -d 'Also show dive/ files (spec only)'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from create" -l topic -d 'Artifact topic' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from create" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from create" -l source -d 'Source artifact stem for [[wiki-link]]' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from create" -l tags -d 'Comma-separated tags (e.g. domain/combat,stage/research)' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from create" -l dive -d 'Route to dive/ instead of spec/ (requires --source; spec only)'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from latest" -l include-dives -d 'Also scan dive/ files when finding latest (spec only)'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from archive" -l batch -d 'Batch archive multiple files' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from archive" -l dry-run -d 'Preview what would be archived without acting'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from prune" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be archived'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from comments" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from comments" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from retag" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand r; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from list" -l all -d 'Show artifacts from all projects'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from list" -l archived -d 'Show archived artifacts instead of active'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from list" -l include-dives -d 'Also show dive/ files (spec only)'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from create" -l topic -d 'Artifact topic' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from create" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from create" -l source -d 'Source artifact stem for [[wiki-link]]' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from create" -l tags -d 'Comma-separated tags (e.g. domain/combat,stage/research)' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from create" -l dive -d 'Route to dive/ instead of spec/ (requires --source; spec only)'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from latest" -l include-dives -d 'Also scan dive/ files when finding latest (spec only)'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from archive" -l batch -d 'Batch archive multiple files' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from archive" -l dry-run -d 'Preview what would be archived without acting'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from prune" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be archived'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from comments" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from comments" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from retag" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand report; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from list" -l all -d 'Show artifacts from all projects'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from list" -l archived -d 'Show archived artifacts instead of active'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from list" -l include-dives -d 'Also show dive/ files (spec only)'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from create" -l topic -d 'Artifact topic' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from create" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from create" -l source -d 'Source artifact stem for [[wiki-link]]' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from create" -l tags -d 'Comma-separated tags (e.g. domain/combat,stage/research)' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from create" -l dive -d 'Route to dive/ instead of spec/ (requires --source; spec only)'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from latest" -l include-dives -d 'Also scan dive/ files when finding latest (spec only)'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from archive" -l batch -d 'Batch archive multiple files' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from archive" -l dry-run -d 'Preview what would be archived without acting'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from prune" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be archived'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from comments" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from comments" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from retag" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand rp; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from list" -l all -d 'Show artifacts from all projects'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from list" -l archived -d 'Show archived artifacts instead of active'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from list" -l include-dives -d 'Also show dive/ files (spec only)'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from create" -l topic -d 'Artifact topic' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from create" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from create" -l source -d 'Source artifact stem for [[wiki-link]]' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from create" -l tags -d 'Comma-separated tags (e.g. domain/combat,stage/research)' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from create" -l dive -d 'Route to dive/ instead of spec/ (requires --source; spec only)'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from latest" -l include-dives -d 'Also scan dive/ files when finding latest (spec only)'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from archive" -l batch -d 'Batch archive multiple files' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from archive" -l dry-run -d 'Preview what would be archived without acting'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from prune" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be archived'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from comments" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from comments" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from retag" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand doc; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from list" -l all -d 'Show artifacts from all projects'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from list" -l archived -d 'Show archived artifacts instead of active'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from list" -l include-dives -d 'Also show dive/ files (spec only)'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from create" -l topic -d 'Artifact topic' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from create" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from create" -l source -d 'Source artifact stem for [[wiki-link]]' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from create" -l tags -d 'Comma-separated tags (e.g. domain/combat,stage/research)' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from create" -l dive -d 'Route to dive/ instead of spec/ (requires --source; spec only)'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from latest" -l include-dives -d 'Also scan dive/ files when finding latest (spec only)'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from archive" -l batch -d 'Batch archive multiple files' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from archive" -l dry-run -d 'Preview what would be archived without acting'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from prune" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be archived'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from comments" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from comments" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from retag" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand d; and not __fish_seen_subcommand_from list create read latest archive show prune comments rename retag help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from list" -l all -d 'Show artifacts from all projects'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from list" -l archived -d 'Show archived artifacts instead of active'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from list" -l include-dives -d 'Also show dive/ files (spec only)'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from create" -l topic -d 'Artifact topic' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from create" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from create" -l source -d 'Source artifact stem for [[wiki-link]]' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from create" -l tags -d 'Comma-separated tags (e.g. domain/combat,stage/research)' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from create" -l dive -d 'Route to dive/ instead of spec/ (requires --source; spec only)'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from latest" -l include-dives -d 'Also scan dive/ files when finding latest (spec only)'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from archive" -l batch -d 'Batch archive multiple files' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from archive" -l dry-run -d 'Preview what would be archived without acting'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from prune" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be archived'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from comments" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from comments" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from retag" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand vault; and not __fish_seen_subcommand_from init migrate project related check search status help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand vault; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "init" -d 'Initialize ~/blueprints/ repository'
complete -c ct -n "__fish_ct_using_subcommand vault; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "migrate" -d 'Migrate artifacts from ~/.claude/ to ~/blueprints/'
complete -c ct -n "__fish_ct_using_subcommand vault; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "project" -d 'Print detected project name'
complete -c ct -n "__fish_ct_using_subcommand vault; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "related" -d 'Find related artifacts by topic keyword overlap'
complete -c ct -n "__fish_ct_using_subcommand vault; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "check" -d 'Check for unresolved wiki-links (via Obsidian CLI)'
complete -c ct -n "__fish_ct_using_subcommand vault; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "search" -d 'Search artifacts (via Obsidian CLI)'
complete -c ct -n "__fish_ct_using_subcommand vault; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "status" -d 'Show vault status (git state, artifact count)'
complete -c ct -n "__fish_ct_using_subcommand vault; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from migrate" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from project" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from related" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from related" -l archive -d 'Include archived artifacts'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from related" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from check" -l archive -d 'Include archived artifacts'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from check" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from search" -l type -d 'Filter by artifact type (spec, plan, review, report, doc)' -r -f -a "spec\t''
plan\t''
review\t''
report\t''
doc\t''"
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from search" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from search" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from search" -l archive -d 'Include archived artifacts'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "init" -d 'Initialize ~/blueprints/ repository'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "migrate" -d 'Migrate artifacts from ~/.claude/ to ~/blueprints/'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "project" -d 'Print detected project name'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "related" -d 'Find related artifacts by topic keyword overlap'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "check" -d 'Check for unresolved wiki-links (via Obsidian CLI)'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search artifacts (via Obsidian CLI)'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show vault status (git state, artifact count)'
complete -c ct -n "__fish_ct_using_subcommand vault; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand v; and not __fish_seen_subcommand_from init migrate project related check search status help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand v; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "init" -d 'Initialize ~/blueprints/ repository'
complete -c ct -n "__fish_ct_using_subcommand v; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "migrate" -d 'Migrate artifacts from ~/.claude/ to ~/blueprints/'
complete -c ct -n "__fish_ct_using_subcommand v; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "project" -d 'Print detected project name'
complete -c ct -n "__fish_ct_using_subcommand v; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "related" -d 'Find related artifacts by topic keyword overlap'
complete -c ct -n "__fish_ct_using_subcommand v; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "check" -d 'Check for unresolved wiki-links (via Obsidian CLI)'
complete -c ct -n "__fish_ct_using_subcommand v; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "search" -d 'Search artifacts (via Obsidian CLI)'
complete -c ct -n "__fish_ct_using_subcommand v; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "status" -d 'Show vault status (git state, artifact count)'
complete -c ct -n "__fish_ct_using_subcommand v; and not __fish_seen_subcommand_from init migrate project related check search status help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from migrate" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from project" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from related" -l project -d 'Project path (defaults to current git repo / cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from related" -l archive -d 'Include archived artifacts'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from related" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from check" -l archive -d 'Include archived artifacts'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from check" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from search" -l type -d 'Filter by artifact type (spec, plan, review, report, doc)' -r -f -a "spec\t''
plan\t''
review\t''
report\t''
doc\t''"
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from search" -s p -l project -d 'Filter by project path' -r
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from search" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from search" -l archive -d 'Include archived artifacts'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from help" -f -a "init" -d 'Initialize ~/blueprints/ repository'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from help" -f -a "migrate" -d 'Migrate artifacts from ~/.claude/ to ~/blueprints/'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from help" -f -a "project" -d 'Print detected project name'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from help" -f -a "related" -d 'Find related artifacts by topic keyword overlap'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from help" -f -a "check" -d 'Check for unresolved wiki-links (via Obsidian CLI)'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search artifacts (via Obsidian CLI)'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show vault status (git state, artifact count)'
complete -c ct -n "__fish_ct_using_subcommand v; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand notify" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand n" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "check-refs" -d 'Check doc references against project filesystem'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "churn" -d 'Report per-module LOC and recent git churn'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "apply-patch" -d 'Apply a patch (envelope format) to files under cwd'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from slug" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from phases" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from completion" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -l base -d 'Base branch for comparison' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -l format -d 'Output format: text or json' -r -f -a "text\t''
json\t''"
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -l max-total -d 'Max total diff lines before truncation' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -l max-file -d 'Per-file diff line threshold' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -l stat -d 'Output diff --stat instead of full diff'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -l cochanges -d 'Include co-change candidates in output'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from check-refs" -l project-root -d 'Project root path (default: git rev-parse --show-toplevel)' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from check-refs" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -l base -d 'Base branch/ref for changed-file detection' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -l threshold -d 'Min co-change fraction 0.0-1.0' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -l min-commits -d 'Min commits a file must appear in' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -l max-files -d 'Max output files (integer or \'all\')' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -l num-commits -d 'How many recent commits to analyze' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from churn" -l project-root -d 'Project root path (default: git rev-parse --show-toplevel)' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from churn" -l since -d 'Git log time window (e.g. 2w, 30d, 3m)' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from churn" -l min-loc -d 'Minimum LOC to include in output' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from churn" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from apply-patch" -l cwd -d 'Working directory (default: process cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from apply-patch" -l dry-run -d 'Preview changes without writing to disk'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from apply-patch" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "check-refs" -d 'Check doc references against project filesystem'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "churn" -d 'Report per-module LOC and recent git churn'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "apply-patch" -d 'Apply a patch (envelope format) to files under cwd'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "check-refs" -d 'Check doc references against project filesystem'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "churn" -d 'Report per-module LOC and recent git churn'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "apply-patch" -d 'Apply a patch (envelope format) to files under cwd'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn apply-patch help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from slug" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from phases" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from completion" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -l base -d 'Base branch for comparison' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -l format -d 'Output format: text or json' -r -f -a "text\t''
json\t''"
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -l max-total -d 'Max total diff lines before truncation' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -l max-file -d 'Per-file diff line threshold' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -l stat -d 'Output diff --stat instead of full diff'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -l cochanges -d 'Include co-change candidates in output'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from check-refs" -l project-root -d 'Project root path (default: git rev-parse --show-toplevel)' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from check-refs" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from cochanges" -l base -d 'Base branch/ref for changed-file detection' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from cochanges" -l threshold -d 'Min co-change fraction 0.0-1.0' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from cochanges" -l min-commits -d 'Min commits a file must appear in' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from cochanges" -l max-files -d 'Max output files (integer or \'all\')' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from cochanges" -l num-commits -d 'How many recent commits to analyze' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from cochanges" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from churn" -l project-root -d 'Project root path (default: git rev-parse --show-toplevel)' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from churn" -l since -d 'Git log time window (e.g. 2w, 30d, 3m)' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from churn" -l min-loc -d 'Minimum LOC to include in output' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from churn" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from apply-patch" -l cwd -d 'Working directory (default: process cwd)' -r
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from apply-patch" -l dry-run -d 'Preview changes without writing to disk'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from apply-patch" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "check-refs" -d 'Check doc references against project filesystem'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "churn" -d 'Report per-module LOC and recent git churn'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "apply-patch" -d 'Apply a patch (envelope format) to files under cwd'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -s d -l db -r -F
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -l json
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "index" -d 'Index a directory for symbol discovery'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "search" -d 'Search symbols or text across files'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "outline" -d 'Show symbols defined in a file'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "show" -d 'Read source by symbol name or file path'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "ls" -d 'Show file tree, repo list, or repo stats'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "refs" -d 'Find references to a symbol'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "importers" -d 'Find files that import a given file or package'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "impact" -d 'Find transitive callers of a symbol'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "trace" -d 'Follow the call graph downward from a symbol'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "impls" -d 'Find types that implement a symbol or what a type implements'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "context" -d 'Bundled context: source, callers, conformance, and file imports'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "investigate" -d 'Kind-adaptive investigation for symbols'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "structure" -d 'Structural overview of the indexed codebase'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "diff" -d 'Show git diff scoped to a symbol\'s definition'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "hook" -d 'Agent-integration hooks (nudge, remind, install)'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "version" -d 'Print sym version information'
complete -c ct -n "__fish_ct_using_subcommand sym; and not __fish_seen_subcommand_from index search outline show ls refs importers impact trace impls context investigate structure diff hook version help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from index" -s w -l workers -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from index" -l ignore -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from index" -s f -l force
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from index" -l reset
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from index" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from search" -s n -l limit -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from search" -s k -l kind -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from search" -s l -l lang -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from search" -l path -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from search" -l exclude -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from search" -s t -l text
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from search" -s e -l exact
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from search" -s i -l ignore-case
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from outline" -s s -l signatures
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from outline" -l names
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from outline" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from show" -s C -l context -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from show" -l all
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from show" -l stdin
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from ls" -s D -l depth -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from ls" -l repos
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from ls" -l stats
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from refs" -s D -l depth -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from refs" -s n -l limit -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from refs" -s C -l context -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from refs" -l path -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from refs" -l exclude -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from refs" -l file -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from refs" -l importers
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from refs" -l impact
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from refs" -l stdin
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from refs" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from importers" -s D -l depth -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from importers" -s n -l limit -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from importers" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impact" -s D -l depth -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impact" -s n -l limit -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impact" -s C -l context -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impact" -l stdin
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impact" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from trace" -l depth -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from trace" -s n -l limit -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from trace" -l kinds -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from trace" -l stdin
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from trace" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impls" -s l -l lang -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impls" -s n -l limit -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impls" -l path -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impls" -l exclude -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impls" -l of -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impls" -l resolved
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impls" -l unresolved
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impls" -l stdin
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from impls" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from context" -s n -l callers -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from context" -l stdin
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from context" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from investigate" -l stdin
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from investigate" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from structure" -s n -l limit -r
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from structure" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from diff" -l stat
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from diff" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from hook" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from hook" -f -a "nudge" -d 'Suggest a sym equivalent when an agent is about to grep'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from hook" -f -a "remind" -d 'Print a short reminder block an agent can inject as context'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from hook" -f -a "install" -d 'Install sym hooks into the given agent'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from hook" -f -a "uninstall" -d 'Remove sym hooks from the given agent'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from hook" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from version" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "index" -d 'Index a directory for symbol discovery'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search symbols or text across files'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "outline" -d 'Show symbols defined in a file'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "show" -d 'Read source by symbol name or file path'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "ls" -d 'Show file tree, repo list, or repo stats'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "refs" -d 'Find references to a symbol'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "importers" -d 'Find files that import a given file or package'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "impact" -d 'Find transitive callers of a symbol'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "trace" -d 'Follow the call graph downward from a symbol'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "impls" -d 'Find types that implement a symbol or what a type implements'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "context" -d 'Bundled context: source, callers, conformance, and file imports'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "investigate" -d 'Kind-adaptive investigation for symbols'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "structure" -d 'Structural overview of the indexed codebase'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "diff" -d 'Show git diff scoped to a symbol\'s definition'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "hook" -d 'Agent-integration hooks (nudge, remind, install)'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "version" -d 'Print sym version information'
complete -c ct -n "__fish_ct_using_subcommand sym; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand mcp; and not __fish_seen_subcommand_from blueprint apply-patch help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand mcp; and not __fish_seen_subcommand_from blueprint apply-patch help" -f -a "blueprint" -d 'Serve the blueprint/vault MCP over stdio'
complete -c ct -n "__fish_ct_using_subcommand mcp; and not __fish_seen_subcommand_from blueprint apply-patch help" -f -a "apply-patch" -d 'Serve the apply_patch MCP over stdio'
complete -c ct -n "__fish_ct_using_subcommand mcp; and not __fish_seen_subcommand_from blueprint apply-patch help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand mcp; and __fish_seen_subcommand_from blueprint" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand mcp; and __fish_seen_subcommand_from apply-patch" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand mcp; and __fish_seen_subcommand_from help" -f -a "blueprint" -d 'Serve the blueprint/vault MCP over stdio'
complete -c ct -n "__fish_ct_using_subcommand mcp; and __fish_seen_subcommand_from help" -f -a "apply-patch" -d 'Serve the apply_patch MCP over stdio'
complete -c ct -n "__fish_ct_using_subcommand mcp; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and not __fish_seen_subcommand_from stats prune help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and not __fish_seen_subcommand_from stats prune help" -f -a "stats" -d 'Show apply_patch telemetry summary'
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and not __fish_seen_subcommand_from stats prune help" -f -a "prune" -d 'Delete telemetry older than --days'
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and not __fish_seen_subcommand_from stats prune help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and __fish_seen_subcommand_from stats" -l days -d 'Window in days' -r
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and __fish_seen_subcommand_from stats" -l all-projects -d 'Walk all project databases'
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and __fish_seen_subcommand_from stats" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and __fish_seen_subcommand_from prune" -l days -d 'Retention window in days' -r
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and __fish_seen_subcommand_from help" -f -a "stats" -d 'Show apply_patch telemetry summary'
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Delete telemetry older than --days'
complete -c ct -n "__fish_ct_using_subcommand apply-patch; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "plan" -d 'Plan file operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "spec" -d 'Spec file operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "review" -d 'Review file operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "report" -d 'Report file operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "doc" -d 'Doc file operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "vault" -d 'Vault repository management'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "read" -d 'Read artifact by stem (resolves across all types)'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "notify" -d 'Handle notification hooks'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "tool" -d 'Utility tools'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "sym" -d 'Code indexing and symbol discovery'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "mcp" -d 'Run the MCP stdio server'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "apply-patch" -d 'Inspect or prune apply_patch telemetry'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from plan spec review report doc vault read notify tool sym mcp apply-patch help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from spec" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from spec" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from spec" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from spec" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from spec" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from spec" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from spec" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from spec" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from spec" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from spec" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from review" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from report" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from report" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from report" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from report" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from report" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from report" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from report" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from report" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from report" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from report" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from doc" -f -a "list" -d 'List artifacts for the current project'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from doc" -f -a "create" -d 'Create a new artifact file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from doc" -f -a "read" -d 'Read artifact file body or frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from doc" -f -a "latest" -d 'Find most recently modified artifact file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from doc" -f -a "archive" -d 'Move an artifact file to archive/ subfolder'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from doc" -f -a "show" -d 'Show artifact content by ID'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from doc" -f -a "prune" -d 'Archive artifact files older than N days'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from doc" -f -a "comments" -d 'Extract inline HTML comments from an artifact'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from doc" -f -a "rename" -d 'Rename an artifact file and update its frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from doc" -f -a "retag" -d 'Fix auto-derived tags (type/*, project/*) in frontmatter'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "init" -d 'Initialize ~/blueprints/ repository'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "migrate" -d 'Migrate artifacts from ~/.claude/ to ~/blueprints/'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "project" -d 'Print detected project name'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "related" -d 'Find related artifacts by topic keyword overlap'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "check" -d 'Check for unresolved wiki-links (via Obsidian CLI)'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "search" -d 'Search artifacts (via Obsidian CLI)'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from vault" -f -a "status" -d 'Show vault status (git state, artifact count)'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "check-refs" -d 'Check doc references against project filesystem'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "churn" -d 'Report per-module LOC and recent git churn'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "apply-patch" -d 'Apply a patch (envelope format) to files under cwd'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "index" -d 'Index a directory for symbol discovery'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "search" -d 'Search symbols or text across files'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "outline" -d 'Show symbols defined in a file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "show" -d 'Read source by symbol name or file path'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "ls" -d 'Show file tree, repo list, or repo stats'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "refs" -d 'Find references to a symbol'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "importers" -d 'Find files that import a given file or package'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "impact" -d 'Find transitive callers of a symbol'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "trace" -d 'Follow the call graph downward from a symbol'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "impls" -d 'Find types that implement a symbol or what a type implements'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "context" -d 'Bundled context: source, callers, conformance, and file imports'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "investigate" -d 'Kind-adaptive investigation for symbols'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "structure" -d 'Structural overview of the indexed codebase'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "diff" -d 'Show git diff scoped to a symbol\'s definition'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "hook" -d 'Agent-integration hooks (nudge, remind, install)'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from sym" -f -a "version" -d 'Print sym version information'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from mcp" -f -a "blueprint" -d 'Serve the blueprint/vault MCP over stdio'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from mcp" -f -a "apply-patch" -d 'Serve the apply_patch MCP over stdio'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from apply-patch" -f -a "stats" -d 'Show apply_patch telemetry summary'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from apply-patch" -f -a "prune" -d 'Delete telemetry older than --days'
