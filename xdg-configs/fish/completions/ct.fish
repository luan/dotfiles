# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_ck_global_optspecs
	string join \n h/help
end

function __fish_ck_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_ck_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_ck_using_subcommand
	set -l cmd (__fish_ck_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c ck -n "__fish_ck_needs_command" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_needs_command" -f -a "tui" -d 'Launch the interactive TUI'
complete -c ck -n "__fish_ck_needs_command" -f -a "task" -d 'Task operations'
complete -c ck -n "__fish_ck_needs_command" -f -a "t" -d 'Task operations'
complete -c ck -n "__fish_ck_needs_command" -f -a "plan" -d 'Plan file operations'
complete -c ck -n "__fish_ck_needs_command" -f -a "p" -d 'Plan file operations'
complete -c ck -n "__fish_ck_needs_command" -f -a "project" -d 'Project operations'
complete -c ck -n "__fish_ck_needs_command" -f -a "j" -d 'Project operations'
complete -c ck -n "__fish_ck_needs_command" -f -a "tool" -d 'Utility tools'
complete -c ck -n "__fish_ck_needs_command" -f -a "o" -d 'Utility tools'
complete -c ck -n "__fish_ck_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand tui" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "list" -d 'List tasks'
complete -c ck -n "__fish_ck_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "show" -d 'Show task details'
complete -c ck -n "__fish_ck_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "create" -d 'Create a new task'
complete -c ck -n "__fish_ck_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "edit" -d 'Edit an existing task'
complete -c ck -n "__fish_ck_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "status" -d 'Update task status'
complete -c ck -n "__fish_ck_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "prune" -d 'Archive completed tasks older than N days'
complete -c ck -n "__fish_ck_using_subcommand task; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from list" -l status -d 'Filter by status (pending, in_progress, completed)' -r -f -a "pending\t''
in_progress\t''
completed\t''
active\t''
all\t''"
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from list" -l sort -d 'Sort by field (id, subject, priority)' -r -f -a "id\t''
subject\t''
priority\t''"
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from list" -l tree -d 'Display tasks as a tree grouped by parent'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from show" -l json -d 'Output as JSON'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from create" -l description -d 'Task description' -r
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from create" -l priority -d 'Priority (1-3)' -r
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from create" -l parent -d 'Parent task ID' -r
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from edit" -l subject -d 'New subject' -r
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from edit" -l status -d 'New status (pending, in_progress, completed)' -r -f -a "pending\t''
in_progress\t''
completed\t''"
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from edit" -l priority -d 'New priority (1-5)' -r
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from prune" -l list -d 'Only prune tasks from this list ID' -r
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be pruned without archiving'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "list" -d 'List tasks'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show task details'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new task'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Edit an existing task'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "status" -d 'Update task status'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive completed tasks older than N days'
complete -c ck -n "__fish_ck_using_subcommand task; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "list" -d 'List tasks'
complete -c ck -n "__fish_ck_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "show" -d 'Show task details'
complete -c ck -n "__fish_ck_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "create" -d 'Create a new task'
complete -c ck -n "__fish_ck_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "edit" -d 'Edit an existing task'
complete -c ck -n "__fish_ck_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "status" -d 'Update task status'
complete -c ck -n "__fish_ck_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "prune" -d 'Archive completed tasks older than N days'
complete -c ck -n "__fish_ck_using_subcommand t; and not __fish_seen_subcommand_from list show create edit status prune help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from list" -l status -d 'Filter by status (pending, in_progress, completed)' -r -f -a "pending\t''
in_progress\t''
completed\t''
active\t''
all\t''"
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from list" -l sort -d 'Sort by field (id, subject, priority)' -r -f -a "id\t''
subject\t''
priority\t''"
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from list" -l tree -d 'Display tasks as a tree grouped by parent'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from show" -l json -d 'Output as JSON'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from create" -l description -d 'Task description' -r
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from create" -l priority -d 'Priority (1-3)' -r
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from create" -l parent -d 'Parent task ID' -r
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from edit" -l subject -d 'New subject' -r
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from edit" -l status -d 'New status (pending, in_progress, completed)' -r -f -a "pending\t''
in_progress\t''
completed\t''"
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from edit" -l priority -d 'New priority (1-5)' -r
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from prune" -l days -d 'Age threshold in days' -r
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from prune" -l list -d 'Only prune tasks from this list ID' -r
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from prune" -l dry-run -d 'Dry run — print what would be pruned without archiving'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from prune" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "list" -d 'List tasks'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show task details'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new task'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Edit an existing task'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "status" -d 'Update task status'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "prune" -d 'Archive completed tasks older than N days'
complete -c ck -n "__fish_ck_using_subcommand t; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show help" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "list" -d 'List execution plans for the current project'
complete -c ck -n "__fish_ck_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "create" -d 'Create a new plan file'
complete -c ck -n "__fish_ck_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "read" -d 'Read plan file body or frontmatter'
complete -c ck -n "__fish_ck_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "latest" -d 'Find most recently modified plan file'
complete -c ck -n "__fish_ck_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "archive" -d 'Move a plan file to archive/ subfolder'
complete -c ck -n "__fish_ck_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "show" -d 'Show plan content by ID'
complete -c ck -n "__fish_ck_using_subcommand plan; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from list" -l all -d 'Show plans from all projects'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from list" -l archived -d 'Show archived plans instead of active'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from create" -l topic -d 'Plan topic' -r
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from create" -l project -d 'Project path' -r
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from create" -l prefix -d 'Filename prefix' -r
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from create" -l body -d 'Plan body content' -r
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "list" -d 'List execution plans for the current project'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new plan file'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read plan file body or frontmatter'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified plan file'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move a plan file to archive/ subfolder'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show plan content by ID'
complete -c ck -n "__fish_ck_using_subcommand plan; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show help" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "list" -d 'List execution plans for the current project'
complete -c ck -n "__fish_ck_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "create" -d 'Create a new plan file'
complete -c ck -n "__fish_ck_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "read" -d 'Read plan file body or frontmatter'
complete -c ck -n "__fish_ck_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "latest" -d 'Find most recently modified plan file'
complete -c ck -n "__fish_ck_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "archive" -d 'Move a plan file to archive/ subfolder'
complete -c ck -n "__fish_ck_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "show" -d 'Show plan content by ID'
complete -c ck -n "__fish_ck_using_subcommand p; and not __fish_seen_subcommand_from list create read latest archive show help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from list" -s p -l project -d 'Filter by project path' -r
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from list" -l all -d 'Show plans from all projects'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from list" -l archived -d 'Show archived plans instead of active'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from create" -l topic -d 'Plan topic' -r
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from create" -l project -d 'Project path' -r
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from create" -l slug -d 'Custom slug (auto-generated if omitted)' -r
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from create" -l prefix -d 'Filename prefix' -r
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from create" -l body -d 'Plan body content' -r
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from read" -l frontmatter -d 'Output frontmatter as JSON'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from read" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from latest" -l project -d 'Project path (defaults to git root or cwd)' -r
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from latest" -l task-file -d 'Resolve this file directly instead of mtime heuristic' -r
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from latest" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from archive" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "list" -d 'List execution plans for the current project'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "create" -d 'Create a new plan file'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "read" -d 'Read plan file body or frontmatter'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "latest" -d 'Find most recently modified plan file'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "archive" -d 'Move a plan file to archive/ subfolder'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show plan content by ID'
complete -c ck -n "__fish_ck_using_subcommand p; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand project; and not __fish_seen_subcommand_from list show help" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand project; and not __fish_seen_subcommand_from list show help" -f -a "list" -d 'List known projects'
complete -c ck -n "__fish_ck_using_subcommand project; and not __fish_seen_subcommand_from list show help" -f -a "show" -d 'Show project details'
complete -c ck -n "__fish_ck_using_subcommand project; and not __fish_seen_subcommand_from list show help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand project; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ck -n "__fish_ck_using_subcommand project; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand project; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand project; and __fish_seen_subcommand_from help" -f -a "list" -d 'List known projects'
complete -c ck -n "__fish_ck_using_subcommand project; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show project details'
complete -c ck -n "__fish_ck_using_subcommand project; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand j; and not __fish_seen_subcommand_from list show help" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand j; and not __fish_seen_subcommand_from list show help" -f -a "list" -d 'List known projects'
complete -c ck -n "__fish_ck_using_subcommand j; and not __fish_seen_subcommand_from list show help" -f -a "show" -d 'Show project details'
complete -c ck -n "__fish_ck_using_subcommand j; and not __fish_seen_subcommand_from list show help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand j; and __fish_seen_subcommand_from list" -l json -d 'Output as JSON'
complete -c ck -n "__fish_ck_using_subcommand j; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand j; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand j; and __fish_seen_subcommand_from help" -f -a "list" -d 'List known projects'
complete -c ck -n "__fish_ck_using_subcommand j; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show project details'
complete -c ck -n "__fish_ck_using_subcommand j; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ck -n "__fish_ck_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ck -n "__fish_ck_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ck -n "__fish_ck_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ck -n "__fish_ck_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ck -n "__fish_ck_using_subcommand tool; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from slug" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from phases" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from completion" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -l base -d 'Base branch for comparison' -r
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -l format -d 'Output format: text or json' -r -f -a "text\t''
json\t''"
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -l max-total -d 'Max total diff lines before truncation' -r
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -l max-file -d 'Per-file diff line threshold' -r
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from gitcontext" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -l base -d 'Base branch/ref for changed-file detection' -r
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -l threshold -d 'Min co-change fraction 0.0-1.0' -r
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -l min-commits -d 'Min commits a file must appear in' -r
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -l max-files -d 'Max output files (integer or \'all\')' -r
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -l num-commits -d 'How many recent commits to analyze' -r
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from cochanges" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ck -n "__fish_ck_using_subcommand tool; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ck -n "__fish_ck_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ck -n "__fish_ck_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ck -n "__fish_ck_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ck -n "__fish_ck_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ck -n "__fish_ck_using_subcommand o; and not __fish_seen_subcommand_from slug phases completion gitcontext cochanges help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from slug" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from phases" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from completion" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -l base -d 'Base branch for comparison' -r
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -l format -d 'Output format: text or json' -r -f -a "text\t''
json\t''"
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -l max-total -d 'Max total diff lines before truncation' -r
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -l max-file -d 'Per-file diff line threshold' -r
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from gitcontext" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from cochanges" -l base -d 'Base branch/ref for changed-file detection' -r
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from cochanges" -l threshold -d 'Min co-change fraction 0.0-1.0' -r
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from cochanges" -l min-commits -d 'Min commits a file must appear in' -r
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from cochanges" -l max-files -d 'Max output files (integer or \'all\')' -r
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from cochanges" -l num-commits -d 'How many recent commits to analyze' -r
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from cochanges" -s h -l help -d 'Print help'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
complete -c ck -n "__fish_ck_using_subcommand o; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand help; and not __fish_seen_subcommand_from tui task plan project tool help" -f -a "tui" -d 'Launch the interactive TUI'
complete -c ck -n "__fish_ck_using_subcommand help; and not __fish_seen_subcommand_from tui task plan project tool help" -f -a "task" -d 'Task operations'
complete -c ck -n "__fish_ck_using_subcommand help; and not __fish_seen_subcommand_from tui task plan project tool help" -f -a "plan" -d 'Plan file operations'
complete -c ck -n "__fish_ck_using_subcommand help; and not __fish_seen_subcommand_from tui task plan project tool help" -f -a "project" -d 'Project operations'
complete -c ck -n "__fish_ck_using_subcommand help; and not __fish_seen_subcommand_from tui task plan project tool help" -f -a "tool" -d 'Utility tools'
complete -c ck -n "__fish_ck_using_subcommand help; and not __fish_seen_subcommand_from tui task plan project tool help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "list" -d 'List tasks'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "show" -d 'Show task details'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "create" -d 'Create a new task'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "edit" -d 'Edit an existing task'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "status" -d 'Update task status'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from task" -f -a "prune" -d 'Archive completed tasks older than N days'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "list" -d 'List execution plans for the current project'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "create" -d 'Create a new plan file'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "read" -d 'Read plan file body or frontmatter'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "latest" -d 'Find most recently modified plan file'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "archive" -d 'Move a plan file to archive/ subfolder'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from plan" -f -a "show" -d 'Show plan content by ID'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from project" -f -a "list" -d 'List known projects'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from project" -f -a "show" -d 'Show project details'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "slug" -d 'Generate URL-safe slug from text'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "phases" -d 'Parse phase markers from plan file'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "gitcontext" -d 'Gather branch context (diff, log, files) for skills'
complete -c ck -n "__fish_ck_using_subcommand help; and __fish_seen_subcommand_from tool" -f -a "cochanges" -d 'Find files frequently changed together with current changes'
