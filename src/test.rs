#[cfg(test)]
mod test {
	use std::env::current_dir;
	use std::ops::Deref;
	use std::time::{Duration, Instant};

	use chrono::{DateTime, Months, Utc, Weekday};
	use itertools::Itertools;
	use lazy_static::lazy_static;
	use num_traits::cast::FromPrimitive;

	use crate::traits::{CommitStatsExt, GlobalStatsExt};
	use crate::{Author, CommitArgs, CommitHash, Repo, SortStatsBy};

	lazy_static! {
		static ref SINCE: DateTime<Utc> = Utc::now().checked_sub_months(Months::new(3)).unwrap();
		static ref UNTIL: DateTime<Utc> = Utc::now();
		static ref COMMIT_ARGS: CommitArgs = CommitArgs::builder()
			.since(SINCE.timestamp())
			.until(UNTIL.timestamp())
			.target_branch("develop")
			.exclude_merges(true)
			.exclude_author("blue TV Build".into())
			.build()
			.unwrap();
	}

	fn init_log() {
		let subscriber = tracing_subscriber::fmt()
			.compact()
			.with_file(false)
			.with_line_number(false)
			.with_max_level(tracing::Level::TRACE)
			.with_thread_ids(false)
			.with_thread_names(false)
			.finish();
		tracing::subscriber::set_global_default(subscriber).unwrap();
	}

	fn checkout_repo() -> Repo {
		let repo_dir = std::env::var("TEST_REPO_DIR").expect("Environment variable `TEST_REPO_DIR` is not defined");
		let repo = Repo::from(&repo_dir);
		repo.fetch_all().unwrap();
		repo
	}

	#[test]
	fn test_new_repo() {
		init_log();
		let current_dir = current_dir().unwrap();
		let path = current_dir;
		println!("path: {:?}", path);
		let repo = Repo::try_from(&path).unwrap();
		println!("repo: {}", repo);

		assert_eq!(path.to_str(), repo.to_str());
	}

	#[test]
	fn test_fetch() {
		init_log();
		let mut ticker = Ticker::new();
		checkout_repo();
		println!("fetched repo in {:?}", ticker.tick().0);
	}

	#[test]
	fn test_list_commits() {
		init_log();
		let mut ticker = Ticker::new();
		let repo = checkout_repo();
		ticker.tick();
		let commits = repo.list_commits(COMMIT_ARGS.clone()).unwrap();
		println!("listed commits in {:?}", ticker.tick().0);
		println!("total commits: {}", commits.len());
		assert!(commits.len() > 0);

		for commit in &commits {
			println!("commits: {}", commit);
		}
	}

	#[test]
	fn test_reduced_stats_per_author() {
		init_log();
		let repo = checkout_repo();
		let commits = repo.list_commits(COMMIT_ARGS.clone()).unwrap();
		println!("total commits: {}", commits.len());
		assert!(commits.len() > 0);

		let stats = repo.commits_stats(&commits).unwrap();
		assert_eq!(commits.len(), stats.len());

		let mut ticker = Ticker::new();
		let reduced = stats.reduced_stats();
		println!("generated reduced stats in {:?}", ticker.tick().0);
		println!("-----------------------------------------------");

		for (author, entry) in reduced.iter() {
			println!("Author: {}", author);
			println!("\ttotal commits: {}", entry.len());
			let mut k = 0;
			for stat in entry.iter() {
				println!("\t[{k}] {stat}");
				k += 1;
			}
			println!("-----------------------------------------------");
		}
	}

	#[test]
	fn test_contributors_stats() {
		init_log();
		let mut ticker = Ticker::new();
		let repo = checkout_repo();

		let commits = repo.list_commits(COMMIT_ARGS.clone()).unwrap();
		println!("total commits: {}", commits.len());
		println!("-----------------------------------------------");
		assert!(commits.len() > 0);

		let stats = repo.commits_stats(&commits).unwrap();
		assert_eq!(commits.len(), stats.len());

		let reduced = stats.reduced_stats();

		ticker.tick();
		let global_stats = reduced.global_stats(SortStatsBy::LinesAdded);
		println!("generated contributor's stats in {:?}", ticker.tick().0);
		println!("-----------------------------------------------");

		println!("Contributors Stats:");
		for global_stat in global_stats.iter() {
			println!("{}", global_stat);
		}

		println!("-----------------------------------------------");
	}

	#[test]
	fn test_show() {
		init_log();
		let repo = checkout_repo();
		let commit_hash = CommitHash::try_from("a9ae91ebf675cc57fb93cbcb6e179f89f0199e8e").unwrap();
		let stats = repo.commit_stats(&commit_hash).unwrap();
		println!("stats: {}", stats);
	}

	#[test]
	fn test_commits_per_month() {
		init_log();
		let mut ticker = Ticker::new();
		let repo = checkout_repo();

		let commits = repo.list_commits(COMMIT_ARGS.clone()).unwrap();
		println!("total commits: {}", commits.len());
		println!("---------------------------------------------");

		let stats = repo.commits_stats(&commits).unwrap();
		assert_eq!(commits.len(), stats.len());

		ticker.tick();
		let cloned_stats = stats.clone();
		let commits_per_months = cloned_stats.commits_per_month();
		println!("generated commits per month in {:?}", ticker.tick().0);
		println!("---------------------------------------------");

		let mut total_commits = 0;

		for (key, value) in commits_per_months
			.detailed_stats()
			.iter()
			.sorted_by_key(|(key, _)| key.to_string())
		{
			println!("date: {key}");

			for (author, stats) in value.iter() {
				println!("    {author} = {stats}");
				total_commits += stats.commits_count;
			}

			println!("--------------------------------------------");
		}

		assert_eq!(commits.len(), total_commits);

		let global_stats = commits_per_months.global_stats();
		println!("global stats:");
		println!("--------------------------------------------");
		for (key, value) in global_stats.iter().sorted_by_key(|(key, _)| key.to_string()) {
			println!("date: {key}, {}", value);
		}
	}

	#[test]
	fn test_commits_per_weekday() {
		init_log();
		let mut ticker = Ticker::new();
		let repo = checkout_repo();
		println!("checked out repo in {:?}", ticker.tick().0);
		println!("---------------------------------------------");

		let commits = repo.list_commits(COMMIT_ARGS.deref().clone()).unwrap();
		println!("listed commits from repo in {:?}", ticker.tick().0);
		println!("total commits: {}", commits.len());
		println!("---------------------------------------------");

		let stats = repo.commits_stats(&commits).unwrap();
		println!("parsed commits in {:?}", ticker.tick().0);
		println!("total stats: {}", stats.len());
		assert_eq!(commits.len(), stats.len());
		println!("---------------------------------------------");

		let commits_per_weekday = stats.commits_per_weekday();

		println!("commits per weekday created in {:?}", ticker.tick().0);
		println!("---------------------------------------------");

		let mut total_commits = 0;

		for (key, value) in commits_per_weekday.detailed_stats().iter().sorted_by_key(|a| a.0) {
			let weekday = Weekday::from_u8(*key).unwrap();
			println!("WeekDay: {weekday:?}");
			for (author, stats) in value.iter() {
				println!("{author} : {stats}");
				total_commits += stats.commits_count;
			}
			println!("---------------------------------------------");
		}

		println!("total commits: {total_commits}");
		assert_eq!(commits.len(), total_commits);

		let global_stats = commits_per_weekday.global_stats();

		println!("global commits per weekday created in {:?}", ticker.tick().0);
		println!("---------------------------------------------");

		total_commits = 0;

		for (key, value) in global_stats.iter().sorted_by_key(|a| a.0) {
			println!("WeekDay: {:?}, stats: {value}", Weekday::from_u8(*key).unwrap());
			total_commits += value.commits_count;
		}
		println!("total commits: {total_commits}");
		assert_eq!(commits.len(), total_commits);
		println!("---------------------------------------------");
		println!("done. {:?}", ticker.tick().1);
	}

	#[test]
	fn test_commits_per_day_hour() {
		init_log();
		let mut ticker = Ticker::new();
		let repo = checkout_repo();
		println!("checked out repo in {:?}", ticker.tick().0);
		println!("---------------------------------------------");

		let commits = repo.list_commits(COMMIT_ARGS.deref().clone()).unwrap();
		let stats = repo.commits_stats(&commits).unwrap();
		let commits_per_day_hour = stats.commits_per_day_hour();

		println!("commits per day hour created in {:?}", ticker.tick().0);
		println!("---------------------------------------------");

		let mut total_commits = 0;

		for (key, value) in commits_per_day_hour.detailed_stats().iter().sorted_by_key(|(key, _)| *key) {
			println!("Hour: {key}");
			for (author, stats) in value.iter() {
				println!("{author} : {stats}");
				total_commits += stats.commits_count;
			}
			println!("---------------------------------------------");
		}

		println!("total commits: {total_commits}");
		assert_eq!(commits.len(), total_commits);

		let global_stats = commits_per_day_hour.global_stats();

		println!("global commits per day hour created in {:?}", ticker.tick().0);
		println!("---------------------------------------------");

		total_commits = 0;

		for (key, value) in global_stats.iter().sorted_by_key(|(key, _)| *key) {
			println!("Hour: {key}, stats: {value}");
			total_commits += value.commits_count;
		}
		println!("total commits: {total_commits}");
		assert_eq!(commits.len(), total_commits);
		println!("---------------------------------------------");
		println!("done. {:?}", ticker.tick().1);
	}

	#[test]
	fn test_string_to_author() {
		init_log();
		let mut author: Author = "Alessandro Crugnola <alessandro@gmail.com>".try_into().unwrap();
		println!("Author: {}", author);

		author = "Alessandro Crugnola <sephiroth> <alessandro@gmail.com>".try_into().unwrap();
		println!("Author: {}", author);

		author = "Alessandro <alessandro.crugnola_123+1@gmail.com>".try_into().unwrap();
		println!("Author: {}", author);

		author = "Alessandro Crugnola <>".try_into().unwrap();
		println!("Author: {}", author);
	}

	#[derive(Debug)]
	struct Ticker {
		start: Instant,
		current: Instant,
	}

	impl Ticker {
		pub fn new() -> Self {
			Ticker {
				start: Instant::now(),
				current: Instant::now(),
			}
		}

		pub fn tick(&mut self) -> (Duration, Duration) {
			let elapsed = self.current.elapsed();
			let total = self.start.elapsed();
			self.current = Instant::now();
			(elapsed, total)
		}
	}
}
