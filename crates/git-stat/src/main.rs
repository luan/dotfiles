use std::{
    collections::HashSet,
    env,
    ffi::{OsStr, OsString},
    io::{self, BufRead, BufReader, IsTerminal, Write},
    process::{Command, Stdio},
};

use clap::Parser;
use serde::Deserialize;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const BROKEN_PIPE: &str = "output closed";

#[derive(Parser, Debug)]
#[command(
    name = "git stat",
    about = "A compact, colorful view of commit and pull request statistics",
    after_help = "Examples:\n  git stat                 Recent commits\n  git stat -f -n 5         Include files for five commits\n  git stat -c              Current branch relative to origin's default branch\n  git stat -u              Current branch relative to its upstream\n  git stat 123             Pull request #123\n  git stat --stack [123]   Current stack, or the stack containing PR #123"
)]
struct Cli {
    /// Compare the current branch with origin's default branch
    #[arg(short = 'c', conflicts_with = "upstream")]
    current: bool,

    /// Compare the current branch with its upstream
    #[arg(short = 'u', conflicts_with = "current")]
    upstream: bool,

    /// Show per-file statistics
    #[arg(short = 'f')]
    files: bool,

    /// Show the current stack; optionally pass a pull request number
    #[arg(long)]
    stack: bool,

    /// Arguments passed through to git log, or a pull request number
    #[arg(
        value_name = "GIT_LOG_ARG",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    args: Vec<OsString>,
}

#[derive(Debug)]
enum Mode {
    Log { range: Option<String>, total: bool },
    PullRequest(u64),
    Stack(Option<u64>),
}

#[derive(Clone, Debug, Default)]
struct Stats {
    files: Vec<FileStat>,
    additions: u64,
    deletions: u64,
}

#[derive(Clone, Debug)]
struct FileStat {
    path: String,
    additions: u64,
    deletions: u64,
}

#[derive(Deserialize)]
struct PullRequestInfo {
    number: u64,
    title: String,
}

#[derive(Deserialize)]
struct PullRequestFile {
    filename: String,
    additions: u64,
    deletions: u64,
}

#[derive(Deserialize)]
struct RefValue {
    #[serde(rename = "ref")]
    name: String,
}

#[derive(Deserialize)]
struct RemotePullRequest {
    number: u64,
    state: String,
    merged_at: Option<String>,
    head: RemoteHead,
}

#[derive(Deserialize)]
struct RemoteHead {
    #[serde(rename = "ref")]
    name: String,
    sha: String,
}

#[derive(Deserialize)]
struct RemoteStack {
    number: u64,
    base: RefValue,
    pull_requests: Vec<RemotePullRequest>,
}

#[derive(Clone, Deserialize)]
struct StackBranch {
    name: String,
    #[serde(default)]
    base: String,
    #[serde(default)]
    head: String,
    #[serde(default, rename = "isCurrent")]
    is_current: bool,
    #[serde(default, rename = "isSelected")]
    is_selected: bool,
    #[serde(default, rename = "isMerged")]
    is_merged: bool,
    pr: Option<StackPr>,
}

#[derive(Clone, Deserialize)]
struct StackPr {
    number: u64,
    #[allow(dead_code)]
    state: Option<String>,
}

#[derive(Deserialize)]
struct StackView {
    #[serde(default, rename = "stackNumber")]
    stack_number: Option<u64>,
    trunk: String,
    branches: Vec<StackBranch>,
}

struct Paint {
    enabled: bool,
}

impl Paint {
    fn wrap(&self, style: &str, value: impl AsRef<str>) -> String {
        if self.enabled {
            format!("{style}{}{RESET}", value.as_ref())
        } else {
            value.as_ref().to_owned()
        }
    }
}

fn main() {
    let raw_args: Vec<_> = env::args_os().collect();
    let mut cli = Cli::parse_from(&raw_args);
    restore_git_delimiter(&raw_args[1..], &mut cli.args);
    if let Err(error) = run(cli)
        && error != BROKEN_PIPE
    {
        eprintln!("git stat: {error}");
        std::process::exit(1);
    }
}

fn restore_git_delimiter(raw_args: &[OsString], parsed_args: &mut Vec<OsString>) {
    let Some(position) = raw_args.iter().position(|arg| arg == "--") else {
        return;
    };
    if parsed_args.iter().any(|arg| arg == "--") {
        return;
    }
    let values_after_delimiter = raw_args.len() - position - 1;
    parsed_args.insert(parsed_args.len() - values_after_delimiter, "--".into());
}

fn run(cli: Cli) -> Result<(), String> {
    let mode = select_mode(&cli)?;
    let paint = Paint {
        enabled: io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none(),
    };
    match mode {
        Mode::Log { range, total } => {
            with_pager(|output| render_log(&cli, range.as_deref(), total, &paint, output))
        }
        Mode::PullRequest(number) => page(render_pull_request(number, cli.files, &paint)?),
        Mode::Stack(number) => page(render_stack(number, cli.files, &paint)?),
    }
}

fn select_mode(cli: &Cli) -> Result<Mode, String> {
    if cli.stack {
        if cli.current || cli.upstream {
            return Err("--stack cannot be combined with -c or -u".into());
        }
        let number = optional_pr_number(&cli.args, "--stack accepts at most one PR number")?;
        return Ok(Mode::Stack(number));
    }

    if cli.current {
        let base = command_text(
            "git",
            [
                "symbolic-ref",
                "--quiet",
                "--short",
                "refs/remotes/origin/HEAD",
            ],
        )
        .map_err(|_| "origin/HEAD is not set".to_string())?;
        let merge_base = command_text("git", ["merge-base", "HEAD", base.trim()])?;
        return Ok(Mode::Log {
            range: Some(format!("{}..HEAD", merge_base.trim())),
            total: true,
        });
    }

    if cli.upstream {
        command_text("git", ["rev-parse", "--verify", "@{upstream}"])
            .map_err(|_| "current branch has no upstream".to_string())?;
        return Ok(Mode::Log {
            range: Some("@{upstream}..HEAD".into()),
            total: false,
        });
    }

    if cli.args.first().and_then(|arg| parse_number(arg)).is_some() {
        let number = optional_pr_number(&cli.args, "PR mode does not accept git log arguments")?
            .expect("the first argument was numeric");
        Ok(Mode::PullRequest(number))
    } else {
        Ok(Mode::Log {
            range: None,
            total: false,
        })
    }
}

fn optional_pr_number(args: &[OsString], error: &str) -> Result<Option<u64>, String> {
    match args {
        [] => Ok(None),
        [value] => parse_number(value).map(Some).ok_or_else(|| error.into()),
        _ => Err(error.into()),
    }
}

fn parse_number(value: &OsStr) -> Option<u64> {
    let value = value.to_str()?;
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

fn render_log(
    cli: &Cli,
    range: Option<&str>,
    total: bool,
    paint: &Paint,
    output: &mut dyn Write,
) -> Result<(), String> {
    let mut command = Command::new("git");
    command.args(["log", "--pretty=format:%x1e%h%x1f%s%x1f%cr"]);
    command.arg(if cli.files {
        "--numstat"
    } else {
        "--shortstat"
    });
    if let Some(range) = range {
        command.arg(range);
    }
    command.args(&cli.args);
    let display = command_display(&command);
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("could not run {display}: {error}"))?;
    let stdout = child.stdout.take().expect("piped stdout");
    let mut reader = BufReader::new(stdout);
    let mut record = Vec::new();
    while reader
        .read_until(b'\x1e', &mut record)
        .map_err(|error| format!("could not read {display}: {error}"))?
        != 0
    {
        if record.last() == Some(&b'\x1e') {
            record.pop();
        }
        if !record.iter().all(u8::is_ascii_whitespace) {
            render_log_record(&record, cli.files, paint, output)?;
            output.flush().map_err(write_error)?;
        }
        record.clear();
    }
    let status = child
        .wait()
        .map_err(|error| format!("could not wait for {display}: {error}"))?;
    if !status.success() {
        return Err(format!("{display} exited with {status}"));
    }

    if total {
        let range = range.expect("branch mode always has a range");
        let stats = diff_stats(range)?;
        writeln!(
            output,
            "\n{}{}",
            paint.wrap(&format!("{YELLOW}{BOLD}"), "Branch total"),
            render_summary(&stats, paint)
        )
        .map_err(write_error)?;
    }
    Ok(())
}

fn render_log_record(
    record: &[u8],
    show_files: bool,
    paint: &Paint,
    output: &mut dyn Write,
) -> Result<(), String> {
    let record =
        std::str::from_utf8(record).map_err(|_| "git log returned non-UTF-8 output".to_string())?;
    let mut fields = record.trim_matches('\n').splitn(3, '\x1f');
    let hash = fields.next().unwrap_or_default();
    let subject = fields.next().unwrap_or_default();
    let tail = fields.next().unwrap_or_default();
    let (age, stats) = parse_log_tail(tail, show_files);
    writeln!(
        output,
        "{}{}",
        commit_header(hash, subject, age, paint),
        render_summary(&stats, paint)
    )
    .map_err(write_error)?;
    if show_files {
        output
            .write_all(render_files(&stats.files, paint).as_bytes())
            .map_err(write_error)?;
    }
    Ok(())
}

fn parse_log_tail(tail: &str, show_files: bool) -> (&str, Stats) {
    let mut lines = tail.lines();
    let age = lines.next().unwrap_or_default();
    let stats = if show_files {
        stats_from_files(lines.filter_map(parse_numstat).collect())
    } else {
        lines.find_map(parse_shortstat).unwrap_or_default()
    };
    (age, stats)
}

fn parse_shortstat(line: &str) -> Option<Stats> {
    if !line.contains(" changed") {
        return None;
    }
    let mut stats = Stats::default();
    for part in line.split(',') {
        let mut words = part.split_whitespace();
        let count = words.next()?.parse::<u64>().ok()?;
        match words.next()? {
            "file" | "files" => {
                stats.files = (0..count)
                    .map(|_| FileStat {
                        path: String::new(),
                        additions: 0,
                        deletions: 0,
                    })
                    .collect();
            }
            "insertion(+)" | "insertions(+)" => stats.additions = count,
            "deletion(-)" | "deletions(-)" => stats.deletions = count,
            _ => {}
        }
    }
    Some(stats)
}

fn parse_numstat(line: &str) -> Option<FileStat> {
    let mut fields = line.splitn(3, '\t');
    let additions = parse_count(fields.next()?)?;
    let deletions = parse_count(fields.next()?)?;
    let path = fields.next()?.to_owned();
    Some(FileStat {
        path,
        additions,
        deletions,
    })
}

fn parse_count(value: &str) -> Option<u64> {
    if value == "-" {
        Some(0)
    } else {
        value.parse().ok()
    }
}

fn stats_from_files(files: Vec<FileStat>) -> Stats {
    Stats {
        additions: files.iter().map(|file| file.additions).sum(),
        deletions: files.iter().map(|file| file.deletions).sum(),
        files,
    }
}

fn commit_header(hash: &str, subject: &str, age: &str, paint: &Paint) -> String {
    let subject = truncate(subject, 60);
    format!(
        "{} {} {}",
        paint.wrap(YELLOW, hash),
        paint.wrap(BOLD, format!("{subject:<60}")),
        paint.wrap(MAGENTA, format!("{:>20}", format!("({age})")))
    )
}

fn render_pull_request(number: u64, show_files: bool, paint: &Paint) -> Result<String, String> {
    let info: PullRequestInfo =
        gh_json(["pr", "view", &number.to_string(), "--json", "number,title"])?;
    let stats = pull_request_stats(number)?;
    let mut output = paint.wrap(
        &format!("{CYAN}{BOLD}"),
        format!("#{} {}", info.number, info.title.replace('\t', " ")),
    );
    output.push_str(&render_summary(&stats, paint));
    output.push('\n');
    if show_files {
        output.push_str(&render_files(&stats.files, paint));
    }
    Ok(output)
}

fn render_stack(number: Option<u64>, show_files: bool, paint: &Paint) -> Result<String, String> {
    let (mut stack, remote) = match number {
        Some(selected) => (remote_stack(selected)?, true),
        None => (gh_json(["stack", "view", "--json"])?, false),
    };
    if stack.branches.is_empty() {
        return Err("stack has no branches".into());
    }

    fill_stack_bounds(&mut stack)?;
    let width = stack
        .branches
        .iter()
        .map(|branch| branch.name.chars().count())
        .max()
        .unwrap_or(0)
        + 22;
    let mut output = String::new();
    let mut stack_files = HashSet::new();
    let mut stack_additions = 0;
    let mut stack_deletions = 0;

    if let Some(number) = stack.stack_number {
        output.push_str(&paint.wrap(MAGENTA, format!("Stack #{number}")));
        output.push_str("\n\n");
    }

    for branch in stack
        .branches
        .iter()
        .rev()
        .filter(|branch| !branch.is_merged)
    {
        render_stack_branch(
            branch,
            width,
            remote,
            show_files,
            paint,
            &mut output,
            &mut stack_files,
            &mut stack_additions,
            &mut stack_deletions,
        )?;
    }

    let merged: Vec<_> = stack
        .branches
        .iter()
        .rev()
        .filter(|branch| branch.is_merged)
        .collect();
    if !merged.is_empty() {
        output.push_str(&paint.wrap(MAGENTA, "├─── merged ────"));
        output.push_str("\n\n");
        for branch in merged {
            render_stack_branch(
                branch,
                width,
                true,
                show_files,
                paint,
                &mut output,
                &mut stack_files,
                &mut stack_additions,
                &mut stack_deletions,
            )?;
        }
    }

    output.push_str(&paint.wrap(MAGENTA, format!("└ {}", stack.trunk)));
    output.push_str("\n\n");
    let total = if remote {
        Stats {
            files: stack_files
                .into_iter()
                .map(|path| FileStat {
                    path,
                    additions: 0,
                    deletions: 0,
                })
                .collect(),
            additions: stack_additions,
            deletions: stack_deletions,
        }
    } else {
        let first = stack.branches.first().expect("checked non-empty");
        let last = stack.branches.last().expect("checked non-empty");
        diff_stats(&format!("{}..{}", first.base, last.head))?
    };
    output.push_str(&paint.wrap(&format!("{YELLOW}{BOLD}"), "Stack total"));
    output.push_str(&render_summary(&total, paint));
    output.push('\n');
    Ok(output)
}

// Keep the branch renderer flat; introduce a context struct if it gains another output mode.
#[allow(clippy::too_many_arguments)]
fn render_stack_branch(
    branch: &StackBranch,
    width: usize,
    remote: bool,
    show_files: bool,
    paint: &Paint,
    output: &mut String,
    stack_files: &mut HashSet<String>,
    stack_additions: &mut u64,
    stack_deletions: &mut u64,
) -> Result<(), String> {
    let pr = branch
        .pr
        .as_ref()
        .ok_or_else(|| format!("{} has no pull request", branch.name))?
        .number;
    let (prefix, suffix, style) = if branch.is_current {
        ("»", " (current)", format!("{CYAN}{BOLD}"))
    } else if branch.is_selected {
        ("»", " (selected)", format!("{CYAN}{BOLD}"))
    } else if branch.is_merged {
        ("│", "", MAGENTA.to_owned())
    } else {
        ("├", "", BOLD.to_owned())
    };
    let status = if branch.is_merged { "✓" } else { "○" };
    let label = format!("{prefix} {} {status} #{pr}{suffix}", branch.name);
    let stats = if remote || branch.is_merged {
        pull_request_stats(pr)?
    } else {
        diff_stats(&format!("{}..{}", branch.base, branch.head))?
    };
    output.push_str(&paint.wrap(&style, format!("{label:<width$}")));
    output.push_str(&render_summary(&stats, paint));
    output.push('\n');
    if show_files {
        output.push_str(&render_files(&stats.files, paint));
    }
    output.push('\n');
    for file in &stats.files {
        stack_files.insert(file.path.clone());
    }
    *stack_additions += stats.additions;
    *stack_deletions += stats.deletions;
    Ok(())
}

fn remote_stack(selected: u64) -> Result<StackView, String> {
    let endpoint = format!("repos/{{owner}}/{{repo}}/stacks?pull_request={selected}");
    let stacks: Vec<RemoteStack> = gh_json(["api", endpoint.as_str()])?;
    let stack = stacks
        .into_iter()
        .next()
        .ok_or_else(|| format!("PR #{selected} is not part of a stack"))?;
    let mut previous = String::new();
    let branches = stack
        .pull_requests
        .into_iter()
        .map(|pr| {
            let branch = StackBranch {
                name: pr.head.name,
                base: previous.clone(),
                head: pr.head.sha.clone(),
                is_current: false,
                is_selected: pr.number == selected,
                is_merged: pr.merged_at.is_some(),
                pr: Some(StackPr {
                    number: pr.number,
                    state: Some(pr.state),
                }),
            };
            previous = pr.head.sha;
            branch
        })
        .collect();
    Ok(StackView {
        stack_number: Some(stack.number),
        trunk: stack.base.name,
        branches,
    })
}

fn fill_stack_bounds(stack: &mut StackView) -> Result<(), String> {
    let first = stack
        .branches
        .first_mut()
        .ok_or_else(|| "stack has no branches".to_string())?;
    if first.base.is_empty() {
        let pr = first
            .pr
            .as_ref()
            .ok_or_else(|| "first stack branch has no pull request".to_string())?
            .number;
        first.base = gh_field(pr, "baseRefOid")?;
    }
    for branch in &mut stack.branches {
        if branch.head.is_empty() {
            branch.head.clone_from(&branch.name);
        }
    }
    Ok(())
}

fn gh_field(number: u64, field: &str) -> Result<String, String> {
    let output = command_text(
        "gh",
        [
            "pr",
            "view",
            &number.to_string(),
            "--json",
            field,
            "--jq",
            &format!(".{field}"),
        ],
    )?;
    Ok(output.trim().to_owned())
}

fn pull_request_stats(number: u64) -> Result<Stats, String> {
    let endpoint = format!("repos/{{owner}}/{{repo}}/pulls/{number}/files?per_page=100");
    let pages = command_text("gh", ["api", "--paginate", "--slurp", endpoint.as_str()])?;
    let page_files: Vec<Vec<PullRequestFile>> =
        serde_json::from_str(&pages).map_err(|error| format!("invalid gh response: {error}"))?;
    Ok(stats_from_files(
        page_files
            .into_iter()
            .flatten()
            .map(|file| FileStat {
                path: file.filename,
                additions: file.additions,
                deletions: file.deletions,
            })
            .collect(),
    ))
}

fn diff_stats(range: &str) -> Result<Stats, String> {
    let raw = command_text("git", ["diff", "--numstat", range])?;
    Ok(stats_from_files(
        raw.lines().filter_map(parse_numstat).collect(),
    ))
}

fn render_summary(stats: &Stats, paint: &Paint) -> String {
    let files = stats.files.len();
    let file_word = if files == 1 { "file" } else { "files" };
    let mut output = format!(
        " │ {}",
        paint.wrap(CYAN, format!("{files:>2} {file_word} changed"))
    );
    if stats.additions > 0 {
        let word = if stats.additions == 1 {
            "insertion"
        } else {
            "insertions"
        };
        output.push_str(&format!(
            ", {}",
            paint.wrap(
                GREEN,
                format!(
                    "{} {word}({})",
                    stats.additions,
                    bar('+', stats.additions, stats.deletions, 30)
                )
            )
        ));
    }
    if stats.deletions > 0 {
        let word = if stats.deletions == 1 {
            "deletion"
        } else {
            "deletions"
        };
        output.push_str(&format!(
            ", {}",
            paint.wrap(
                RED,
                format!(
                    "{} {word}({})",
                    stats.deletions,
                    bar('-', stats.deletions, stats.additions, 30)
                )
            )
        ));
    }
    output
}

fn render_files(files: &[FileStat], paint: &Paint) -> String {
    let width = files
        .iter()
        .map(|file| file.path.chars().count())
        .max()
        .unwrap_or(0);
    let mut output = String::new();
    for file in files {
        let total = file.additions + file.deletions;
        output.push_str(&format!(
            "  {:width$} │ {total:>4} {}{}\n",
            file.path,
            paint.wrap(GREEN, "+".repeat(scaled(file.additions, total, 60))),
            paint.wrap(RED, "-".repeat(scaled(file.deletions, total, 60))),
        ));
    }
    output
}

fn bar(character: char, value: u64, other: u64, width: usize) -> String {
    character
        .to_string()
        .repeat(scaled(value, value + other, width))
}

fn scaled(value: u64, total: u64, width: usize) -> usize {
    if value == 0 {
        0
    } else if total <= width as u64 {
        value as usize
    } else {
        ((value as f64 / total as f64 * width as f64).round() as usize).max(1)
    }
}

fn truncate(value: &str, width: usize) -> String {
    let mut chars = value.chars();
    let result: String = chars.by_ref().take(width).collect();
    if chars.next().is_some() && width > 1 {
        format!("{}…", result.chars().take(width - 1).collect::<String>())
    } else {
        result
    }
}

fn gh_json<T, I, S>(args: I) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let text = command_text("gh", args)?;
    serde_json::from_str(&text).map_err(|error| format!("invalid gh response: {error}"))
}

fn command_text<I, S>(program: &str, args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.args(args);
    command_output(&mut command)
}

fn command_output(command: &mut Command) -> Result<String, String> {
    let display = command_display(command);
    let output = command
        .output()
        .map_err(|error| format!("could not run {display}: {error}"))?;
    if !output.status.success() {
        io::stderr().write_all(&output.stderr).ok();
        return Err(format!("{display} exited with {}", output.status));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("{display} returned non-UTF-8 output"))
}

fn page(output: String) -> Result<(), String> {
    with_pager(|writer| writer.write_all(output.as_bytes()).map_err(write_error))
}

fn write_error(error: io::Error) -> String {
    if error.kind() == io::ErrorKind::BrokenPipe {
        BROKEN_PIPE.into()
    } else {
        error.to_string()
    }
}

fn command_display(command: &Command) -> String {
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}

fn with_pager(render: impl FnOnce(&mut dyn Write) -> Result<(), String>) -> Result<(), String> {
    if io::stdout().is_terminal()
        && let Ok(mut child) = Command::new("less")
            .arg("-FRX")
            .env("LESSCHARSET", "utf-8")
            .stdin(Stdio::piped())
            .spawn()
    {
        let mut stdin = child.stdin.take().expect("piped stdin");
        let result = render(&mut stdin);
        drop(stdin);
        let status = child.wait().map_err(|error| error.to_string())?;
        result?;
        if !status.success() {
            return Err(format!("less exited with {status}"));
        }
        return Ok(());
    }
    render(&mut io::stdout().lock())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numstat() {
        let file = parse_numstat("12\t3\tsrc/main.rs").unwrap();
        assert_eq!(file.path, "src/main.rs");
        assert_eq!(file.additions, 12);
        assert_eq!(file.deletions, 3);
    }

    #[test]
    fn parses_shortstat() {
        let stats =
            parse_shortstat(" 17 files changed, 153 insertions(+), 65 deletions(-)").unwrap();
        assert_eq!(stats.files.len(), 17);
        assert_eq!(stats.additions, 153);
        assert_eq!(stats.deletions, 65);
    }

    #[test]
    fn counts_binary_numstat_as_a_changed_file() {
        let file = parse_numstat("-\t-\timage.png").unwrap();
        assert_eq!(file.path, "image.png");
        assert_eq!(file.additions, 0);
        assert_eq!(file.deletions, 0);
    }

    #[test]
    fn truncates_unicode_by_character() {
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("🦀 rust", 3), "🦀 …");
    }

    #[test]
    fn scales_nonzero_changes_to_at_least_one_character() {
        assert_eq!(scaled(1, 1_000, 30), 1);
        assert_eq!(scaled(0, 1_000, 30), 0);
    }

    #[test]
    fn restores_git_path_delimiter() {
        let raw = ["-n", "2", "--", "main"].map(OsString::from);
        let mut parsed = ["-n", "2", "main"].map(OsString::from).to_vec();
        restore_git_delimiter(&raw, &mut parsed);
        assert_eq!(
            parsed,
            ["-n", "2", "--", "main"].map(OsString::from).to_vec()
        );
    }
}
