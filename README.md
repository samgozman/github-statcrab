# github-statcrab

## Setup

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

## Running the Server

The server automatically loads environment variables from the `.env` file:

```bash
cargo run
```

The server will start on `http://0.0.0.0:3000` and will automatically read your GitHub token from the `.env` file.

### API Endpoints

Once the server is running, you can access the following endpoints:

- **Stats Card**: `GET /stats-card?username=<github_username>`
  - Example: `http://localhost:3000/stats-card?username=samgozman`
  - Optional parameters: `theme`, `hide`, `hide_title`, `hide_background`, etc.

- **Languages Card**: `GET /langs-card?username=<github_username>`  
  - Example: `http://localhost:3000/langs-card?username=samgozman`
  - Optional parameters: `theme`, `layout`, `max_languages`, etc.

Both endpoints return SVG images that can be embedded in README files or web pages.

## Testing

Run the tests with:

```bash
cargo test
```

or to run a specific test:

```bash
make test
```
