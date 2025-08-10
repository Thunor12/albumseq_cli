//! # Command Handlers
//!
//! This module contains functions that implement the logic for each CLI command.
//! Each handler is responsible for updating the context, performing calculations,
//! or displaying output as needed.
//!
//! ## Example
//! ```rust
//! handle_add_tracklist(&mut ctx, &name, tracks);
//! handle_propose(&ctx, &tracklist, &medium, &count, &min_score);
//! ```

use crate::context::ProgramContext;
use crate::utils::format_duration;
use albumseq::{
    Constraint as AlbumConstraint, ConstraintKind as AlbumConstraintKind, Duration,
    Medium as AlbumMedium, Track, Tracklist, TracklistPermutations, score_tracklist,
};
use colored::*; // Add this at the top for colored output

/// Parses a constraint kind and its arguments from CLI input.
/// Returns `Some(AlbumConstraintKind)` if parsing is successful, or `None` if invalid.
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

/// Splits a tracklist into sides based on medium max duration per side.
/// Returns a vector of vectors, each representing a side.
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

/// Handles adding a new tracklist to the context.
/// Returns true if the tracklist was added or replaced.
pub fn handle_add_tracklist(ctx: &mut ProgramContext, name: &String, tracks: Vec<Track>) -> bool {
    ctx.add_or_replace_tracklist(name.clone(), tracks);

    true
}

/// Handles adding a new medium to the context.
/// Returns true if the medium was added or replaced.
pub fn handle_add_medium(
    ctx: &mut ProgramContext,
    name: &String,
    sides: usize,
    max_duration: Duration,
) -> bool {
    ctx.add_or_replace_medium(name.clone(), sides, max_duration);

    true
}

/// Handles adding a constraint to the context.
/// Returns true if the constraint was added.
pub fn handle_add_constraint(
    ctx: &mut ProgramContext,
    kind: &String,
    args: &Vec<String>,
    weight: usize,
) -> bool {
    if let Some(kind) = parse_constraint_kind(kind, args) {
        let constraint = AlbumConstraint {
            kind,
            weight: weight,
        };
        ctx.add_or_replace_constraint(constraint);
        return true;
    }

    false
}

/// Handles removing a constraint from the context by index.
/// Returns true if the constraint was removed.
pub fn handle_remove_constraint(ctx: &mut ProgramContext, index: &usize) -> bool {
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
        return false;
    }

    println!("{} constraints removed", before_len - ctx.constraints.len());

    true
}

/// Handles displaying the context or filtered parts of it.
pub fn handle_show(ctx: &ProgramContext, filter: &Option<String>) {
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

/// Handles proposing top scoring tracklist permutations for a tracklist & medium.
pub fn handle_propose(
    ctx: &ProgramContext,
    tracklist_name: &str,
    medium_name: &str,
    count: &usize,
    min_score: &Option<usize>,
) {
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
            "{}",
            format!(
                "Top {} permutations for tracklist '{}' on medium '{}' with score >= {}:",
                count, tracklist_name, medium_name, min
            )
            .bold()
            .cyan()
        );
    } else {
        println!(
            "{}",
            format!(
                "Top {} permutations for tracklist '{}' on medium '{}':",
                count, tracklist_name, medium_name
            )
            .bold()
            .cyan()
        );
    }

    for (idx, (score, tl)) in scored_perms.into_iter().take(*count).enumerate() {
        println!(
            "{} {}",
            "Permutation".yellow().bold(),
            format!("#{}", idx + 1).yellow().bold()
        );
        println!("{} {}", "Score:".green().bold(), score.to_string().green().bold());

        let max_title_len = tl
            .0
            .iter()
            .map(|t| t.title.len())
            .max()
            .unwrap_or(5)
            .max("Title".len());

        // Table header
        println!(
            "{:<3} {:<width$} {:>8}",
            "#",
            "Title".bold(),
            "Duration".bold(),
            width = max_title_len
        );
        println!(
            "{:-<3} {:-<width$} {:-<8}",
            "",
            "",
            "",
            width = max_title_len
        );

        let sides = split_tracklist_by_side(&tl, &medium);

        let mut track_idx = 1;
        for (side_idx, side_tracks) in sides.iter().enumerate() {
            println!(
                "{} {}",
                "Side".blue().bold(),
                format!("{}", side_idx + 1).blue().bold()
            );
            for t in side_tracks {
                println!(
                    "{:<3} {:<width$} {:>8}",
                    track_idx,
                    t.title.clone(),
                    format_duration(t.duration),
                    width = max_title_len
                );
                track_idx += 1;
            }
        }

        let total_duration: Duration = tl.0.iter().map(|t| t.duration).sum();

        println!(
            "{:<3} {:<width$} {:>8}",
            "",
            "TOTAL".bold(),
            format_duration(total_duration).bold(),
            width = max_title_len
        );
        println!();
    }
}
