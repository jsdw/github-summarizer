use crate::api::client::Api;
use crate::variables;
use crate::utils::{DateTime, ItemState};

const QUERY: &str = r#"
    query PullRequestContributions($user:String!, $from:DateTime!, $to:DateTime!, $cursor:String) {
        user(login:$user) {
            contributions_collection: contributionsCollection(from:$from, to:$to) {
                pull_request_contributions: pullRequestContributions(first:100, after:$cursor) {
                    page_info: pageInfo {
                        end_cursor: endCursor,
                        has_next_page: hasNextPage
                    }
                    nodes {
                        pull_request: pullRequest {
                            repository {
                                name,
                                owner { login }
                            },
                            title,
                            state,
                            created_at: createdAt,
                            body_text: bodyText,
                        }
                    }
                }
            }
        }
    }
"#;

#[derive(serde::Deserialize)]
struct QueryResult {
    user: QueryUser,
}

#[derive(serde::Deserialize)]
struct QueryUser {
    contributions_collection: ContributionsCollection,
}

#[derive(serde::Deserialize)]
struct ContributionsCollection {
    pull_request_contributions: PullRequestContributions,
}

#[derive(serde::Deserialize)]
struct PullRequestContributions {
    page_info: QueryPageInfo,
    nodes: Vec<PullRequestContributionNodes>,
}

#[derive(serde::Deserialize)]
struct QueryPageInfo {
    end_cursor: Option<String>,
    has_next_page: bool,
}

#[derive(serde::Deserialize)]
struct PullRequestContributionNodes {
    pull_request: PullRequestInfo,
}

#[derive(serde::Deserialize)]
struct PullRequestInfo {
    repository: QueryRepository,
    title: String,
    state: ItemState,
    created_at: DateTime,
    body_text: String,
}

#[derive(serde::Deserialize)]
struct QueryRepository {
    name: String,
    owner: QueryRepositoryOwner,
}

#[derive(serde::Deserialize)]
struct QueryRepositoryOwner {
    login: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PullRequest {
    pub repository: String,
    pub owner: String,
    pub title: String,
    pub state: ItemState,
    pub created_at: DateTime,
    pub body_text: String,
}

pub async fn query(api: &Api, created_after: DateTime, created_before: DateTime) -> Result<Vec<PullRequest>, anyhow::Error> {
    let user = api.user();

    let mut items = vec![];
    let mut cursor = None;

    loop {
        let res: QueryResult = api.query(QUERY, variables!(
            "user": &user,
            "from": created_after,
            "to": created_before,
            "cursor": cursor
        )).await?;

        let pr_contributions = res.user.contributions_collection.pull_request_contributions;

        for pr in pr_contributions.nodes {
            let item = PullRequest {
                repository: pr.pull_request.repository.name,
                owner: pr.pull_request.repository.owner.login,
                title: pr.pull_request.title,
                state: pr.pull_request.state,
                created_at: pr.pull_request.created_at,
                body_text: pr.pull_request.body_text,
            };
            items.push(item);
        }

        cursor = pr_contributions.page_info.end_cursor;
        if !pr_contributions.page_info.has_next_page || cursor.is_none() {
            break
        }
    }

    items.sort_by_key(|item| item.created_at);

    Ok(items)
}