use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};

use anyhow::{anyhow, Context};
use chrono::{DateTime, Datelike, Months, NaiveDateTime, Timelike, Utc, Weekday};
use lazy_static::lazy_static;

use crate::traits::{CommitStatsExt, GlobalStatsExt};
use crate::{
	Author, CommitArgs, CommitArgsBuilder, CommitDetail, CommitHash, CommitStats, CommitsPerDayHour, CommitsPerWeekday, GlobalStat,
	MinimalCommitDetail, SimpleStat, SortStatsBy,
};

lazy_static! {
	static ref AUTHOR_STR_RE: regex::Regex = regex::Regex::new("^(?:\"?([^\"]*)\"?\\s)?(?:<?(.+@[^>]+)?>?)$").unwrap();
}

// region Author

impl Author {
	pub fn new<T: Into<String>>(name: T) -> Self {
		Author {
			name: name.into(),
			email: None,
		}
	}

	pub fn with_email(mut self, email: &str) -> Self {
		self.email = Some(email.to_string());
		self
	}

	pub fn with_email_opt(mut self, email: Option<&str>) -> Self {
		if let Some(email) = email {
			self.email = Some(email.to_string());
		} else {
			self.email = None;
		}
		self
	}

	pub fn from(other: &Author) -> Self {
		Author {
			name: other.name.to_string(),
			email: other.email.clone(),
		}
	}
}

impl<'a> TryFrom<&'a str> for Author {
	type Error = anyhow::Error;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		let find = AUTHOR_STR_RE
			.captures(value)
			.ok_or(anyhow!("failed to parse author string. Got {:}", value))?;

		if find.len() == 3 {
			let name = find
				.get(1)
				.ok_or(anyhow!("failed to extract author name from {:}", value))?
				.as_str();

			let email = find.get(2).map_or_else(|| None, |s| Some(s.as_str().to_string()));
			Ok(Author {
				name: name.to_string(),
				email,
			})
		} else {
			Err(anyhow!("invalid author mailmap"))
		}
	}
}

impl TryFrom<String> for Author {
	type Error = anyhow::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		value.as_str().try_into()
	}
}

impl PartialEq for Author {
	fn eq(&self, other: &Self) -> bool {
		let email_match = match &self.email {
			Some(e1) => match &other.email {
				Some(e2) => e1.eq_ignore_ascii_case(e2),
				None => false,
			},
			None => false,
		};

		self.name.eq_ignore_ascii_case(&other.name) || email_match
	}
}

impl Eq for Author {}

impl Display for Author {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if let Some(email) = &self.email {
			write!(f, "{} <{}>", self.name, email.as_str())
		} else {
			write!(f, "{} <>", self.name)
		}
	}
}

// endregion Author

// region CommitHash

impl Display for CommitHash {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl From<&str> for CommitHash {
	fn from(value: &str) -> Self {
		CommitHash(value.to_string())
	}
}

impl<'a> From<&'a CommitHash> for &'a str {
	fn from(value: &'a CommitHash) -> Self {
		&value.0
	}
}

// endregion CommitHash

// region CommitArgs

impl CommitArgsBuilder {
	pub fn since(mut self, value: i64) -> Self {
		self.0.since = Some(value);
		self
	}

	pub fn until(mut self, value: i64) -> Self {
		self.0.until = Some(value);
		self
	}

	pub fn exclude_merges(mut self, value: bool) -> Self {
		self.0.exclude_merges = value;
		self
	}

	pub fn target_branch(mut self, value: &str) -> Self {
		self.0.target_branch = Some(value.to_string());
		self
	}

	pub fn author(mut self, value: Author) -> Self {
		self.0.author = Some(value);
		self
	}

	pub fn exclude_author(mut self, value: String) -> Self {
		self.0.exclude_author = Some(value);
		self
	}

	pub fn build(self) -> anyhow::Result<CommitArgs> {
		self.0.validate()?;
		Ok(self.0)
	}
}

impl CommitArgs {
	pub fn builder() -> CommitArgsBuilder {
		CommitArgsBuilder(Default::default())
	}

	pub(crate) fn validate(&self) -> anyhow::Result<()> {
		if self.author.is_some() && self.exclude_author.is_some() {
			return Err(anyhow!("cannot specify both author and exclude_author"));
		}

		if let Some(since) = self.since {
			DateTime::from_timestamp(since, 0).context("invalid datetime specified for since")?;
		}

		if let Some(until) = self.until {
			DateTime::from_timestamp(until, 0).context("invalid datetime specified for until")?;
		}

		return Ok(());
	}
}

impl IntoIterator for CommitArgs {
	type Item = OsString;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		let mut args: Vec<OsString> = vec![];

		if let Some(target_branch) = self.target_branch {
			args.push(target_branch.into());
		} else {
			args.push("--all".into());
		}

		args.push("--pretty=%H".into());

		if let Some(since) = self.since {
			let datetime = DateTime::from_timestamp(since, 0).unwrap();
			args.push(format!("--since={:}", datetime.format("%Y-%m-%d").to_string()).into());
		}

		if let Some(until) = self.until {
			let datetime = DateTime::from_timestamp(until, 0).unwrap();
			args.push(format!("--until={:}", datetime.format("%Y-%m-%d").to_string()).into());
		}

		if let Some(author) = self.author.as_ref() {
			args.push(format!("--author={:}", author.name).into());
		}

		if self.exclude_merges {
			args.push("--no-merges".into());
		}

		if let Some(exclude_author) = self.exclude_author.as_ref() {
			args.push("--perl-regexp".into());
			args.push(format!("--author=^((?!{:}).*)$", exclude_author).into());
		}

		args.into_iter()
	}
}

// endregion CommitArgs

// region CommitStats

impl std::ops::Add for CommitStats {
	type Output = CommitStats;

	fn add(self, rhs: Self) -> Self::Output {
		CommitStats {
			files_changed: self.files_changed.saturating_add(rhs.files_changed),
			lines_added: self.lines_added.saturating_add(rhs.lines_added),
			lines_deleted: self.lines_deleted.saturating_add(rhs.lines_deleted),
		}
	}
}

impl std::ops::AddAssign for CommitStats {
	fn add_assign(&mut self, rhs: Self) {
		self.files_changed = self.files_changed.saturating_add(rhs.files_changed);
		self.lines_added = self.lines_added.saturating_add(rhs.lines_added);
		self.lines_deleted = self.lines_deleted.saturating_add(rhs.lines_deleted);
	}
}

impl Display for CommitStats {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"files changed: {}, lines added: {}, lines deleted: {}",
			self.files_changed, self.lines_added, self.lines_deleted
		)
	}
}

// endregion CommitStats

// region GlobalStat

impl Display for GlobalStat {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"author: {}, total commits: {}, {}",
			self.author, self.commits_count, self.stats
		)
	}
}

// endregion GlobalStat

// region SimpleStat

impl SimpleStat {
	pub fn new() -> Self {
		SimpleStat::default()
	}
}

impl Display for SimpleStat {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "total commits: {}, {}", self.commits_count, self.stats)
	}
}

impl std::ops::Add for SimpleStat {
	type Output = SimpleStat;

	fn add(self, rhs: Self) -> Self::Output {
		SimpleStat {
			commits_count: self.commits_count.saturating_add(rhs.commits_count),
			stats: self.stats.add(rhs.stats),
		}
	}
}

impl std::ops::AddAssign for SimpleStat {
	fn add_assign(&mut self, rhs: Self) {
		self.commits_count = self.commits_count.saturating_add(rhs.commits_count);
		self.stats = self.stats + rhs.stats;
	}
}

impl From<CommitDetail<'_>> for SimpleStat {
	fn from(value: CommitDetail) -> Self {
		value.stats.into()
	}
}

impl From<CommitStats> for SimpleStat {
	fn from(value: CommitStats) -> Self {
		SimpleStat {
			commits_count: 1,
			stats: value,
		}
	}
}

// endregion SimpleStat

// region MinimalCommitDetail

impl<'a> From<CommitDetail<'a>> for MinimalCommitDetail<'a> {
	fn from(value: CommitDetail<'a>) -> Self {
		MinimalCommitDetail {
			hash: value.hash,
			author_timestamp: value.author_timestamp,
			stats: value.stats,
		}
	}
}

impl Display for MinimalCommitDetail<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {}", self.hash, self.stats)
	}
}

// endregion MinimalCommitDetail

// region SortStatsBy

impl Default for SortStatsBy {
	fn default() -> Self {
		SortStatsBy::Commits
	}
}

// endregion SortStatsBy

// region CommitDetail

impl CommitDetail<'_> {
	pub fn get_author_datetime(&self) -> DateTime<Utc> {
		let naive = NaiveDateTime::from_timestamp_opt(self.author_timestamp, 0).unwrap();
		DateTime::from_naive_utc_and_offset(naive, Utc)
	}
}

impl Display for CommitDetail<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}, author: {}, {}, {}",
			self.hash,
			self.author,
			self.get_author_datetime(),
			self.stats
		)
	}
}

// endregion CommitDetail

// region GlobalStatsExt

impl GlobalStatsExt for HashMap<Author, Vec<MinimalCommitDetail<'_>>> {
	fn global_stats(&self, sort_stats_by: SortStatsBy) -> Vec<GlobalStat> {
		let mut global_stats = self
			.iter()
			.map(|(key, value)| {
				let stats = value.iter().map(|item| item.stats).reduce(|acc, item| acc + item).unwrap();
				let total_commits = value.len();
				GlobalStat {
					author: Author::from(key),
					commits_count: total_commits,
					stats,
				}
			})
			.collect::<Vec<_>>();

		match sort_stats_by {
			SortStatsBy::Commits => global_stats.sort_by_key(|item| item.commits_count),
			SortStatsBy::FilesChanged => global_stats.sort_by_key(|item| item.stats.files_changed),
			SortStatsBy::LinesAdded => global_stats.sort_by_key(|item| item.stats.lines_added),
			SortStatsBy::LinesDeleted => global_stats.sort_by_key(|item| item.stats.lines_deleted),
		}

		global_stats.reverse();
		global_stats
	}
}

// endregion GlobalStatsExt

// region CommitStatsExt

impl<'a> CommitStatsExt for Vec<CommitDetail<'_>> {
	fn reduced_stats(&self) -> HashMap<Author, Vec<MinimalCommitDetail>> {
		let mut hashmap: HashMap<Author, Vec<MinimalCommitDetail>> = HashMap::new();

		let mut cloned = self.to_vec();

		while !cloned.is_empty() {
			let commit = cloned.remove(0);
			let author = commit.author.to_owned();
			let minimal_commit: MinimalCommitDetail = commit.into();
			let mut vec: Vec<MinimalCommitDetail> = Vec::new();
			let mut index = Some(0);

			while index.is_some() {
				index = cloned.iter().position(|c| {
					let ca = &c.author;
					author.eq(ca)
				});

				if let Some(index) = index {
					let commit2 = cloned.remove(index);
					vec.push(commit2.into());
				}
			}

			vec.insert(0, minimal_commit);
			hashmap.insert(author.to_owned(), vec);
		}
		hashmap
	}

	fn commits_per_month(mut self) -> HashMap<String, HashMap<Author, SimpleStat>> {
		let mut result: HashMap<String, HashMap<Author, SimpleStat>> = HashMap::new();
		if self.len() > 1 {
			let last = self.last().unwrap();
			let first = self.first().unwrap();
			let last_date = last.get_author_datetime();
			let mut first_date = first
				.get_author_datetime()
				.with_day(last_date.day())
				.unwrap()
				.with_hour(0)
				.unwrap()
				.with_minute(0)
				.unwrap()
				.with_second(0)
				.unwrap()
				.with_nanosecond(0)
				.unwrap();

			loop {
				let date_key = first_date.with_day0(0).unwrap().format("%Y-%m").to_string();
				let mut current_map: HashMap<Author, SimpleStat> = HashMap::new();

				if self.is_empty() {
					break;
				}

				loop {
					if self.is_empty() {
						break;
					}

					let commit = self.get(0).unwrap();
					let commit_datetime = commit.get_author_datetime();
					if commit_datetime.year() <= first_date.year() && commit_datetime.month() <= first_date.month() {
						let removed = self.remove(0);
						let author = removed.author.to_owned();
						if current_map.contains_key(&author) {
							*current_map.get_mut(&author).unwrap() += removed.into();
						} else {
							current_map.insert(author, removed.into());
						}
					} else {
						break;
					}
				}
				result.insert(date_key, current_map);

				first_date = first_date.checked_add_months(Months::new(1)).unwrap();
				if first_date > last_date {
					break;
				}
			}
		}
		result
	}

	fn commits_per_weekday(mut self) -> CommitsPerWeekday {
		let mut final_map: HashMap<Weekday, HashMap<Author, SimpleStat>> = HashMap::from([
			(Weekday::Mon, HashMap::new()),
			(Weekday::Tue, HashMap::new()),
			(Weekday::Wed, HashMap::new()),
			(Weekday::Thu, HashMap::new()),
			(Weekday::Fri, HashMap::new()),
			(Weekday::Sat, HashMap::new()),
			(Weekday::Sun, HashMap::new()),
		]);

		for commit in self.iter_mut() {
			let author = commit.author.to_owned();
			let datetime = commit.get_author_datetime();
			let weekday = datetime.weekday();
			if !final_map.get(&weekday).unwrap().contains_key(&author) {
				final_map.get_mut(&weekday).unwrap().insert(author.clone(), SimpleStat::new());
			}
			*final_map.get_mut(&weekday).unwrap().get_mut(&author).unwrap() += commit.to_owned().into();
		}
		CommitsPerWeekday(final_map)
	}

	fn commits_per_day_hour(self) -> CommitsPerDayHour {
		let mut final_map: HashMap<u32, HashMap<Author, SimpleStat>> = HashMap::new();
		for i in 0..24 {
			final_map.insert(i, HashMap::new());
		}

		for commit in self.into_iter() {
			let author = commit.author.to_owned();
			let datetime = commit.get_author_datetime();
			let hour = datetime.hour();
			if !final_map.get(&hour).unwrap().contains_key(&author) {
				final_map.get_mut(&hour).unwrap().insert(author, commit.into());
			} else {
				*final_map.get_mut(&hour).unwrap().get_mut(&author).unwrap() += commit.into();
			}
		}
		CommitsPerDayHour(final_map)
	}
}

// endregion CommitStatsExt

// region CommitsPerWeekday

impl CommitsPerWeekday {
	pub fn detailed_stats(&self) -> &HashMap<Weekday, HashMap<Author, SimpleStat>> {
		&self.0
	}

	pub fn global_stats(&self) -> HashMap<Weekday, SimpleStat> {
		let mut global_map: HashMap<Weekday, SimpleStat> = HashMap::new();
		for (key, value) in self.0.iter() {
			global_map.insert(key.clone(), SimpleStat::new());
			for (_, stats) in value.iter() {
				*global_map.get_mut(key).unwrap() += stats.clone();
			}
		}
		global_map
	}
}

// endregion CommitsPerWeekday

// region CommitsPerDayHour

impl CommitsPerDayHour {
	pub fn detailed_stats(&self) -> &HashMap<u32, HashMap<Author, SimpleStat>> {
		&self.0
	}

	pub fn global_stats(&self) -> HashMap<u32, SimpleStat> {
		let mut global_map: HashMap<u32, SimpleStat> = HashMap::new();
		for (key, value) in self.0.iter() {
			global_map.insert(key.clone(), SimpleStat::new());
			for (_, stats) in value.iter() {
				*global_map.get_mut(key).unwrap() += stats.clone();
			}
		}
		global_map
	}
}

// endregion CommitsPerDayHour
