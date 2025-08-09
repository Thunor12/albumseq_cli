// mod cli;
// mod commands;
mod context;
mod utils;

// use crate::cli::{Cli, Commands};
// use crate::commands::handle_propose;
use crate::context::ProgramContext;
use crate::utils::{format_duration, parse_duration};

use albumseq::{
    Constraint as AlbumConstraint, ConstraintKind as AlbumConstraintKind, Duration,
    Medium as AlbumMedium, Track, Tracklist, TracklistPermutations, score_tracklist,
};
use clap::{Parser, Subcommand};

use std::path::{Path, PathBuf};

const DEFAULT_CONTEXT_PATH: &str = "context.json";

/// Helper: Split a tracklist into sides based on medium max duration per side
fn split_tracklist_by_side<'a>(
    tracklist: &'a Tracklist,
    medium: &'a AlbumMedium,
) -> Vec<Vec<&'a Track>> {
    let mut sides = Vec::new();
    let mut current_side = Vec::new();
    let mut current_duration = 0.0;

    for track in &tracklist.0 {
        if current_duration + track.duration <= medium.max_duration_per_side {
            current_side.push(track);
            current_duration += track.duration;
        } else {
            sides.push(current_side);
            current_side = vec![track];
            current_duration = track.duration;
        }

        if sides.len() == medium.sides {
            break;
        }
    }

    if !current_side.is_empty() {
        sides.push(current_side);
    }

    sides
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

fn parse_constraint_kind(kind: &str, args: &[String]) -> Option<AlbumConstraintKind> {
    match kind.to_lowercase().as_str() {
        "atpos" => {
            if args.len() == 2 {
                let pos = args[1].parse::<usize>();
                if let Ok(pos) = pos {
                    Some(AlbumConstraintKind::AtPosition(args[0].clone(), pos))
                } else {
                    eprintln!("Invalid position number: {}", args[1]);
                    None
                }
            } else {
                eprintln!("AtPosition constraint requires exactly 2 arguments: title pos");
                None
            }
        }
        "adjacent" => {
            if args.len() == 2 {
                Some(AlbumConstraintKind::Adjacent(
                    args[0].clone(),
                    args[1].clone(),
                ))
            } else {
                eprintln!("Adjacent constraint requires exactly 2 arguments: title1 title2");
                None
            }
        }
        "onsameside" => {
            if args.len() == 2 {
                Some(AlbumConstraintKind::OnSameSide(
                    args[0].clone(),
                    args[1].clone(),
                ))
            } else {
                eprintln!("OnSameSide constraint requires exactly 2 arguments: title1 title2");
                None
            }
        }
        _ => {
            eprintln!("Unknown constraint kind: {}", kind);
            None
        }
    }
}

fn handle_propose(
    cli: &Cli,
    tracklist_name: &str,
    medium_name: &str,
    count: &usize,
    min_score: &Option<usize>,
) {
    let ctx = ProgramContext::load_or_create(&cli.context);

    // Find the tracklist by name
    let ser_tl = ctx
        .tracklists
        .iter()
        .find(|tl| tl.name.eq_ignore_ascii_case(tracklist_name));

    if ser_tl.is_none() {
        eprintln!("Tracklist '{}' not found", tracklist_name);
        return;
    }
    let ser_tl = ser_tl.unwrap();
    let tracklist = Tracklist::from(&ser_tl.tracks);

    // Find the medium by name
    let ser_medium = ctx
        .mediums
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case(medium_name));
    if ser_medium.is_none() {
        eprintln!("Medium '{}' not found", medium_name);
        return;
    }
    let ser_medium = ser_medium.unwrap();
    let medium = ser_medium.to_album_medium();

    // Convert constraints to albumseq constraints
    let constraints: Vec<AlbumConstraint> =
        ctx.constraints.iter().cloned().map(|c| c.into()).collect();

    // Create permutations iterator
    let perms = TracklistPermutations::new(&tracklist.0);

    // Score permutations, filter by min_score if provided, keep top `count` by descending score
    let mut scored_perms: Vec<(usize, Tracklist)> = perms
        .map(|perm| {
            let tl = Tracklist(perm.into_iter().cloned().collect());
            let score = score_tracklist(&tl, &constraints, &medium);
            (score, tl)
        })
        .filter(|(score, tl)| medium.fits(tl) && min_score.map_or(true, |min| *score >= min))
        .collect();

    scored_perms.sort_by(|a, b| b.0.cmp(&a.0)); // descending by score

    if let Some(min) = min_score {
        println!(
            "Top {} permutations for tracklist '{}' on medium '{}' with score >= {}:",
            count, tracklist_name, medium_name, min
        );
    } else {
        println!(
            "Top {} permutations for tracklist '{}' on medium '{}':",
            count, tracklist_name, medium_name
        );
    }

    for (score, tl) in scored_perms.into_iter().take(*count) {
        println!("Score: {}", score);

        let max_title_len =
            tl.0.iter()
                .map(|t| t.title.len())
                .max()
                .unwrap_or(5)
                .max("Title".len());

        println!(
            "{:<width$} {:>8}",
            "Title",
            "Duration",
            width = max_title_len
        );
        println!("{} {}", "-".repeat(max_title_len), "-".repeat(8));

        let sides = split_tracklist_by_side(&tl, &medium);

        for (side_idx, side_tracks) in sides.iter().enumerate() {
            println!("----- side {} --------", side_idx);
            for t in side_tracks {
                println!(
                    "{:<width$} {:>8}",
                    t.title,
                    format_duration(t.duration),
                    width = max_title_len
                );
            }
        }

        let total_duration: Duration = tl.0.iter().map(|t| t.duration).sum();

        println!(
            "{:<width$} {:>8}",
            "TOTAL",
            format_duration(total_duration),
            width = max_title_len
        );
        println!();
    }
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

            ctx.add_or_replace_tracklist(name.clone(), parsed_tracks);
            ctx.save(&cli.context);
        }

        Commands::AddMedium {
            name,
            sides,
            max_duration,
        } => {
            let mut ctx = ProgramContext::load_or_create(&cli.context);
            if let Some(duration) = parse_duration(max_duration) {
                ctx.add_or_replace_medium(name.clone(), *sides, duration);
                ctx.save(&cli.context);
            } else {
                eprintln!("Invalid duration format: {}", max_duration);
            }
        }

        Commands::AddConstraint { kind, args, weight } => {
            let mut ctx = ProgramContext::load_or_create(&cli.context);

            if let Some(kind) = parse_constraint_kind(kind, args) {
                let constraint = AlbumConstraint {
                    kind,
                    weight: *weight,
                };
                ctx.add_or_replace_constraint(constraint);
                ctx.save(&cli.context);
            }
        }

        Commands::RemoveConstraint { index } => {
            let mut ctx = ProgramContext::load_or_create(&cli.context);
            let before_len = ctx.constraints.len();

            let cc = ctx.constraints.clone();
            let cc = cc.get(*index);

            if let Some(c) = cc {
                ctx.constraints.remove(*index);
                println!("Removed constraint at index {}", index);
                println!("=== Constraint ===");
                println!("{:?} (weight {})", c.kind, c.weight);
                println!();
            } else {
                eprintln!("Index out of range");
                return;
            }

            ctx.save(&cli.context);
            println!("{} constraints removed", before_len - ctx.constraints.len());
        }

        Commands::Show { filter } => {
            let ctx = ProgramContext::load_or_create(&cli.context);

            let filter = filter.as_deref().unwrap_or("all").to_lowercase();

            if filter == "all" || filter == "tracklists" {
                println!("--- Tracklists ---");
                for (i, tl) in ctx.tracklists.iter().enumerate() {
                    println!("Tracklist {}:", i);
                    for track in tl.tracks.0.iter() {
                        println!("  {} ({})", track.title, track.duration);
                    }
                }
            }

            if filter == "all" || filter == "media" {
                println!("--- Media ---");
                for m in &ctx.mediums {
                    println!(
                        "Medium: {} | Sides: {} | Max per side: {} sec",
                        m.name, m.sides, m.max_duration_per_side
                    );
                }
            }

            if filter == "all" || filter == "constraints" {
                println!("=== Constraints ===");
                for c in &ctx.constraints {
                    println!("{:?} (weight {})", c.kind, c.weight);
                }
                println!();
            }
        }

        Commands::Propose {
            tracklist,
            medium,
            count,
            min_score,
        } => {
            handle_propose(&cli, tracklist, medium, count, min_score);
        }
    }
}
