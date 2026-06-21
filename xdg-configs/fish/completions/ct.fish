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
complete -c ct -n "__fish_ct_needs_command" -f -a "apply-patch" -d 'Apply patches from stdin'
complete -c ct -n "__fish_ct_needs_command" -f -a "shell" -d 'Shell integration helpers'
complete -c ct -n "__fish_ct_needs_command" -f -a "tui" -d 'Terminal UI helpers'
complete -c ct -n "__fish_ct_needs_command" -f -a "notify" -d 'Handle notification hooks'
complete -c ct -n "__fish_ct_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand apply-patch" -l cwd -d 'Working directory for raw apply; default: process cwd' -r
complete -c ct -n "__fish_ct_using_subcommand apply-patch" -l dry-run -d 'Preview raw apply without writing to disk'
complete -c ct -n "__fish_ct_using_subcommand apply-patch" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand shell; and not __fish_seen_subcommand_from completion help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand shell; and not __fish_seen_subcommand_from completion help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand shell; and not __fish_seen_subcommand_from completion help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand shell; and __fish_seen_subcommand_from completion" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand shell; and __fish_seen_subcommand_from help" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand shell; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand tui; and not __fish_seen_subcommand_from usage-bar usage-bars help" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tui; and not __fish_seen_subcommand_from usage-bar usage-bars help" -f -a "usage-bar" -d 'Render subscription usage bars from JSON on stdin'
complete -c ct -n "__fish_ct_using_subcommand tui; and not __fish_seen_subcommand_from usage-bar usage-bars help" -f -a "usage-bars" -d 'Render subscription usage bars for all local providers'
complete -c ct -n "__fish_ct_using_subcommand tui; and not __fish_seen_subcommand_from usage-bar usage-bars help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand tui; and __fish_seen_subcommand_from usage-bar" -l width -d 'Terminal width in cells' -r
complete -c ct -n "__fish_ct_using_subcommand tui; and __fish_seen_subcommand_from usage-bar" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tui; and __fish_seen_subcommand_from usage-bars" -l width -d 'Terminal width in cells' -r
complete -c ct -n "__fish_ct_using_subcommand tui; and __fish_seen_subcommand_from usage-bars" -l interval-ms -d 'Watch redraw interval in milliseconds' -r
complete -c ct -n "__fish_ct_using_subcommand tui; and __fish_seen_subcommand_from usage-bars" -l sidebar -d 'Render the tmux mux-sidebar layout'
complete -c ct -n "__fish_ct_using_subcommand tui; and __fish_seen_subcommand_from usage-bars" -l watch -d 'Continuously redraw and reload config changes'
complete -c ct -n "__fish_ct_using_subcommand tui; and __fish_seen_subcommand_from usage-bars" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand tui; and __fish_seen_subcommand_from help" -f -a "usage-bar" -d 'Render subscription usage bars from JSON on stdin'
complete -c ct -n "__fish_ct_using_subcommand tui; and __fish_seen_subcommand_from help" -f -a "usage-bars" -d 'Render subscription usage bars for all local providers'
complete -c ct -n "__fish_ct_using_subcommand tui; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand notify" -s h -l help -d 'Print help'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from apply-patch shell tui notify help" -f -a "apply-patch" -d 'Apply patches from stdin'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from apply-patch shell tui notify help" -f -a "shell" -d 'Shell integration helpers'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from apply-patch shell tui notify help" -f -a "tui" -d 'Terminal UI helpers'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from apply-patch shell tui notify help" -f -a "notify" -d 'Handle notification hooks'
complete -c ct -n "__fish_ct_using_subcommand help; and not __fish_seen_subcommand_from apply-patch shell tui notify help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from shell" -f -a "completion" -d 'Generate shell completion scripts'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tui" -f -a "usage-bar" -d 'Render subscription usage bars from JSON on stdin'
complete -c ct -n "__fish_ct_using_subcommand help; and __fish_seen_subcommand_from tui" -f -a "usage-bars" -d 'Render subscription usage bars for all local providers'
