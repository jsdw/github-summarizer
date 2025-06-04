# github-summarizer

A small binary which connects to your github account and outputs your activity (issues opened, PRs created and repositories created) since the given `--from` date.

Example usage:

```
cargo run -- --from 2024-12-01T00:00Z --gh-token $(cat ~/.gh_token) > ~/Documents/james-2025h1-github-summary.txt
```

Where `$(cat ~/.gh_token)` in this example points to a GITHUB token that was saved to the file `~/.gh_token`.

May be useful as a reminder of what you've been up to, or just pass the file/text to your LLM of choice and have it turn it into a nice summary for you.