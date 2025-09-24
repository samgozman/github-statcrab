# github-statcrab

`github-statcrab` is a Rust-based web server that generates dynamic SVG cards displaying GitHub user statistics and programming language usage. It leverages the GitHub API to fetch user data and presents it in a visually appealing format to be embedded in README files or web pages.

**Simple to install & use, easy to customize with various themes and layouts!**

<details>
<summary>Table of contents</summary>

- [github-statcrab](#github-statcrab)
  - [GitHub Top Languages card](#github-top-languages-card)
    - [Horizontal layout](#horizontal-layout)
    - [Vertical layout](#vertical-layout)
    - [Options for `/api/langs-card`](#options-for-apilangs-card)
  - [GitHub user stats card](#github-user-stats-card)
    - [Options for `/api/stats-card`](#options-for-apistats-card)
      - [Available Statistics to Hide](#available-statistics-to-hide)
  - [Themes](#themes)
    - [Adding new themes](#adding-new-themes)
  - [Usage Guide](#usage-guide)
    - [For Developers](#for-developers)
      - [Running From Docker Latest Image](#running-from-docker-latest-image)
      - [Build \& Run Server Locally](#build--run-server-locally)
        - [API Endpoints](#api-endpoints)
        - [Testing](#testing)

</details>

## GitHub Top Languages card

Displays the top programming languages used by a GitHub user in a visually appealing card format. You can insert it into your GitHub README with a simple markdown snippet.

Just replace `<your-hosted-instacne>` with the URL of your hosted instance of the `github-statcrab` server and `samgozman` with your GitHub username.

### Horizontal layout

```markdown
[![GitHub Top Languages](https://<your-hosted-instacne>/api/langs-card?username=samgozman&layout=horizontal&max_languages=8&theme=dracula&size_weight=0.5&count_weight=0.5)](https://github.com/samgozman/github-statcrab)
```

[![GitHub Top Languages for samgozman](https://github-statcrab-ce.extr.app/api/langs-card?username=samgozman&layout=horizontal&max_languages=8&theme=dracula&size_weight=0.5&count_weight=0.5)](https://github.com/samgozman/github-statcrab)

### Vertical layout

```markdown
[![GitHub Top Languages](https://<your-hosted-instacne>/api/langs-card?username=samgozman&layout=vertical&max_languages=8&theme=dracula&size_weight=0.5&count_weight=0.5)](https://github.com/samgozman/github-statcrab)
```

[![GitHub Top Languages for samgozman](https://github-statcrab-ce.extr.app/api/langs-card?username=samgozman&layout=vertical&max_languages=8&theme=dracula&size_weight=0.5&count_weight=0.5)](https://github.com/samgozman/github-statcrab)

### Options for `/api/langs-card`

| Parameter | Description | Type | Required | Default | Example |
|-----------|-------------|------|----------|---------|---------|
| `username` | GitHub username | `string` | ✅ | - | `samgozman` |
| `layout` | Card layout orientation | `string` | ❌ | `vertical` | `horizontal`, `vertical` |
| `max_languages` | Maximum number of languages to display | `number` | ❌ | `8` | `5` |
| `size_weight` | Weight factor for repository size in ranking | `number` | ❌ | `0.5` | `0.3` |
| `count_weight` | Weight factor for file count in ranking | `number` | ❌ | `0.5` | `0.7` |
| `exclude_repo` | Comma-separated list of repositories to exclude | `string` | ❌ | - | `repo1,repo2,private-repo` |
| `theme` | Visual theme for the card | `string` | ❌ | `light` | `dark`, `dracula`, `transparent-blue`, `monokai` |
| `offset_x` | Horizontal offset for card positioning | `number` | ❌ | `12` | `20` |
| `offset_y` | Vertical offset for card positioning | `number` | ❌ | `12` | `15` |
| `hide_title` | Hide the card title | `boolean` | ❌ | `false` | `true` |
| `hide_background` | Hide the card background | `boolean` | ❌ | `false` | `true` |
| `hide_background_stroke` | Hide the card background border | `boolean` | ❌ | `false` | `true` |

## GitHub user stats card

Can be used to show GitHub user statistics such as total stars, forks, commits, pull requests, issues, and more. You can insert it into your GitHub README with a simple markdown snippet:

```markdown
[![GitHub Stats for samgozman](https://<your-hosted-instacne>/api/stats-card?username=samgozman&theme=monokai)](https://github.com/samgozman/github-statcrab)
```

You should replace `<your-hosted-instacne>` with the URL of your hosted instance of the `github-statcrab` server and `samgozman` with your GitHub username.

[![GitHub Stats for samgozman](https://github-statcrab-ce.extr.app/api/stats-card?username=samgozman&theme=monokai)](https://github.com/samgozman/github-statcrab)

### Options for `/api/stats-card`

| Parameter | Description | Type | Required | Default | Example |
|-----------|-------------|------|----------|---------|---------|
| `username` | GitHub username | `string` | ✅ | - | `samgozman` |
| `hide` | Comma-separated list of stats to hide | `string` | ❌ | - | `stars_count,commits_ytd_count` |
| `theme` | Visual theme for the card | `string` | ❌ | `light` | `dark`, `dracula`, `monokai`, `transparent-blue` |
| `offset_x` | Horizontal offset for card positioning | `number` | ❌ | `12` | `20` |
| `offset_y` | Vertical offset for card positioning | `number` | ❌ | `12` | `15` |
| `hide_title` | Hide the card title | `boolean` | ❌ | `false` | `true` |
| `hide_background` | Hide the card background | `boolean` | ❌ | `false` | `true` |
| `hide_background_stroke` | Hide the card background border | `boolean` | ❌ | `false` | `true` |

#### Available Statistics to Hide

The `hide` parameter accepts a comma-separated list of the following values:

- `stars_count` - Total stars received across all repositories
- `commits_ytd_count` - Total commits made this year
- `issues_count` - Total issues opened
- `pull_requests_count` - Total pull requests created
- `merge_requests_count` - Total merge requests created
- `reviews_count` - Total pull request reviews performed
- `started_discussions_count` - Total discussions started
- `answered_discussions_count` - Total discussions answered

**Note:** At least 2 statistics must remain visible on the card.

## Themes

The `github-statcrab` server supports multiple visual themes for the generated SVG cards. You can customize the appearance of the cards by using the `theme` parameter in the API requests.

**All available themes and their previews can be found in the [themes readme file](https://github.com/samgozman/github-statcrab/blob/main/assets/css/themes/README.md)**.

### Adding new themes

Adding new themes to the `github-statcrab` is pretty easy. You don't even need to know Rust! You can just open a PR with a new CSS file in the `assets/css/themes/` directory. Make sure to follow the existing theme structure and naming conventions. It's the easiest way to contribute!

Github Actions will automatically do the rest for you, including building the [themes readme file](https://github.com/samgozman/github-statcrab/blob/main/assets/css/themes/README.md) page with previews of all themes (yes, it's automated!).

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
   ```

3. Get a GitHub Personal Access Token:
   - Go to GitHub Settings > Developer settings > Personal access tokens
   - Generate a new token with `public_repo` and `read:user` scopes

4. *(optional)* add a test GitHub username for local e2e testing:

   ```env
   TEST_GITHUB_USERNAME=your_github_username
   ```

5. *(optional)* Set up Sentry for error tracking:
   You can also set optional Sentry configuration for error tracking.

   ```env
   SENTRY_DSN=your_sentry_dsn_here
   SENTRY_ENVIRONMENT=development
   ```

6. *(optional)* Configure cache sizes:
   You can adjust the cache sizes for user stats and language stats in the `.env` file.

   ```env
   # Maximum memory capacity for GitHub API response cache in MiB (default: 32)
   CACHE_MAX_CAPACITY_MB=32
   # TTL for user stats cache in seconds (default: 900 = 15 minutes)
   CACHE_USER_STATS_TTL_SECONDS=900
   # TTL for user languages cache in seconds (default: 3600 = 1 hour)  
   CACHE_USER_LANGUAGES_TTL_SECONDS=3600
   ```

7. *(optional)* Restrict API access to specific users:
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
