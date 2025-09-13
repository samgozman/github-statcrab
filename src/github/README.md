# GitHub Statistics Service

This module provides functionality to fetch real GitHub user statistics using the GitHub GraphQL API.

## Setup

You need to set up a GitHub Personal Access Token to use this service:

1. Go to GitHub Settings > Developer settings > Personal access tokens
2. Generate a new token with appropriate permissions
3. Set the `GITHUB_TOKEN` environment variable:

```bash
export GITHUB_TOKEN="your_github_token_here"
```

## Error Handling

The service handles various error conditions:

- `UserNotFound` - When a GitHub user doesn't exist
- `InvalidUsername` - When username format is invalid
- `RateLimitExceeded` - When GitHub API rate limits are hit
- `MissingToken` - When no GitHub token is configured
- `NetworkError` - When network requests fail
- `GraphQLError` - When GraphQL queries fail

## Rate Limiting

GitHub API has [rate limits](https://github.com/orgs/community/discussions/163553):

- 5000 requests/hour for authenticated requests
- 60 requests/hour for unauthenticated requests

The service implements pagination for repository data to handle users with many repositories efficiently.

## Testing

Most integration tests are disabled by default because they require a real GitHub token. To run with tests:

1. Set `GITHUB_TOKEN` environment variable
2. Uncomment the test cases in `src/web/routes.rs`
3. Run `cargo test`
