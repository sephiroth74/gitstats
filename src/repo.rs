use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use simple_cmd::CommandBuilder;

use crate::{Author, CommitArgs, CommitDetail, CommitHash, CommitStats, Repo};

lazy_static! {
	static ref SHORT_STATS_RE: Regex = regex::Regex::new("(?<files>[\\d]+) files? changed(, (?<insertions>[\\d]+) insertions?\\(\\+\\))?(, (?<deletions>[\\d]+) deletions?\\(\\-\\))?$").unwrap();
	static ref NUMSTATS_RE: Regex = regex::Regex::new("^(?<additions>[\\d]+)\\s+(?<deletions>[\\d]+)\\s+(?<filename>[^\n]+)").unwrap();
}

impl Repo {
	pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> Self {
		Repo { inner: PathBuf::from(s) }
	}

	pub fn to_str(&self) -> Option<&str> {
		self.inner.to_str()
	}

	pub fn fetch(&self) -> anyhow::Result<()> {
		self.git()
			.arg("fetch")
			.build()
			.output()
			.map(|_| ())
			.context("Failed to fetch remote")
	}

	pub fn fetch_all(&self) -> anyhow::Result<()> {
		self.git()
			.args([
				"fetch", "--all",
			])
			.build()
			.output()
			.map(|_| ())
			.context("Failed to fetch remotes")
	}

	pub fn list_commits(&self, options: CommitArgs) -> anyhow::Result<Vec<CommitHash>> {
		options.validate()?;
		let mut command = self.git().arg("log");
		command = command.with_args(options).with_arg("--reverse");
		let output = command.build().output()?;
		Ok(output
			.stdout
			.lines()
			.filter_map(|line| if let Ok(line) = line { Some(CommitHash(line)) } else { None })
			.collect::<Vec<_>>())
	}

	pub fn commits_stats<'c>(&self, commits: &'c Vec<CommitHash>) -> anyhow::Result<Vec<CommitDetail<'c>>> {
		commits.into_par_iter().map(|commit| self.commit_stats(commit)).collect()
	}

	pub fn commit_stats<'c>(&self, commit: &'c CommitHash) -> anyhow::Result<CommitDetail<'c>> {
		let mut command = self.git().with_debug(false);
		let hash: &str = commit.into();

		command = command
			.arg("show")
			.arg("--shortstat")
			.arg("--pretty=format:%H\n%aN\n%aE\n%at\n%s")
			.arg(hash);

		let result = command.build().output()?;
		let output = result.stdout;
		let lines = output.lines().map(|f| f.unwrap()).collect::<Vec<String>>();
		let size = lines.len();

		let mut commit_hash: Option<String> = None;
		let mut author_name: Option<String> = None;
		let mut author_email: Option<String> = None;
		let mut author_date: Option<i64> = None;
		let mut commit_subject: Option<String> = None;

		for index in 0..size - 1 {
			let line = &lines[index];

			match index {
				0 => commit_hash = Some(line.to_string()),
				1 => author_name = Some(line.to_string()),
				2 => author_email = Some(line.to_string()),
				3 => {
					let timestamp = line.parse::<i64>().expect("invalid timestamp");
					author_date = Some(timestamp);
				}
				4 => commit_subject = Some(line.to_string()),
				_ => {
					// unexpected
				}
			}
		}

		let mut files: u32 = 0;
		let mut insertions: u32 = 0;
		let mut deletions: u32 = 0;

		if let Some(find) = SHORT_STATS_RE.captures(lines.last().ok_or(anyhow!("failed to find short stats"))?.as_str()) {
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
			subject: commit_subject.unwrap(),
			author_timestamp: author_date.unwrap(),
			stats,
		};

		Ok(commit)
	}

	fn git(&self) -> CommandBuilder {
		CommandBuilder::new("git").current_dir(&self.inner).with_debug(true)
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
