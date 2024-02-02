# gitstats
generate some stats for git repositories

# Examples:

Fetch the commits per author

```rust

use comfy_table::Table;
use gitstats::{CommitArgs, Repo, SortStatsBy};

fn contributors_stats() {
    let repo = Repo::new("/custom/repo");
    let commits = repo.list_commits(CommitArgs::default()).unwrap();
    let stats = repo.commits_stats(&commits).unwrap();
    let commits_per_author = stats.commits_per_author();
    let mut global_stats = commits_per_author.global_stats(SortStatsBy::LinesAdded);
    global_stats.sort_by(|a,b|b.commits_count.cmp(&a.commits_count));

    let mut table = Table::new();
    table.set_header(["Author", "Commits", "Lines"]);

    for global_stat in global_stats.iter() {
        let commits_count = global_stat.commits_count;
        let total_lines = global_stat.stats.lines_added;
        table.add_row([(&global_stat.author).name.to_string(), commits_count.to_string(), total_lines.to_string()]);
    }

    println!("{}", table);
}

 ```

It will print something like this:

```

 +---------------------+---------+--------+
 | Author              | Commits | Lines  |
 +========================================+
 | John Doe            | 54      | 13355  |
 |---------------------+---------+--------|
 | Jane Doe            | 48      | 1355   |
 |---------------------+---------+--------|
 | Alessandro Crugnola | 45      | 172240 |
 |---------------------+---------+--------|
 | Michael Binary      | 31      | 13845  |
 |---------------------+---------+--------|
 | David One           | 9       | 56     |
 +---------------------+---------+--------+
 ```