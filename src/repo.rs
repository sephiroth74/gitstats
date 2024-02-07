use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use simple_cmd::{CommandBuilder, Vec8ToString};
use which::which;

use crate::{Author, CommitArgs, CommitDetail, CommitHash, CommitStats, Detail, Repo};

lazy_static! {
	static ref SHORT_STATS_RE: Regex = regex::Regex::new("(?<files>[\\d]+) files? changed(, (?<insertions>[\\d]+) insertions?\\(\\+\\))?(, (?<deletions>[\\d]+) deletions?\\(\\-\\))?$").unwrap();
	static ref NUMSTATS_RE: Regex = regex::Regex::new("^(?<additions>[\\d]+)\\s+(?<deletions>[\\d]+)\\s+(?<filename>[^\n]+)").unwrap();
	static ref SIZE_RE: Regex = regex::RegexBuilder::new(r#"^size-pack:\s*(?<size>[\d]+)$"#).multi_line(true).build().unwrap();
}

impl Repo {
	/// Create a new instance of a Repository
	/// # Examples:
	/// ```rust
	/// use gitstats::Repo;
	/// fn main() {
	///     let repo_dir = "#/custom/path/to/repo";
	///     let repo = Repo::from(&repo_dir);
	/// }
	/// ```
	pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> Self {
		Repo { inner: PathBuf::from(s) }
	}

	pub fn to_str(&self) -> Option<&str> {
		self.inner.to_str()
	}

	/// Fetch the repository using the default remote
	/// # Examples:
	/// ```rust
	/// use gitstats::Repo;
	///
	/// fn main() {
	///     let repo_dir = "/custom/path/to/repo";
	///     let repo = Repo::from(&repo_dir);
	///     match repo.fetch() {
	///         Ok(_) => println!("fetch complete"),
	///         Err(err) => println!("Error: {err}"),
	///     }
	/// }
	/// ```
	pub fn fetch(&self) -> anyhow::Result<()> {
		self.git()?
			.arg("fetch")
			.build()
			.output()
			.map(|_| ())
			.context("Failed to fetch remote")
	}

	/// Fetch all the remotes
	pub fn fetch_all(&self) -> anyhow::Result<()> {
		self.git()?
			.args([
				"fetch", "--all",
			])
			.build()
			.output()
			.map(|_| ())
			.context("Failed to fetch remotes")
	}

	/// Returns a list of commits based on the input arguments
	/// # Examples:
	/// ```rust
	/// use gitstats::Repo;
	/// use gitstats::CommitArgs;
	///
	/// fn main() {
	/// let repo_dir = "/custom/path/to/repo";
	///     let repo = Repo::from(&repo_dir);
	/// 	let commit_args = CommitArgs::default();
	///     match repo.list_commits(commit_args) {
	///         Ok(commits) => println!("got commits: {commits}"),
	///         Err(err) => println!("Error: {err}"),
	///     }
	/// }
	/// ```
	pub fn list_commits(&self, options: CommitArgs) -> anyhow::Result<Vec<CommitHash>> {
		options.validate()?;
		let mut command = self.git()?.arg("log");
		command = command.with_args(options).with_arg("--reverse");
		let output = command.build().output()?;
		Ok(output
			.stdout
			.lines()
			.filter_map(|line| if let Ok(line) = line { Some(CommitHash(line)) } else { None })
			.collect::<Vec<_>>())
	}

	pub fn first_commit(&self) -> anyhow::Result<Option<CommitDetail>> {
		let command = self.git()?.with_args(&[
			"rev-list",
			"--max-parents=0",
			"HEAD",
		]);
		let output = command.build().output()?;
		if let Some(commit) = output.stdout.as_str().map(|line| CommitHash(line.trim().to_string())) {
			Ok(Some(self.commit_stats(commit)?))
		} else {
			Ok(None)
		}
	}

	pub fn last_commit(&self) -> anyhow::Result<Option<CommitDetail>> {
		let command = self.git()?.with_args(&[
			"rev-list", "-n", "1", "HEAD",
		]);
		let output = command.build().output()?;
		if let Some(commit) = output.stdout.as_str().map(|line| CommitHash(line.trim().to_string())) {
			Ok(Some(self.commit_stats(commit)?))
		} else {
			Ok(None)
		}
	}

	/// Return the repository size (in Kilobytes)
	pub fn size(&self) -> anyhow::Result<u64> {
		let command = self.git()?.with_args(&[
			"count-objects",
			"-v",
		]);
		let output = command.build().output()?;
		let string = output
			.stdout
			.as_str()
			.ok_or(anyhow!("failed to find repository size"))?
			.trim();
		if let Some(find) = SIZE_RE.captures(string) {
			let size_string = find.name("size").unwrap().as_str();
			let size: u64 = size_string.parse::<u64>()?;
			Ok(size)
		} else {
			Err(anyhow::Error::msg("failed to find repository size"))
		}
	}

	/// Returns the total commits
	pub fn commits_count(&self) -> anyhow::Result<usize> {
		let command = self.git()?.with_args(&[
			"rev-list", "--count", "--all",
		]);
		let output = command.build().output()?;
		let string = output.stdout.lines().nth(0).ok_or(anyhow!("failed to get total commits"))??;
		Ok(string.parse::<usize>()?)
	}

	pub fn details(&self) -> anyhow::Result<Detail> {
		let size = self.size()?;
		let first_commit = self.first_commit()?;
		let last_commit = self.last_commit()?;
		let commits_count = self.commits_count()?;
		Ok(Detail {
			size,
			commits_count,
			first_commit: first_commit.map(|c| c.author_timestamp),
			last_commit: last_commit.map(|c| c.author_timestamp),
		})
	}

	/// Extract details from a list of commits
	/// # Examples:
	/// ```rust
	///
	/// use gitstats::Repo;
	/// use gitstats::CommitArgs;
	///
	/// fn main() {
	/// let repo_dir = "/custom/path/to/repo";
	///     let repo = Repo::from(&repo_dir);
	/// 	let commit_args = CommitArgs::default();
	///     if let Ok(commits) = repo.list_commits(commit_args) {
	/// 		let stats = repo.commits_stats(&commits);
	///     }
	/// }
	///
	/// ```
	pub fn commits_stats(&self, commits: &Vec<CommitHash>) -> anyhow::Result<Vec<CommitDetail>> {
		commits
			.into_par_iter()
			.map(|commit| self.commit_stats(commit.to_owned()))
			.collect()
	}

	/// Extract details from a commit hash
	pub fn commit_stats(&self, commit: CommitHash) -> anyhow::Result<CommitDetail> {
		let mut command = self.git()?.with_debug(false);
		let hash: &str = (&commit).into();

		command = command
			.arg("show")
			.arg("--shortstat")
			.arg("--pretty=\"format:%H\n%aN\n%aE\n%at\n\"")
			.arg(hash);

		let result = command.build().output()?;
		let output = result.stdout;
		let lines = output.lines().map(|f| f.unwrap()).collect::<Vec<String>>();
		let size = lines.len();

		let mut commit_hash: Option<String> = None;
		let mut author_name: Option<String> = None;
		let mut author_email: Option<String> = None;
		let mut author_date: Option<i64> = None;

		for index in 0..size {
			let line = &lines[index];

			match index {
				0 => commit_hash = Some(line.to_string()),
				1 => author_name = Some(line.to_string()),
				2 => author_email = Some(line.to_string()),
				3 => {
					let timestamp = line.parse::<i64>().expect("invalid timestamp");
					author_date = Some(timestamp);
				}
				_ => {
					// unexpected
				}
			}
		}

		let mut files: u32 = 0;
		let mut insertions: u32 = 0;
		let mut deletions: u32 = 0;

		if let Some(find) = SHORT_STATS_RE.captures(lines.last().ok_or(anyhow!("failed to find last line"))?.as_str()) {
			files = find.name("files").map_or(0, |f| f.as_str().parse::<u32>().unwrap_or(0));
			insertions = find.name("insertions").map_or(0, |f| f.as_str().parse::<u32>().unwrap_or(0));
			deletions = find.name("deletions").map_or(0, |f| f.as_str().parse::<u32>().unwrap_or(0));
		}

		if commit_hash.is_none() {
			return Err(anyhow!("commit hash not found"));
		} else if author_name.is_none() {
			return Err(anyhow!("author name not found"));
		} else if author_email.is_none() {
			return Err(anyhow!("author email not found"));
		} else if author_date.is_none() {
			return Err(anyhow!("author datetime not found"));
		}

		let stats = CommitStats {
			files_changed: files,
			lines_added: insertions,
			lines_deleted: deletions,
		};

		let commit = CommitDetail {
			hash: commit,
			author: Author::new(author_name.unwrap()).with_email_opt(author_email.as_deref()),
			author_timestamp: author_date.unwrap(),
			stats,
		};

		Ok(commit)
	}

	/// Will panic is git is not found
	fn git(&self) -> anyhow::Result<CommandBuilder> {
		let git = which("git")?;
		//Ok(CommandBuilder::new(git).current_dir(&self.inner).with_debug(true))
		Ok(CommandBuilder::new(git).with_debug(true).with_arg("-C").with_arg(&self.inner))
	}
}

impl<'a, T: ?Sized + AsRef<OsStr>> From<&'a T> for Repo {
	fn from(s: &'a T) -> Self {
		Repo::new(s)
	}
}

impl Display for Repo {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.inner)
	}
}
