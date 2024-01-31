use crate::{Author, CommitsPerDayHour, CommitsPerMonth, CommitsPerWeekday, GlobalStat, MinimalCommitDetail, SortStatsBy};
use std::collections::HashMap;

pub trait CommitStatsExt {
	fn reduced_stats(&self) -> HashMap<Author, Vec<MinimalCommitDetail>>;
	fn commits_per_month(self) -> CommitsPerMonth;
	fn commits_per_weekday(self) -> CommitsPerWeekday;
	fn commits_per_day_hour(self) -> CommitsPerDayHour;
}

pub trait GlobalStatsExt {
	fn global_stats(&self, sort_stats_by: SortStatsBy) -> Vec<GlobalStat>;
}
