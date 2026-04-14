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
complete -c ct -n "__fish_ct_needs_command" -f -a "tui" -d 'Launch the interactive TUI'
complete -c ct -n "__fish_ct_needs_command" -f -a "task" -d 'Task operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "t" -d 'Task operations'
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
complete -c ct -n "__fish_ct_needs_command" -f -a "project" -d 'Project operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "j" -d 'Project operations'
complete -c ct -n "__fish_ct_needs_command" -f -a "read" -d 'Read artifact by stem (resolves across all types)'
complete -c ct -n "__fish_ct_needs_command" -f -a "notify" -d 'Handle notification hooks'
complete -c ct -n "__fish_ct_needs_command" -f -a "n" -d 'Handle notification hooks'
complete -c ct -n "__fish_ct_needs_command" -f -a "tool" -d 'Utility tools'
complete -c ct -n "__fish_ct_needs_command" -f -a "o" -d 'Utility tools'
complete -c ct -n "__fish_ct_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand tui" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "list" -d 'List tasks'
complete -c ct -n "__fish_ct_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "show" -d 'Show task details'
complete -c ct -n "__fish_ct_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "create" -d 'Create a new task'
complete -c ct -n "__fish_ct_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "edit" -d 'Edit an existing task'
complete -c ct -n "__fish_ct_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "status" -d 'Update task status'
complete -c ct -n "__fish_ct_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "prune" -d 'Archive completed tasks older than N days'
complete -c ct -n "__fish_ct_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from list" -l status -d 'Filter by status (pending, in_progress, completed)' -r -f -a "pending\t''
in_progress\t''
completed\t''
active\t''
all\t''"
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from list" -l sort -d 'Sort by field (id, subject, priority)' -r -f -a "id\t''
subject\t''
priority\t''"
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from list" -l tree -d 'Display tasks as a tree grouped by parent'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from show" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from create" -l description -d 'Task description' -r
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from create" -l priority -d 'Priority (1-3)' -r
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from create" -l parent -d 'Parent task ID' -r
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from edit" -l subject -d 'New subject' -r
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from edit" -l status -d 'New status (pending, in_progress, completed)' -r -f -a "pending\t''
in_progress\t''
completed\t''"
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from edit" -l priority -d 'New priority (1-5)' -r
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from prune" -l list -d 'Only prune tasks from this list ID' -r
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be pruned without archiving'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "list" -d 'List tasks'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show task details'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new task'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Edit an existing task'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "status" -d 'Update task status'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive completed tasks older than N days'
complete -c ct -n "__fish_ct_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "list" -d 'List tasks'
complete -c ct -n "__fish_ct_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "show" -d 'Show task details'
complete -c ct -n "__fish_ct_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "create" -d 'Create a new task'
complete -c ct -n "__fish_ct_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "edit" -d 'Edit an existing task'
complete -c ct -n "__fish_ct_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "status" -d 'Update task status'
complete -c ct -n "__fish_ct_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "prune" -d 'Archive completed tasks older than N days'
complete -c ct -n "__fish_ct_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from list" -l status -d 'Filter by status (pending, in_progress, completed)' -r -f -a "pending\t''
in_progress\t''
completed\t''
active\t''
all\t''"
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from list" -l sort -d 'Sort by field (id, subject, priority)' -r -f -a "id\t''
subject\t''
priority\t''"
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from list" -l tree -d 'Display tasks as a tree grouped by parent'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from show" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from create" -l description -d 'Task description' -r
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from create" -l priority -d 'Priority (1-3)' -r
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from create" -l parent -d 'Parent task ID' -r
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from edit" -l subject -d 'New subject' -r
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from edit" -l status -d 'New status (pending, in_progress, completed)' -r -f -a "pending\t''
in_progress\t''
completed\t''"
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from edit" -l priority -d 'New priority (1-5)' -r
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from prune" -l list -d 'Only prune tasks from this list ID' -r
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be pruned without archiving'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "list" -d 'List tasks'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show task details'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new task'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Edit an existing task'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "status" -d 'Update task status'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive completed tasks older than N days'
complete -c ct -n "__fish_ct_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
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
complete -c ct -n "__fish_ct_using_subcommand plan; and __fish_seen_subcommand_from create" -l body -d 'Artifact body content' -r
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
complete -c ct -n "__fish_ct_using_subcommand p; and __fish_seen_subcommand_from create" -l body -d 'Artifact body content' -r
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
complete -c ct -n "__fish_ct_using_subcommand spec; and __fish_seen_subcommand_from create" -l body -d 'Artifact body content' -r
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
complete -c ct -n "__fish_ct_using_subcommand s; and __fish_seen_subcommand_from create" -l body -d 'Artifact body content' -r
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
complete -c ct -n "__fish_ct_using_subcommand review; and __fish_seen_subcommand_from create" -l body -d 'Artifact body content' -r
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
complete -c ct -n "__fish_ct_using_subcommand r; and __fish_seen_subcommand_from create" -l body -d 'Artifact body content' -r
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
complete -c ct -n "__fish_ct_using_subcommand report; and __fish_seen_subcommand_from create" -l body -d 'Artifact body content' -r
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
complete -c ct -n "__fish_ct_using_subcommand rp; and __fish_seen_subcommand_from create" -l body -d 'Artifact body content' -r
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
complete -c ct -n "__fish_ct_using_subcommand doc; and __fish_seen_subcommand_from create" -l body -d 'Artifact body content' -r
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
complete -c ct -n "__fish_ct_using_subcommand d; and __fish_seen_subcommand_from create" -l body -d 'Artifact body content' -r
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
complete -c ct -n "__fish_ct_using_subcommand project; and not __fish_seen_subcommand_from list show help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand project; and not __fish_seen_subcommand_from list show help" -f -a "list" -d 'List known projects'
complete -c ct -n "__fish_ct_using_subcommand project; and not __fish_seen_subcommand_from list show help" -f -a "show" -d 'Show project details'
complete -c ct -n "__fish_ct_using_subcommand project; and not __fish_seen_subcommand_from list show help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand project; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand project; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand project; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand project; and __fish_seen_subcommand_from help" -f -a "list" -d 'List known projects'
complete -c ct -n "__fish_ct_using_subcommand project; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show project details'
complete -c ct -n "__fish_ct_using_subcommand project; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand j; and not __fish_seen_subcommand_from list show help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand j; and not __fish_seen_subcommand_from list show help" -f -a "list" -d 'List known projects'
complete -c ct -n "__fish_ct_using_subcommand j; and not __fish_seen_subcommand_from list show help" -f -a "show" -d 'Show project details'
complete -c ct -n "__fish_ct_using_subcommand j; and not __fish_seen_subcommand_from list show help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand j; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ct -n "__fish_ct_using_subcommand j; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand j; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand j; and __fish_seen_subcommand_from help" -f -a "list" -d 'List known projects'
complete -c ct -n "__fish_ct_using_subcommand j; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show project details'
complete -c ct -n "__fish_ct_using_subcommand j; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ct -n "__fish_ct_using_subcommand read" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand notify" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand n" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "check-refs" -d 'Check doc references against project filesystem'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "churn" -d 'Report per-module LOC and recent git churn'
complete -c ct -n "__fish_ct_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
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
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "check-refs" -d 'Check doc references against project filesystem'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "churn" -d 'Report per-module LOC and recent git churn'
complete -c ct -n "__fish_ct_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "check-refs" -d 'Check doc references against project filesystem'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "churn" -d 'Report per-module LOC and recent git churn'
complete -c ct -n "__fish_ct_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext check-refs cochanges churn help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
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
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "check-refs" -d 'Check doc references against project filesystem'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "churn" -d 'Report per-module LOC and recent git churn'
complete -c ct -n "__fish_ct_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "tui" -d 'Launch the interactive TUI'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "task" -d 'Task operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "plan" -d 'Plan file operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "spec" -d 'Spec file operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "review" -d 'Review file operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "report" -d 'Report file operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "doc" -d 'Doc file operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "vault" -d 'Vault repository management'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "project" -d 'Project operations'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "read" -d 'Read artifact by stem (resolves across all types)'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "notify" -d 'Handle notification hooks'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "tool" -d 'Utility tools'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from tui task plan spec review report doc vault project read notify tool help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "list" -d 'List tasks'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "show" -d 'Show task details'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "create" -d 'Create a new task'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "edit" -d 'Edit an existing task'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "status" -d 'Update task status'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "prune" -d 'Archive completed tasks older than N days'
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
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from project" -f -a "list" -d 'List known projects'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from project" -f -a "show" -d 'Show project details'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "check-refs" -d 'Check doc references against project filesystem'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "churn" -d 'Report per-module LOC and recent git churn'
