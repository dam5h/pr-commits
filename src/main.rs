use clap::Parser;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// GitHub repository owner (e.g., "your_org")
    #[arg(short, long)]
    owner: String,

    /// GitHub repository name (e.g., "your_repo")
    #[arg(short, long)]
    repo: String,

    /// Path to the file containing your GitHub token
    #[arg(short, long)]
    token_path: PathBuf,

    /// List of pull request numbers to fetch
    #[arg(short, long, required = true, num_args=1..)]
    prs: Vec<u32>,
}

#[derive(Deserialize, Debug)]
struct Commit {
    sha: String,
    commit: CommitInfo,
}

#[derive(Deserialize, Debug)]
struct PullRequest {
    title: String,
}

#[derive(Deserialize, Debug)]
struct CommitInfo {
    author: UserInfo,
    message: String,
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    name: String,
    date: String,
}

async fn fetch_pr_title(
    owner: &str,
    repo: &str,
    pr_number: u32,
    token: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, pr_number
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token))?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("rust-client"));

    let response = reqwest::Client::new()
        .get(&url)
        .headers(headers)
        .send()
        .await?
        .json::<PullRequest>()
        .await?;

    Ok(response.title)
}

async fn fetch_commits_for_pr(
    owner: &str,
    repo: &str,
    pr_number: u32,
    token: &str,
) -> Result<Vec<Commit>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/commits",
        owner, repo, pr_number
    );

    // Set up headers
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token))?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("rust-client"));

    // Make the API request
    let response = reqwest::Client::new()
        .get(&url)
        .headers(headers)
        .send()
        .await?
        .json::<Vec<Commit>>()
        .await?;

    Ok(response)
}

fn print_commit_table(pr_number: u32, pr_title: &str, commits: &[Commit]) {
    println!("PR #{} - {}", pr_number, pr_title);
    println!(
        "{:<40} | {:<25} | {:<20} | {}",
        "Commit SHA", "Date", "Author", "Message"
    );
    println!("{:-<40}-+-{:-<25}-+-{:-<60}", "", "", "");

    for commit in commits {
        println!(
            "{:<40} | {:<25} | {:<20} | {}",
            commit.sha,
            commit.commit.author.date,
            commit.commit.author.name,
            commit.commit.message.lines().next().unwrap_or("")
        );
    }
    println!("\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read the token from the provided file path
    let token = std::fs::read_to_string(&args.token_path)?
        .trim()
        .to_string();

    for &pr_number in &args.prs {
        let pr_title = fetch_pr_title(&args.owner, &args.repo, pr_number, &token).await?;
        let commits = fetch_commits_for_pr(&args.owner, &args.repo, pr_number, &token).await?;
        print_commit_table(pr_number, &pr_title, &commits);
    }

    Ok(())
}
