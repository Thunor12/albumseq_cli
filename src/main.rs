// mod cli;
mod commands;
mod context;
mod utils;

// use crate::cli::{Cli, Commands};
use crate::commands::{
    handle_add_constraint, handle_add_medium, handle_add_tracklist, handle_propose,
    handle_remove_constraint, handle_show,
};
use crate::context::ProgramContext;
use crate::utils::parse_duration;

use albumseq::{
    Constraint as AlbumConstraint, ConstraintKind as AlbumConstraintKind, Medium as AlbumMedium,
    Track, Tracklist,
};
use clap::{Parser, Subcommand};

use std::path::{Path, PathBuf};

const DEFAULT_CONTEXT_PATH: &str = "context.json";

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
    /// Add or replace a named tracklist
    AddTracklist {
        /// Name of the tracklist
        #[arg(short, long)]
        name: String,

        /// Tracks in format "Title:Duration" (duration supports MM:SS or decimal minutes)
        #[arg(short, long)]
        tracks: Vec<String>,
    },
    /// Add or replace a named medium
    AddMedium {
        /// Name of the medium
        #[arg(short, long)]
        name: String,

        /// Number of sides (integer)
        #[arg(short = 's', long)]
        sides: usize,

        /// Max duration per side (MM:SS or decimal minutes)
        #[arg(short = 'd', long)]
        max_duration: String,
    },
    /// Add a constraint
    AddConstraint {
        /// Constraint kind: "atpos", "adjacent", or "onsameside"
        #[arg(short, long)]
        kind: String,

        /// Arguments depending on kind
        #[arg(short = 'a', long)]
        args: Vec<String>,

        /// Weight of the constraint
        #[arg(short, long, default_value = "1")]
        weight: usize,
    },
    /// Remove a constraint
    RemoveConstraint {
        #[arg(long)]
        index: usize,
    },
    /// Show the entire context
    Show {
        /// Filter what to show: "tracklists", "media", "constraints", or leave empty for all
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Propose top scoring tracklist permutations for a tracklist & medium
    Propose {
        /// Tracklist name to use
        #[arg(short, long)]
        tracklist: String,

        /// Medium name to use
        #[arg(short, long)]
        medium: String,

        /// Number of propositions to show
        #[arg(short, long, default_value = "15")]
        count: usize,

        /// Minimum score to include (optional)
        #[arg(short = 'm', long)]
        min_score: Option<usize>,
    },
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
                        let title = parts[0].to_string();
                        let duration_str = parts[1];
                        parse_duration(duration_str).map(|duration| Track { title, duration })
                    } else {
                        None
                    }
                })
                .collect();

            if !parsed_tracks.is_empty() {
                if handle_add_tracklist(&mut ctx, name, parsed_tracks) {
                    ctx.save(&cli.context);
                }
            } else {
                eprintln!("No valid tracks provided for tracklist '{}'", name);
            }
        }

        Commands::AddMedium {
            name,
            sides,
            max_duration,
        } => {
            let mut ctx = ProgramContext::load_or_create(&cli.context);

            if let Some(duration) = parse_duration(max_duration) {
                if handle_add_medium(&mut ctx, name, *sides, duration) {
                    ctx.save(&cli.context);
                }
            } else {
                eprintln!("Invalid duration format: {}", max_duration);
            }
        }

        Commands::AddConstraint { kind, args, weight } => {
            let mut ctx = ProgramContext::load_or_create(&cli.context);

            if handle_add_constraint(&mut ctx, kind, args, *weight) {
                ctx.save(&cli.context);
            }
        }

        Commands::RemoveConstraint { index } => {
            let mut ctx = ProgramContext::load_or_create(&cli.context);
            if handle_remove_constraint(&mut ctx, index) {
                ctx.save(&cli.context);
            }
        }

        Commands::Show { filter } => {
            let ctx = ProgramContext::load_or_create(&cli.context);
            handle_show(&ctx, filter);
        }

        Commands::Propose {
            tracklist,
            medium,
            count,
            min_score,
        } => {
            let ctx = ProgramContext::load_or_create(&cli.context);
            handle_propose(&ctx, tracklist, medium, count, min_score);
        }
    }
}
