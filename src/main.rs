mod cards;

use cards::stats_card::StatsCard;

fn main() {
    let card = StatsCard {
        username: "Sam Gozman".to_string(),
        stars_count: Some(123),
        commits_ytd_count: Some(123),
        issues_count: Some(123),
        pull_requests_count: Some(123),
        merge_requests_count: Some(123),
        reviews_count: Some(123),
        started_discussions_count: Some(123),
        answered_discussions_count: Some(123),
    }
    .render();

    println!("{}", card);
}
