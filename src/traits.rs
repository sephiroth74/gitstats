use std::collections::HashMap;

use crate::{
	Author, CommitsHeatMap, CommitsPerDayHour, CommitsPerMonth, CommitsPerWeekday, GlobalStat, MinimalCommitDetail, SortStatsBy,
};

pub trait CommitStatsExt {
	fn reduced_stats(&self) -> HashMap<Author, Vec<MinimalCommitDetail>>;
	fn commits_per_month(self) -> CommitsPerMonth;
	fn commits_per_weekday(self) -> CommitsPerWeekday;
	fn commits_per_day_hour(self) -> CommitsPerDayHour;

	/// Return a commit heatmap
	/// # Examples:
	/// ```rust
	///
	/// use chrono::Weekday;
	/// use comfy_table::Table;
	/// use gitstats::{CommitArgs, Repo};
	/// use num_traits::cast::FromPrimitive;
	///
	/// fn commits_heatmap() {
	///	    let repo = Repo::new("/custom/repo");
	///	    let commits = repo.list_commits(CommitArgs::default()).unwrap();
	///	    let stats = repo.commits_stats(&commits).unwrap();
	///	    let commits_heatmap = stats.commits_heatmap();
	///     let global_stats = commits_heatmap.global_stats();
	///
	///	    let mut table = Table::new();
	///	    table.set_header(vec![
	///		    "Weekday/Hour",
	///		    "0",
	///		    "1",
	///		    "2",
	///		    "3",
	///		    "4",
	///		    "5",
	///		    "6",
	///		    "7",
	///		    "8",
	///		    "9",
	///		    "10",
	///		    "11",
	///		    "12",
	///		    "13",
	///		    "14",
	///		    "15",
	///		    "16",
	///		    "17",
	///		    "18",
	///		    "19",
	///		    "20",
	///		    "21",
	///		    "22",
	///		    "23",
	///	    ]);
	///
	///	    let mut rows: Vec<Vec<String>> = Vec::new();
	///	    for weekday in 0..7 {
	///		    let mut row = vec![Weekday::from_u8(weekday).unwrap().to_string()];
	///		    for _hour in 0..24 {
	///			    row.push("0".to_string());
	///		    }
	///		    rows.push(row);
	///	    }
	///
	///	    for (weekday, hours) in global_stats.iter().enumerate() {
	///		    for (hour, stats) in hours.iter().enumerate() {
	///			    let row = rows.get_mut(weekday).unwrap();
	///			    let current_value = row.get((hour + 1) as usize).unwrap().parse::<usize>().unwrap();
	///			    let new_value = current_value + stats.commits_count;
	///			    *row.get_mut((hour + 1) as usize).unwrap() = new_value.to_string();
	///		    }
	///	    }
	///	    table.add_rows(rows);
	///	    println!("{table}");
	///}
	/// ```
	///
	/// It will print something like this:
	///
	/// ```
	///
	/// +--------------+---+---+---+---+---+---+---+---+----+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
	/// | Weekday/Hour | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8  | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 |
	/// +=============================================================================================================================+
	/// | Mon          | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 3 | 2  | 2 | 5  | 4  | 6  | 7  | 3  | 1  | 1  | 1  | 0  | 0  | 0  | 0  | 0  | 0  |
	/// |--------------+---+---+---+---+---+---+---+---+----+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----|
	/// | Tue          | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 5 | 6  | 2 | 5  | 0  | 9  | 9  | 9  | 6  | 1  | 0  | 0  | 0  | 0  | 0  | 0  | 0  |
	/// |--------------+---+---+---+---+---+---+---+---+----+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----|
	/// | Wed          | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 2 | 12 | 2 | 4  | 3  | 3  | 5  | 5  | 4  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  |
	/// |--------------+---+---+---+---+---+---+---+---+----+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----|
	/// | Thu          | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 3  | 7 | 5  | 5  | 1  | 3  | 1  | 7  | 3  | 1  | 0  | 0  | 0  | 0  | 0  | 0  |
	/// |--------------+---+---+---+---+---+---+---+---+----+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----|
	/// | Fri          | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 0  | 4 | 0  | 1  | 4  | 6  | 2  | 2  | 1  | 0  | 0  | 0  | 0  | 0  | 0  | 0  |
	/// |--------------+---+---+---+---+---+---+---+---+----+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----|
	/// | Sat          | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0  | 0 | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  |
	/// |--------------+---+---+---+---+---+---+---+---+----+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----|
	/// | Sun          | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0  | 0 | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  | 0  |
	/// +--------------+---+---+---+---+---+---+---+---+----+---+----+----+----+----+----+----+----+----+----+----+----+----+----+----+
	/// ```
	///
	fn commits_heatmap(self) -> CommitsHeatMap;
}

pub trait GlobalStatsExt {
	fn global_stats(&self, sort_stats_by: SortStatsBy) -> Vec<GlobalStat>;
}
