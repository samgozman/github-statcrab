use moka::future::Cache;
use std::{env, sync::OnceLock, time::Duration};

use crate::cards::langs_card::LanguageStat;
use crate::github::types::GitHubStats;

/// Cache configuration settings
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum cache capacity in MB
    pub max_capacity_mb: u64,
    /// TTL for user stats cache
    pub user_stats_ttl: Duration,
    /// TTL for user languages cache
    pub user_languages_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity_mb: 32,
            user_stats_ttl: Duration::from_secs(900), // 15 minutes
            user_languages_ttl: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl CacheConfig {
    /// Load cache configuration from environment variables
    pub fn from_env() -> Self {
        let max_capacity_mb = env::var("CACHE_MAX_CAPACITY_MB")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(32);

        let user_stats_ttl = env::var("CACHE_USER_STATS_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(900));

        let user_languages_ttl = env::var("CACHE_USER_LANGUAGES_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(3600));

        Self {
            max_capacity_mb,
            user_stats_ttl,
            user_languages_ttl,
        }
    }
}

/// Cache key for GitHub API responses
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CacheKey {
    UserLanguages {
        username: String,
        excluded_repos_hash: u64,
    },
}

impl CacheKey {
    /// Create a cache key for user languages with excluded repositories
    pub fn user_languages(username: String, excluded_repos: &[String]) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        excluded_repos.hash(&mut hasher);
        let excluded_repos_hash = hasher.finish();

        Self::UserLanguages {
            username,
            excluded_repos_hash,
        }
    }
}

/// GitHub API response cache manager
pub struct GitHubCache {
    stats_cache: Cache<String, GitHubStats>,
    languages_cache: Cache<CacheKey, Vec<LanguageStat>>,
}

impl GitHubCache {
    /// Create a new cache instance with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        let stats_cache = Cache::builder()
            .weigher(|_key: &String, value: &GitHubStats| {
                // Rough estimation based on struct size and string contents
                let base_size = std::mem::size_of::<GitHubStats>();
                let name_size = value.name.as_ref().map(|n| n.len()).unwrap_or(0);
                let login_size = value.login.len();
                (base_size + name_size + login_size)
                    .try_into()
                    .unwrap_or(u32::MAX)
            })
            .max_capacity(config.max_capacity_mb * 1024 * 1024)
            .time_to_live(config.user_stats_ttl)
            .build();

        let languages_cache = Cache::builder()
            .weigher(|_key: &CacheKey, value: &Vec<LanguageStat>| {
                // Rough estimation for Vec<LanguageStat>
                let base_size = std::mem::size_of::<Vec<LanguageStat>>();
                let contents_size = value
                    .iter()
                    .map(|lang| std::mem::size_of::<LanguageStat>() + lang.name.len())
                    .sum::<usize>();
                (base_size + contents_size).try_into().unwrap_or(u32::MAX)
            })
            .max_capacity(config.max_capacity_mb * 1024 * 1024)
            .time_to_live(config.user_languages_ttl)
            .build();

        Self {
            stats_cache,
            languages_cache,
        }
    }

    /// Get or insert user stats with the configured TTL
    pub async fn get_or_insert_user_stats<F, Fut>(
        &self,
        username: String,
        fetch_fn: F,
    ) -> Result<GitHubStats, crate::github::types::GitHubApiError>
    where
        F: FnOnce() -> Fut,
        Fut:
            std::future::Future<Output = Result<GitHubStats, crate::github::types::GitHubApiError>>,
    {
        if let Some(stats) = self.stats_cache.get(&username).await {
            tracing::debug!("Cache hit for user stats: {}", username);
            return Ok(stats);
        }

        tracing::debug!("Cache miss for user stats: {}, fetching...", username);
        let stats = fetch_fn().await?;

        // Insert into cache (TTL is handled by the cache configuration)
        self.stats_cache.insert(username, stats.clone()).await;

        Ok(stats)
    }

    /// Get or insert user languages with the configured TTL
    pub async fn get_or_insert_user_languages<F, Fut>(
        &self,
        username: String,
        excluded_repos: &[String],
        fetch_fn: F,
    ) -> Result<Vec<LanguageStat>, crate::github::types::GitHubApiError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<
                Output = Result<Vec<LanguageStat>, crate::github::types::GitHubApiError>,
            >,
    {
        let key = CacheKey::user_languages(username.clone(), excluded_repos);

        if let Some(languages) = self.languages_cache.get(&key).await {
            tracing::debug!("Cache hit for user languages: {}", username);
            return Ok(languages);
        }

        tracing::debug!("Cache miss for user languages: {}, fetching...", username);
        let languages = fetch_fn().await?;

        // Insert into cache (TTL is handled by the cache configuration)
        self.languages_cache.insert(key, languages.clone()).await;

        Ok(languages)
    }

    /// Get current cache statistics for monitoring
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.stats_cache.entry_count() + self.languages_cache.entry_count(),
            weighted_size: self.stats_cache.weighted_size() + self.languages_cache.weighted_size(),
            stats_cache_entries: self.stats_cache.entry_count(),
            stats_cache_size: self.stats_cache.weighted_size(),
            languages_cache_entries: self.languages_cache.entry_count(),
            languages_cache_size: self.languages_cache.weighted_size(),
        }
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries across all caches
    pub entry_count: u64,
    /// Total weighted size across all caches in bytes
    pub weighted_size: u64,
    /// Number of entries in stats cache
    pub stats_cache_entries: u64,
    /// Weighted size of stats cache in bytes
    pub stats_cache_size: u64,
    /// Number of entries in languages cache
    pub languages_cache_entries: u64,
    /// Weighted size of languages cache in bytes
    pub languages_cache_size: u64,
}

// Global cache instance
static GITHUB_CACHE: OnceLock<GitHubCache> = OnceLock::new();

/// Get or initialize the global GitHub cache instance
pub fn get_github_cache() -> &'static GitHubCache {
    GITHUB_CACHE.get_or_init(|| {
        let config = CacheConfig::from_env();
        tracing::info!(
            "Initializing GitHub cache with capacity: {}MB, stats TTL: {}s, languages TTL: {}s",
            config.max_capacity_mb,
            config.user_stats_ttl.as_secs(),
            config.user_languages_ttl.as_secs()
        );
        GitHubCache::new(config)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_capacity_mb, 32);
        assert_eq!(config.user_stats_ttl, Duration::from_secs(900));
        assert_eq!(config.user_languages_ttl, Duration::from_secs(3600));
    }

    #[test]
    fn test_cache_key_user_languages() {
        let key1 = CacheKey::user_languages(
            "user1".to_string(),
            &["repo1".to_string(), "repo2".to_string()],
        );
        let key2 = CacheKey::user_languages(
            "user1".to_string(),
            &["repo1".to_string(), "repo2".to_string()],
        );
        let key3 = CacheKey::user_languages("user1".to_string(), &["repo1".to_string()]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
