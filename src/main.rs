use albumseq::{Duration, Track, Tracklist};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const DEFAULT_CONTEXT_PATH: &str = "context.json";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SerTrack {
    pub title: String,
    pub duration: Duration,
}

impl From<&Track> for SerTrack {
    fn from(t: &Track) -> Self {
        SerTrack {
            title: t.title.clone(),
            duration: t.duration,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SerTracklist(pub Vec<SerTrack>);

impl From<Tracklist> for SerTracklist {
    fn from(tl: Tracklist) -> Self {
        SerTracklist(tl.0.iter().map(|t| t.into()).collect())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NamedSerTracklist {
    pub name: String,
    pub tracks: SerTracklist,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ProgramContext {
    pub tracklists: Vec<NamedSerTracklist>,
}

impl ProgramContext {
    fn load_or_create(path: &Path) -> Self {
        if path.exists() {
            let data = fs::read_to_string(path).expect("Failed to read context file");
            serde_json::from_str(&data).expect("Failed to parse context file")
        } else {
            let ctx = Self::default();
            ctx.save(path);
            ctx
        }
    }

    fn save(&self, path: &Path) {
        let json = serde_json::to_string_pretty(self).expect("Failed to serialize context");
        fs::write(path, json).expect("Failed to write context file");
    }

    /// Add or replace a tracklist by name
    fn add_or_replace_tracklist(&mut self, name: String, tracks: Vec<Track>) {
        let new_list = NamedSerTracklist {
            name: name.clone(),
            tracks: SerTracklist(tracks.iter().map(|t| t.into()).collect()),
        };

        if let Some(existing) = self
            .tracklists
            .iter_mut()
            .find(|tl| tl.name.eq_ignore_ascii_case(&name))
        {
            *existing = new_list;
            println!("Replaced tracklist '{}'", name);
        } else {
            self.tracklists.push(new_list);
            println!("Added tracklist '{}'", name);
        }
    }
}

/// Format a duration in minutes (f64) as "MM:SS"
fn format_duration(duration: Duration) -> String {
    let total_seconds = (duration * 60.0).round() as u64;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long, default_value = DEFAULT_CONTEXT_PATH)]
    context: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new context file
    Init,
    /// Add a new named tracklist to the context (replaces if same name exists)
    AddTracklist {
        /// Name of the tracklist
        #[arg(short, long)]
        name: String,

        /// Tracks in the format "Title:Duration"
        #[arg(short, long)]
        tracks: Vec<String>,
    },
    /// Show the context
    Show,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            if Path::new(&cli.context).exists() {
                eprintln!("Context file already exists at {:?}", cli.context);
            } else {
                let ctx = ProgramContext::default();
                ctx.save(&cli.context);
                println!("Created new context at {:?}", cli.context);
            }
        }
        Commands::AddTracklist { name, tracks } => {
            let mut ctx = ProgramContext::load_or_create(&cli.context);

            let parsed_tracks: Vec<Track> = tracks
                .iter()
                .filter_map(|s| {
                    let parts: Vec<_> = s.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        parts[1].parse::<f64>().ok().map(|duration| Track {
                            title: parts[0].to_string(),
                            duration,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            ctx.add_or_replace_tracklist(name.clone(), parsed_tracks);
            ctx.save(&cli.context);
        }
        Commands::Show => {
            let ctx = ProgramContext::load_or_create(&cli.context);

            if ctx.tracklists.is_empty() {
                println!("No tracklists in context.");
                return;
            }

            for tl in &ctx.tracklists {
                println!("=== {} ===", tl.name);

                let max_title_len = tl
                    .tracks
                    .0
                    .iter()
                    .map(|t| t.title.len())
                    .max()
                    .unwrap_or(5)
                    .max("Title".len());

                let mut total_duration: Duration = 0.0;

                println!(
                    "{:<width$} {:>8}",
                    "Title",
                    "Duration",
                    width = max_title_len
                );
                println!("{} {}", "-".repeat(max_title_len), "-".repeat(8));

                for t in &tl.tracks.0 {
                    println!(
                        "{:<width$} {:>8}",
                        t.title,
                        format_duration(t.duration),
                        width = max_title_len
                    );
                    total_duration += t.duration;
                }

                println!(
                    "{:<width$} {:>8}",
                    "TOTAL",
                    format_duration(total_duration),
                    width = max_title_len
                );
                println!();
            }
        }
    }
}
