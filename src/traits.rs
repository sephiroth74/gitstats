use crate::{CommitsHeatMap, CommitsPerAuthor, CommitsPerDayHour, CommitsPerMonth, CommitsPerWeekday};

pub trait CommitStatsExt {
	/// Return the commits per author
	///
	/// # Examples:
	/// ```rust
	///
	/// use comfy_table::Table;
	/// use gitstats::{CommitArgs, Repo, SortStatsBy};
	/// fn contributors_stats() {
	/// 	let repo = Repo::new("/custom/repo");
	/// 	let commits = repo.list_commits(CommitArgs::default()).unwrap();
	/// 	let stats = repo.commits_stats(&commits).unwrap();
	/// 	let commits_per_author = stats.commits_per_author();
	/// 	let mut global_stats = commits_per_author.global_stats(SortStatsBy::LinesAdded);
	/// 	global_stats.sort_by(|a,b|b.commits_count.cmp(&a.commits_count));
	///
	/// 	let mut table = Table::new();
	/// 	table.set_header(["Author", "Commits", "Lines"]);
	///
	/// 	for global_stat in global_stats.iter() {
	/// 		let commits_count = global_stat.commits_count;
	/// 		let total_lines = global_stat.stats.lines_added;
	/// 		table.add_row([(&global_stat.author).name.to_string(), commits_count.to_string(), total_lines.to_string()]);
	/// 	}
	///
	/// 	println!("{table}");
	/// }
	///
	/// ```
	///
	/// It will print something like this:
	///
	/// ```
	///
	/// +---------------------+---------+--------+
	/// | Author              | Commits | Lines  |
	/// +========================================+
	/// | John Doe            | 54      | 13355  |
	/// |---------------------+---------+--------|
	/// | Jane Doe            | 48      | 1355   |
	/// |---------------------+---------+--------|
	/// | Alessandro Crugnola | 45      | 172240 |
	/// |---------------------+---------+--------|
	/// | Michael Binary      | 31      | 13845  |
	/// |---------------------+---------+--------|
	/// | David One           | 9       | 56     |
	/// +---------------------+---------+--------+
	/// ```
	fn commits_per_author(&self) -> CommitsPerAuthor;

	///
	/// # Examples:
	/// ```rust
	///
	/// use chrono::{Months, Utc};
	/// use itertools::Itertools;
	/// use textplots::{AxisBuilder, Chart, LabelBuilder, LabelFormat, LineStyle, Plot, Shape, TickDisplay, TickDisplayBuilder};
	/// use gitstats::{CommitArgs, Repo};
	///
	/// fn commits_per_month() {
	/// 	let repo = Repo::new("/custom/path");
	/// 	let commits = repo.list_commits(CommitArgs::default()).unwrap();
	/// 	let stats = repo.commits_stats(&commits).unwrap();
	/// 	let commits_per_months = stats.commits_per_month();
	/// 	let global_stats = commits_per_months.global_stats();
	///
	/// 	let mut points = Vec::new();
	/// 	let start = Utc::now().checked_sub_months(Months::new(6)).unwrap();
	/// 	for (index, value) in global_stats.iter().sorted_by_key(|(key, _)| key.to_string()).enumerate() {
	/// 		points.push((index as f32, value.1.commits_count as f32));
	/// 	}
	/// 	Chart::new_with_y_range(100, 50, 0.0, 5.0, 0.0, 50.0)
	/// 		.lineplot(&Shape::Bars(&points))
	/// 		.x_axis_style(LineStyle::Solid)
	/// 		.y_axis_style(LineStyle::Solid)
	/// 		.y_tick_display(TickDisplay::Dense)
	/// 		.x_label_format(LabelFormat::Custom(Box::new(move |val| {
	/// 			let new_start = start.checked_add_months(Months::new(val as u32)).unwrap();
	/// 			format!("{}", new_start.format("%Y-%m"))
	/// 		})))
	/// 		.display();
	/// }
	/// ```
	///
	/// It will print something like this:
	/// ```
	///
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀ 50.0
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡤⠤⠤⠤⠤⠤⠤⠤⠤⠤⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
	/// ⣇⣀⣀⣀⣀⣀⣀⣀⣀⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀ 41.7
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀ 33.3
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⣇⣀⣀⣀⣀⣀⣀⣀⣀⣀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀ 25.0
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡗⠒⠒⠒⠒⠒⠒⠒⠒⠒⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡗⠒⠒⠒⠒⠒⠒⠒⠒⠒⡆ 16.7
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇ 8.3
	/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
	/// ⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠁ 0.0
	/// 2023-07                                    2023-12
	/// ```
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
