use std::{env, fs};
use crate::state::{State, TagInfo};
use octocrab::Octocrab;
use octocrab::models::repos::Object;
use octocrab::params::repos::Reference;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

mod config;
mod state;

fn main() {
    if let Err(err) = async_main() {
        panic!("Unhandled error: {:?}", err);
    }
}

#[tokio::main]
async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let data_folder = env::var("DATA_FOLDER")?;
    fs::create_dir_all(&data_folder)?;

    let config = config::Config::load(&data_folder)?;
    let mut state = State::load(&data_folder);

    let octo = Octocrab::builder()
        .personal_token(env::var("GITHUB_TOKEN")?)
        .build()?;

    loop {
        for entry in &config.entries {
            for_repo(
                &octo,
                &mut state,
                &entry.upstream_owner,
                &entry.upstream_repo,
                &entry.upstream_branch,
                &entry.fork_owner,
                &entry.fork_repo,
                &entry.fork_branch,
            )
            .await;
        }

        state.save(&data_folder);

        tokio::time::sleep(Duration::from_millis(60 * 60 * 1000)).await;
    }
}

async fn for_repo(
    octo: &Octocrab,
    state: &mut State,
    upstream_owner: &String,
    upstream_repo: &String,
    upstream_branch: &String,
    fork_owner: &String,
    fork_repo: &String,
    fork_branch: &String,
) {
    let upstream_tags = octo
        .repos(upstream_owner, upstream_repo)
        .list_tags()
        .per_page(1)
        .send()
        .await
        .unwrap();

    let upstream_tag = upstream_tags.items.first().unwrap();

    let tag_state = state.repo_mut(upstream_owner, upstream_repo);
    tag_state.swap_with_new(upstream_tag);

    if let (
        Some(TagInfo {
            name: latest_name, ..
        }),
        Some(TagInfo {
            name: previous_name,
            ..
        }),
    ) = (&tag_state.latest_tag, &tag_state.previous_tag)
    {
        if latest_name == previous_name {
            info!("No new tag for {upstream_owner}/{upstream_repo}");
            return;
        }

        info!(
            "New upstream tag detected for {upstream_owner}/{upstream_repo}: {previous_name} → {latest_name}"
        );

        let compare = octo
            .commits(upstream_owner, upstream_repo)
            .compare(
                upstream_branch,
                format!("{fork_owner}:{fork_repo}:{fork_branch}",),
            )
            .send()
            .await
            .unwrap();

        if compare.behind_by != 0 {
            info!("Fork {fork_owner}/{fork_repo} is NOT up to date; skipping tag push.");
            return;
        }

        info!("Fork is up to date; pushing tag {latest_name} to {fork_owner}/{fork_repo}");

        if let Err(err) =
            push_tag_to_fork(octo, fork_owner, fork_repo, fork_branch, latest_name).await
        {
            info!("Failed to push tag: {err}");
        }
    }
}

async fn push_tag_to_fork(
    octo: &Octocrab,
    fork_owner: &String,
    fork_repo: &String,
    fork_branch: &String,
    tag_name: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = octo.repos(fork_owner, fork_repo);

    let object = repo
        .get_ref(&Reference::Branch(fork_branch.clone()))
        .await?
        .object;

    let sha = match object {
        Object::Commit { sha, .. } => sha,
        Object::Tag { sha, .. } => sha,
        _ => panic!("Invalid object type"),
    };

    repo.create_ref(&Reference::Tag(tag_name.into()), sha)
        .await?;

    info!("Pushed tag {tag_name} to {fork_owner}/{fork_repo}");

    Ok(())
}
