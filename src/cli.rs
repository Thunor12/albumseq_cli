//! # CLI Argument Parsing Module
//!
//! This module defines the command-line interface for the albumseq CLI tool.
//! It uses the [`clap`](https://docs.rs/clap/) crate to parse arguments and subcommands.
//!
//! ## Main Structs
//! - [`Cli`]: The root struct for parsing global options and subcommands.
//! - [`Commands`]: Enum listing all supported subcommands.
//!
//! ## Supported Commands
//! - `init`: Initialize a new context file.
//! - `add-tracklist`: Add or replace a named tracklist.
//! - `add-medium`: Add or replace a named medium.
//! - `add-constraint`: Add a constraint to the context.
//! - `remove-constraint`: Remove a constraint by index.
//! - `show`: Show the current context or filtered parts of it.
//! - `propose`: Propose top scoring tracklist permutations for a tracklist & medium.
//!
//! ## Example Usage
//! ```sh
//! albumseq_cli init
//! albumseq_cli add-tracklist --name "My Album" --tracks "Song1:3:45" "Song2:4:10"
//! albumseq_cli add-medium --name "Vinyl" --sides 2 --max-duration 22:00
//! albumseq_cli add-constraint --kind adjacent --args "Song1" "Song2" --weight 2
//! albumseq_cli propose --tracklist "My Album" --medium "Vinyl" --count 10 --min-score 5
//! ```
//!
//! See each command's help (`--help`) for more details.

use crate::context::DEFAULT_CONTEXT_PATH;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Root CLI options and subcommands for albumseq_cli.
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Path to the context file (default: context.json)
    #[arg(short, long, default_value = DEFAULT_CONTEXT_PATH)]
    pub context: PathBuf,

    /// The command to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// All supported subcommands for albumseq_cli.
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new context file.
    ///
    /// Example:
    /// albumseq_cli init
    Init,

    /// Add or replace a named tracklist.
    ///
    /// Example:
    /// albumseq_cli add-tracklist --name "My Album" --tracks "Song1:3:45" "Song2:4:10"
    AddTracklist {
        /// Name of the tracklist.
        #[arg(short, long)]
        name: String,

        /// Tracks in format "Title:Duration" (duration supports MM:SS or decimal minutes).
        #[arg(short, long)]
        tracks: Vec<String>,
    },

    /// Add or replace a named medium.
    ///
    /// Example:
    /// albumseq_cli add-medium --name "Vinyl" --sides 2 --max-duration 22:00
    AddMedium {
        /// Name of the medium.
        #[arg(short, long)]
        name: String,

        /// Number of sides (integer).
        #[arg(short = 's', long)]
        sides: usize,

        /// Max duration per side (MM:SS or decimal minutes).
        #[arg(short = 'd', long)]
        max_duration: String,
    },

    /// Add a constraint to the context.
    ///
    /// Example:
    /// albumseq_cli add-constraint --kind adjacent --args "Song1" "Song2" --weight 2
    AddConstraint {
        /// Constraint kind: "atpos", "adjacent", or "onsameside".
        #[arg(short, long)]
        kind: String,

        /// Arguments depending on kind.
        #[arg(short = 'a', long)]
        args: Vec<String>,

        /// Weight of the constraint.
        #[arg(short, long, default_value = "1")]
        weight: usize,
    },

    /// Remove a constraint by index.
    ///
    /// Example:
    /// albumseq_cli remove-constraint --index 0
    RemoveConstraint {
        /// Index of the constraint to remove.
        #[arg(long)]
        index: usize,
    },

    /// Show the current context or filtered parts of it.
    ///
    /// Example:
    /// albumseq_cli show --filter tracklists
    Show {
        /// Filter what to show: "tracklists", "media", "constraints", or leave empty for all.
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Propose top scoring tracklist permutations for a tracklist & medium.
    ///
    /// Example:
    /// albumseq_cli propose --tracklist "My Album" --medium "Vinyl" --count 10 --min-score 5
    Propose {
        /// Tracklist name to use.
        #[arg(short, long)]
        tracklist: String,

        /// Medium name to use.
        #[arg(short, long)]
        medium: String,

        /// Number of propositions to show.
        #[arg(short, long, default_value = "15")]
        count: usize,

        /// Minimum score to include (optional).
        #[arg(short = 'm', long)]
        min_score: Option<usize>,
    },
}
