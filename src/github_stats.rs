use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

use crate::cards::langs_card::LanguageStat;

/// GitHub API response for user statistics
#[derive(Deserialize)]
struct GitHubUser {
    public_repos: u32,
    followers: u32,
    following: u32,
}

/// GitHub API response for repository information
#[derive(Deserialize)]
struct GitHubRepo {
    name: String,
    stargazers_count: u32,
    open_issues_count: u32,
    language: Option<String>,
    size: u32, // size in KB
}

/// GitHub API response for repository languages
#[derive(Deserialize)]
struct GitHubLanguages(HashMap<String, u64>);

/// GitHub statistics for the stats card
#[derive(Debug)]
pub struct GitHubStats {
    pub stars_count: Option<u32>,
    pub commits_ytd_count: Option<u32>,
    pub issues_count: Option<u32>,
    pub pull_requests_count: Option<u32>,
    pub merge_requests_count: Option<u32>,
    pub reviews_count: Option<u32>,
    pub started_discussions_count: Option<u32>,
    pub answered_discussions_count: Option<u32>,
}

impl Default for GitHubStats {
    fn default() -> Self {
        Self {
            stars_count: None,
            commits_ytd_count: None,
            issues_count: None,
            pull_requests_count: None,
            merge_requests_count: None,
            reviews_count: None,
            started_discussions_count: None,
            answered_discussions_count: None,
        }
    }
}

/// Fetches GitHub statistics for a given username
pub async fn fetch_github_stats(username: &str) -> Result<GitHubStats> {
    let client = reqwest::Client::new();
    let mut stats = GitHubStats::default();

    // Fetch user's repositories to calculate total stars
    let repos_url = format!("https://api.github.com/users/{}/repos?per_page=100", username);
    let repos: Vec<GitHubRepo> = client
        .get(&repos_url)
        .header("User-Agent", "github-statcrab")
        .send()
        .await
        .context("Failed to fetch repositories")?
        .json()
        .await
        .context("Failed to parse repositories JSON")?;

    // Calculate total stars and issues
    let total_stars: u32 = repos.iter().map(|repo| repo.stargazers_count).sum();
    let total_issues: u32 = repos.iter().map(|repo| repo.open_issues_count).sum();

    stats.stars_count = Some(total_stars);
    stats.issues_count = Some(total_issues);

    // For now, return placeholder values for other stats since they require
    // more complex GraphQL queries or specific API endpoints
    stats.commits_ytd_count = Some(123); // Placeholder
    stats.pull_requests_count = Some(42); // Placeholder  
    stats.merge_requests_count = Some(10); // Placeholder
    stats.reviews_count = Some(25); // Placeholder
    stats.started_discussions_count = Some(5); // Placeholder
    stats.answered_discussions_count = Some(15); // Placeholder

    Ok(stats)
}

/// Fetches GitHub language statistics for a given username
pub async fn fetch_github_language_stats(username: &str) -> Result<Vec<LanguageStat>> {
    let client = reqwest::Client::new();

    // Fetch user's repositories
    let repos_url = format!("https://api.github.com/users/{}/repos?per_page=100", username);
    let repos: Vec<GitHubRepo> = client
        .get(&repos_url)
        .header("User-Agent", "github-statcrab")
        .send()
        .await
        .context("Failed to fetch repositories")?
        .json()
        .await
        .context("Failed to parse repositories JSON")?;

    let mut language_stats: HashMap<String, LanguageStat> = HashMap::new();

    // Process each repository to collect language statistics
    for repo in repos {
        if let Some(primary_language) = repo.language {
            // Fetch detailed language breakdown for the repository
            let languages_url = format!("https://api.github.com/repos/{}/{}/languages", username, repo.name);
            
            if let Ok(response) = client
                .get(&languages_url)
                .header("User-Agent", "github-statcrab")
                .send()
                .await
            {
                if let Ok(languages) = response.json::<GitHubLanguages>().await {
                    for (lang_name, size_bytes) in languages.0 {
                        let entry = language_stats
                            .entry(lang_name.clone())
                            .or_insert_with(|| LanguageStat {
                                name: lang_name,
                                size_bytes: 0,
                                repo_count: 0,
                            });
                        
                        entry.size_bytes += size_bytes as usize;
                        entry.repo_count += 1;
                    }
                }
            } else {
                // Fallback: use primary language with repository size
                let entry = language_stats
                    .entry(primary_language.clone())
                    .or_insert_with(|| LanguageStat {
                        name: primary_language.clone(),
                        size_bytes: 0,
                        repo_count: 0,
                    });
                
                entry.size_bytes += (repo.size * 1024) as usize; // Convert KB to bytes
                entry.repo_count += 1;
            }
        }
    }

    let mut stats: Vec<LanguageStat> = language_stats.into_values().collect();
    
    // Sort by size descending
    stats.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_stats_default() {
        let stats = GitHubStats::default();
        assert_eq!(stats.stars_count, None);
        assert_eq!(stats.commits_ytd_count, None);
        assert_eq!(stats.issues_count, None);
    }
}