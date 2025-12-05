use crate::api::client::Api;
use crate::utils::DateTime;
use crate::variables;

const QUERY: &str = r#"
    query RepositoriesCreated($user:String!, $from:DateTime!, $to:DateTime!, $cursor:String) {
        user(login:$user) {
            contributions_collection: contributionsCollection(from:$from, to:$to) {
                repository_contributions: repositoryContributions(first:100, after:$cursor) {
                    page_info: pageInfo {
                        end_cursor: endCursor,
                        has_next_page: hasNextPage
                    }
                    nodes {
                        repository {
                            name,
                            description,
                            parent { owner { login } },
                            owner { login },
                            created_at: createdAt,
                            url
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
    repository_contributions: RepositoryContributions,
}

#[derive(serde::Deserialize)]
struct RepositoryContributions {
    page_info: QueryPageInfo,
    nodes: Vec<RepositoryContributionNode>,
}

#[derive(serde::Deserialize)]
struct QueryPageInfo {
    end_cursor: Option<String>,
    has_next_page: bool,
}

#[derive(serde::Deserialize)]
struct RepositoryContributionNode {
    repository: RepositoryInfo,
}

#[derive(serde::Deserialize)]
struct RepositoryInfo {
    name: String,
    description: Option<String>,
    parent: Option<RepositoryParent>,
    owner: RepositoryOwner,
    created_at: DateTime,
    url: String,
}

#[derive(serde::Deserialize)]
struct RepositoryParent {
    owner: RepositoryOwner,
}

#[derive(serde::Deserialize)]
struct RepositoryOwner {
    login: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Repository {
    pub name: String,
    pub description: Option<String>,
    pub owner: String,
    pub original_owner: Option<String>,
    pub created_at: DateTime,
    pub url: String,
}

pub async fn query(
    api: &Api,
    created_after: DateTime,
    created_before: DateTime,
) -> Result<Vec<Repository>, anyhow::Error> {
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

        let repo_contributions = res.user.contributions_collection.repository_contributions;

        for node in repo_contributions.nodes {
            let repo = node.repository;
            let item = Repository {
                name: repo.name,
                description: repo.description,
                owner: repo.owner.login,
                original_owner: repo.parent.map(|p| p.owner.login),
                created_at: repo.created_at,
                url: repo.url,
            };
            items.push(item);
        }

        cursor = repo_contributions.page_info.end_cursor;
        if !repo_contributions.page_info.has_next_page || cursor.is_none() {
            break;
        }
    }

    items.sort_by_key(|item| item.created_at);

    Ok(items)
}
