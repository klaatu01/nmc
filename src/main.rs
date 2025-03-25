use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Clone)]
pub enum Status {
    Waiting,
    Deleting,
    Failed,
    Done,
}

#[derive(Clone)]
pub struct StatusUpdate {
    pub path: PathBuf,
    pub status: Status,
}

#[derive(Clone)]
pub struct Project {
    pub path: PathBuf,
    pub status: Status,
}

impl Project {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            status: Status::Waiting,
        }
    }

    pub fn update_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

fn identify_projects(depth: usize) -> Vec<Project> {
    let mut projects = Vec::new();
    let mut walker = WalkDir::new(".").max_depth(depth).into_iter();

    while let Some(entry) = walker.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Error reading entry: {}", err);
                continue;
            }
        };

        if entry.file_type().is_file() && entry.file_name() == "package.json" {
            if let Some(project_dir) = entry.path().parent() {
                if project_dir.join("node_modules").exists() {
                    projects.push(Project::new(project_dir.to_owned()));
                }
            }
            // Skip descending further into this directory since we've found package.json here.
            walker.skip_current_dir();
        }
    }

    projects
}

async fn status_poller(rx: async_channel::Receiver<StatusUpdate>, projects: Vec<Project>) {
    // Create a multi-progress to manage multiple lines of output
    let multi_progress = MultiProgress::new();

    // Prepare to store our projects and associated progress bars
    let mut project_map: HashMap<PathBuf, Project> =
        projects.into_iter().map(|p| (p.path(), p)).collect();

    // A map of path -> ProgressBar
    let mut bars = HashMap::new();

    // Create a progress bar per project
    for project in project_map.values() {
        let pb = multi_progress.add(ProgressBar::new_spinner());
        pb.set_message(format!("Waiting: {}", project.path().display()));
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
                .template("{spinner} {wide_msg}")
                .unwrap(),
        );
        // Spin at regular intervals
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        bars.insert(project.path(), pb);
    }

    // Keep receiving status updates
    while let Ok(update) = rx.recv().await {
        if let Some(project) = project_map.get_mut(&update.path) {
            project.update_status(update.status.clone());
            if let Some(pb) = bars.get(&update.path) {
                match update.status {
                    Status::Waiting => {
                        // Typically won't happen once we start, but let's handle it anyway
                        pb.set_message(format!("{}", project.path().display()));
                    }
                    Status::Deleting => {
                        pb.set_message(format!("{}", project.path().display()));
                    }
                    Status::Done => {
                        pb.finish_with_message(format!("✓ {}", project.path().display()));
                    }
                    Status::Failed => {
                        pb.finish_with_message(format!("✗ {}", project.path().display()));
                    }
                }
            }
        }
    }
}

async fn delete_projects(projects: Vec<Project>, tx: async_channel::Sender<StatusUpdate>) {
    futures::stream::iter(
        projects
            .into_iter()
            .map(|p| delete_project(p.path(), tx.clone())),
    )
    .buffer_unordered(4)
    .collect::<Vec<_>>()
    .await;
}

async fn delete_project(path: PathBuf, tx: async_channel::Sender<StatusUpdate>) {
    tx.send(StatusUpdate {
        path: path.clone(),
        status: Status::Deleting,
    })
    .await
    .unwrap();

    let node_modules_dir = path.canonicalize().unwrap().join("node_modules");

    match tokio::fs::remove_dir_all(node_modules_dir).await {
        Ok(_) => {
            tx.send(StatusUpdate {
                path,
                status: Status::Done,
            })
            .await
            .unwrap();
        }
        Err(_) => {
            tx.send(StatusUpdate {
                path,
                status: Status::Failed,
            })
            .await
            .unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    let projects = identify_projects(2);

    let (tx, rx) = async_channel::unbounded();

    tokio::spawn(delete_projects(projects.clone(), tx));

    status_poller(rx, projects).await;
}
