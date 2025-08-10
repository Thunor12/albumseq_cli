use clap::{Parser, Subcommand};
use crate::context::DEFAULT_CONTEXT_PATH;
use std::path::PathBuf;

const DEFAULT_CONTEXT_PATH: &str = "context.json";

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short, long, default_value = DEFAULT_CONTEXT_PATH)]
    pub context: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
