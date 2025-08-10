//! # albumseq_cli Main Entry Point
//!
//! **Mission:**  
//! `albumseq_cli` helps musicians, producers, and collectors sequence album tracks for physical media (like vinyl, cassette, or CD) by optimizing track order and side splits according to user-defined constraints and media limitations.
//!
//! This is the main entry point for the albumseq_cli application.
//! It parses command-line arguments, loads or creates the persistent context, and
//! dispatches to the appropriate command handler.
//!
//! ## Example
//! ```sh
//! albumseq_cli add-tracklist --name "MyPlaylist" --tracks "Intro:3:30" "Song 1:4.2"
//! albumseq_cli add-medium --name "Vinyl" --sides 2 --max-duration 20:00
//! albumseq_cli add-constraint --kind atpos --args Intro 0 --weight 50
//! albumseq_cli propose --tracklist "MyAlbum" --medium "Vinyl" --count 10
//! ```

mod cli;
mod commands;
mod context;
mod utils;

use std::path::Path;

use crate::cli::{Cli, Commands};
use crate::commands::{
    handle_add_constraint, handle_add_medium, handle_add_tracklist, handle_propose,
    handle_remove_constraint, handle_show,
};
use crate::context::ProgramContext;
use crate::utils::parse_duration;
use albumseq::Track;
use clap::Parser;

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
