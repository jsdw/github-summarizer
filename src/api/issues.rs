use crate::api::client::Api;
use crate::utils::{DateTime, ItemState};
use crate::variables;

const QUERY: &str = r#"
    query IssueContributions($user:String!, $from:DateTime!, $to:DateTime!, $cursor:String) {
        user(login:$user) {
            contributions_collection: contributionsCollection(from:$from, to:$to) {
                issue_contributions: issueContributions(first:100, after:$cursor) {
                    page_info: pageInfo {
                        end_cursor: endCursor,
                        has_next_page: hasNextPage
                    }
                    nodes {
                        issue {
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
    issue_contributions: IssueContributions,
}

#[derive(serde::Deserialize)]
struct IssueContributions {
    page_info: QueryPageInfo,
    nodes: Vec<IssueContributionNode>,
}

#[derive(serde::Deserialize)]
struct QueryPageInfo {
    end_cursor: Option<String>,
    has_next_page: bool,
}

#[derive(serde::Deserialize)]
struct IssueContributionNode {
    issue: IssueInfo,
}

#[derive(serde::Deserialize)]
struct IssueInfo {
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
pub struct Issue {
    pub repository: String,
    pub owner: String,
    pub title: String,
    pub state: ItemState,
    pub created_at: DateTime,
    pub body_text: String,
}

pub async fn query(
    api: &Api,
    created_after: DateTime,
    created_before: DateTime,
) -> Result<Vec<Issue>, anyhow::Error> {
    let user = api.user();

    let mut items = vec![];
    let mut cursor = None;

    loop {
        let res: QueryResult = api
            .query(
                QUERY,
                variables!(
                    "user": &user,
                    "from": created_after,
                    "to": created_before,
                    "cursor": cursor
                ),
            )
            .await?;

        let pr_contributions = res.user.contributions_collection.issue_contributions;

        for pr in pr_contributions.nodes {
            let item = Issue {
                repository: pr.issue.repository.name,
                owner: pr.issue.repository.owner.login,
                title: pr.issue.title,
                state: pr.issue.state,
                created_at: pr.issue.created_at,
                body_text: pr.issue.body_text,
            };
            items.push(item);
        }

        cursor = pr_contributions.page_info.end_cursor;
        if !pr_contributions.page_info.has_next_page || cursor.is_none() {
            break;
        }
    }

    items.sort_by_key(|item| item.created_at);

    Ok(items)
}
