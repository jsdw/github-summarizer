mod api;
mod utils;

use api::client::Api;
use clap::Parser;
use utils::DateTime;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// An ISO DateTime, for example '2025-06-01T00:00:00Z'.
    #[arg(long)]
    from: DateTime,

    /// A GitHub token to enable access to the APIs. This can also
    /// be provided via the env var GITHUB_TOKEN.
    ///
    /// The token should have read access to discussions, issues,
    /// metadata, and pull requests.
    #[arg(long)]
    gh_token: Option<String>,

    /// An optional user name. If not provided, the username associated
    /// with the provided GitHub token will be used.
    #[arg(long)]
    user: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Cli::parse();

    let from = opts.from;
    let to = DateTime::now();

    let gh_token = opts
        .gh_token
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "GITHUB_TOKEN must be set either via --gh-token or GITHUB_TOKEN env var"
            )
        })?;

    // Spin up an API client to talk to github.
    let api = Api::new(gh_token, opts.user).await?;

    // Get the data.
    let prs = api::pull_requests::query(&api, from, to).await?;
    let issues = api::issues::query(&api, from, to).await?;
    let repositories = api::repositories::query(&api, from, to).await?;

    // Some summary figures.
    let prs_count = prs.len();
    let merged_prs_count = prs
        .iter()
        .filter(|pr| pr.state == utils::ItemState::Merged)
        .count();
    let issues_count = issues.len();
    let merged_issues_count = issues
        .iter()
        .filter(|issue| issue.state == utils::ItemState::Closed)
        .count();
    let non_forked_repository_count = repositories
        .iter()
        .filter(|repo| repo.original_owner.is_none())
        .count();

    let mut out = String::new();
    use std::fmt::Write;

    writeln!(
        out,
        "Below is a summary of what I've worked on in GitHub since {from}."
    )?;
    writeln!(out,)?;
    writeln!(out, "First, the issues that I've opened, in JSON:")?;
    writeln!(out,)?;
    for val in issues {
        let s = serde_json::to_string_pretty(&val)?;
        writeln!(out, "{s}")?;
    }
    writeln!(out,)?;
    writeln!(out, "Next, the pull requests that I've opened, in JSON:")?;
    writeln!(out,)?;
    for val in prs {
        let s = serde_json::to_string_pretty(&val)?;
        writeln!(out, "{s}")?;
    }
    writeln!(out,)?;
    writeln!(
        out,
        "Finally, the repositories that I've created or forked (forks have a non-null 'original_owner' field), in JSON:"
    )?;
    writeln!(out,)?;
    for val in repositories {
        let s = serde_json::to_string_pretty(&val)?;
        writeln!(out, "{s}")?;
    }
    writeln!(out,)?;
    writeln!(out, "In summary, I have:")?;
    writeln!(
        out,
        "- Opened {prs_count} pull requests, of which {merged_prs_count} were merged."
    )?;
    writeln!(
        out,
        "- Opened {issues_count} issues, of which {merged_issues_count} have been closed."
    )?;
    writeln!(
        out,
        "- Created {non_forked_repository_count} repositories (not counting forks)."
    )?;

    println!("{out}");
    Ok(())
}
