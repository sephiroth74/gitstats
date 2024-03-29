use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

mod impls;
mod repo;
mod test;
pub mod traits;

#[derive(Debug, Clone)]
pub struct Repo {
	inner: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitHash(String);

#[derive(Debug, Default, Hash, Clone, Serialize, Deserialize)]
pub struct Author {
	pub name: String,
	pub email: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CommitArgs {
	since: Option<i64>,
	until: Option<i64>,
	author: Option<Author>,
	exclude_merges: bool,
	exclude_author: Option<String>,
	target_branch: Option<String>,
}

pub struct CommitArgsBuilder(CommitArgs);

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CommitStats {
	pub files_changed: u32,
	pub lines_added: u32,
	pub lines_deleted: u32,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct CommitDetail {
	pub hash: CommitHash,
	pub author: Author,
	pub author_timestamp: i64,
	pub stats: CommitStats,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct MinimalCommitDetail {
	pub hash: CommitHash,
	pub author_timestamp: i64,
	pub stats: CommitStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlobalStat {
	pub author: Author,
	pub commits_count: usize,
	pub stats: CommitStats,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SimpleStat {
	pub commits_count: usize,
	pub stats: CommitStats,
}

pub enum SortStatsBy {
	Commits,
	FilesChanged,
	LinesAdded,
	LinesDeleted,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitsPerAuthor(pub(crate) HashMap<Author, Vec<MinimalCommitDetail>>);

#[derive(Debug, Clone, Serialize)]
pub struct CommitsPerWeekday(pub(crate) HashMap<u8, HashMap<Author, SimpleStat>>);

#[derive(Debug, Clone, Serialize)]
pub struct CommitsPerDayHour(pub(crate) HashMap<u32, HashMap<Author, SimpleStat>>);

#[derive(Debug, Clone, Serialize)]
pub struct CommitsPerMonth(pub(crate) HashMap<String, HashMap<Author, SimpleStat>>);

///
/// Contains an hashmap where the key is the Author and the value is a matrix[weekday, hour] of stats
#[derive(Debug, Clone, Serialize)]
pub struct CommitsHeatMap(pub(crate) HashMap<Author, Vec<Vec<SimpleStat>>>);

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Detail {
	/// repository size in Kilobytes
	pub size: u64,
	/// total commits
	pub commits_count: usize,
	// first commit timestamp
	pub first_commit: Option<i64>,
	// last commit timestamp
	pub last_commit: Option<i64>,
}
