# github-statcrab

`github-statcrab` is a Rust-based web server that generates dynamic SVG cards displaying GitHub user statistics and programming language usage. It leverages the GitHub API to fetch user data and presents it in a visually appealing format to be embedded in README files or web pages.

## Usage Guide

This guide will help you set up and run the `github-statcrab` server, as well as how to use its API endpoints.

### For Developers

1. Copy the example environment file:

   ```bash
   cp .env.example .env
   ```

2. Edit `.env` and set your GitHub token:

   ```env
   GITHUB_TOKEN=your_github_personal_access_token_here
   TEST_GITHUB_USERNAME=your_github_username
   ```

3. Get a GitHub Personal Access Token:
   - Go to GitHub Settings > Developer settings > Personal access tokens
   - Generate a new token with `public_repo` and `read:user` scopes

4. *(optional)* Set up Sentry for error tracking:
   You can also set optional Sentry configuration for error tracking.

   ```env
   SENTRY_DSN=your_sentry_dsn_here
   SENTRY_ENVIRONMENT=development
   ```

5. *(optional)* Configure cache sizes:
   You can adjust the cache sizes for user stats and language stats in the `.env` file.

   ```env
   # Maximum memory capacity for GitHub API response cache in MiB (default: 32)
   CACHE_MAX_CAPACITY_MB=32
   # TTL for user stats cache in seconds (default: 900 = 15 minutes)
   CACHE_USER_STATS_TTL_SECONDS=900
   # TTL for user languages cache in seconds (default: 3600 = 1 hour)  
   CACHE_USER_LANGUAGES_TTL_SECONDS=3600
   ```

6. *(optional)* Restrict API access to specific users:
   You can limit which GitHub usernames are allowed to use the API by setting an allowlist in the `.env` file.

   ```env
   # Comma-separated list of GitHub usernames allowed to use the API
   # Leave empty or unset to allow all users (default: empty)
   ALLOWED_USERNAMES=user1,user2,user3
   ```

#### Running From Docker Latest Image

You can run the server using Docker. Make sure to replace `your_github_personal_access_token_here` with your actual GitHub Personal Access Token.

```bash
docker pull ghcr.io/samgozman/github-statcrab/server:latest  
docker run -p 3000:3000 --env-file .env ghcr.io/samgozman/github-statcrab/server:latest
```

#### Build & Run Server Locally

To run the server locally, ensure you have Rust and Cargo installed. You can install them from [rustup.rs](https://rustup.rs/).

The server automatically loads environment variables from the `.env` file:

```bash
make run
```

The server will start on `http://0.0.0.0:3000` and will automatically read your GitHub token from the `.env` file.

##### API Endpoints

Once the server is running, you can access the following endpoints:

- **Stats Card**: `GET /stats-card?username=<github_username>`
  - Example: `http://localhost:3000/stats-card?username=samgozman`
  - Optional parameters: `theme`, `hide`, `hide_title`, `hide_background`, etc.

- **Languages Card**: `GET /langs-card?username=<github_username>`  
  - Example: `http://localhost:3000/langs-card?username=samgozman`
  - Optional parameters: `theme`, `layout`, `max_languages`, etc.

Both endpoints return SVG images that can be embedded in README files or web pages.

##### Testing

Run the tests with:

```bash
make test
```
