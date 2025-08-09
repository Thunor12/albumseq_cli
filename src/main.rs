use albumseq::{
    Constraint as AlbumConstraint, ConstraintKind as AlbumConstraintKind, Duration,
    Medium as AlbumMedium, Track, Tracklist, TracklistPermutations, score_tracklist,
};
use clap::{Parser, Subcommand};

use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const DEFAULT_CONTEXT_PATH: &str = "context.json";

/// Serializable Track for saving/loading
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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

impl From<&SerTrack> for Track {
    fn from(st: &SerTrack) -> Self {
        Track {
            title: st.title.clone(),
            duration: st.duration,
        }
    }
}

/// Serializable Tracklist wrapper
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SerTracklist(pub Vec<SerTrack>);

impl From<&Tracklist> for SerTracklist {
    fn from(tl: &Tracklist) -> Self {
        SerTracklist(tl.0.iter().map(|t| t.into()).collect())
    }
}

impl From<&SerTracklist> for Tracklist {
    fn from(stl: &SerTracklist) -> Self {
        Tracklist(stl.0.iter().map(|st| st.into()).collect())
    }
}

/// Named Tracklist with a name key
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NamedSerTracklist {
    pub name: String,
    pub tracks: SerTracklist,
}

/// Serializable Medium with name for identification
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SerMedium {
    pub name: String,
    pub sides: usize,
    pub max_duration_per_side: Duration,
}

impl SerMedium {
    /// Convert to library Medium (without name)
    pub fn to_album_medium(&self) -> AlbumMedium {
        AlbumMedium {
            sides: self.sides,
            max_duration_per_side: self.max_duration_per_side,
            name: self.name.clone(),
        }
    }
}

/// Serializable constraint kinds for saving/loading
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", content = "data")]
pub enum SerConstraintKind {
    AtPosition(String, usize),
    Adjacent(String, String),
    OnSameSide(String, String),
}

/// Serializable constraint with kind and weight
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerConstraint {
    pub kind: SerConstraintKind,
    pub weight: usize,
}

/// Convert from SerConstraint to albumseq Constraint
impl From<SerConstraint> for AlbumConstraint {
    fn from(c: SerConstraint) -> Self {
        let kind = match c.kind {
            SerConstraintKind::AtPosition(title, pos) => {
                AlbumConstraintKind::AtPosition(title, pos)
            }
            SerConstraintKind::Adjacent(t1, t2) => AlbumConstraintKind::Adjacent(t1, t2),
            SerConstraintKind::OnSameSide(t1, t2) => AlbumConstraintKind::OnSameSide(t1, t2),
        };
        AlbumConstraint {
            kind,
            weight: c.weight,
        }
    }
}

/// Convert from albumseq Constraint to SerConstraint
impl From<&AlbumConstraint> for SerConstraint {
    fn from(c: &AlbumConstraint) -> Self {
        let kind = match &c.kind {
            AlbumConstraintKind::AtPosition(title, pos) => {
                SerConstraintKind::AtPosition(title.clone(), *pos)
            }
            AlbumConstraintKind::Adjacent(t1, t2) => {
                SerConstraintKind::Adjacent(t1.clone(), t2.clone())
            }
            AlbumConstraintKind::OnSameSide(t1, t2) => {
                SerConstraintKind::OnSameSide(t1.clone(), t2.clone())
            }
        };
        SerConstraint {
            kind,
            weight: c.weight,
        }
    }
}

/// The main program context holding tracklists, mediums, and constraints
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ProgramContext {
    pub tracklists: Vec<NamedSerTracklist>,
    pub mediums: Vec<SerMedium>,
    pub constraints: Vec<SerConstraint>,
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

    /// Add or replace a medium by name
    fn add_or_replace_medium(
        &mut self,
        name: String,
        sides: usize,
        max_duration_per_side: Duration,
    ) {
        let new_medium = SerMedium {
            name: name.clone(),
            sides,
            max_duration_per_side,
        };

        if let Some(existing) = self
            .mediums
            .iter_mut()
            .find(|m| m.name.eq_ignore_ascii_case(&name))
        {
            *existing = new_medium;
            println!("Replaced medium '{}'", name);
        } else {
            self.mediums.push(new_medium);
            println!("Added medium '{}'", name);
        }
    }

    /// Add or replace a constraint
    fn add_or_replace_constraint(&mut self, constraint: AlbumConstraint) {
        let ser_constraint = SerConstraint::from(&constraint);
        let kind = ser_constraint.kind;

        if let Some(existing) = self.constraints.iter_mut().find(|c| c.kind == kind) {
            *existing = SerConstraint::from(&constraint);
            println!("Replaced constraint {:?}", kind);
        } else {
            self.constraints.push(SerConstraint::from(&constraint));
            println!("Added constraint {:?}", kind);
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
    Show,
    /// Propose top scoring tracklist permutations for a tracklist & medium
    Propose {
        /// Tracklist name to use
        #[arg(short, long)]
        tracklist: String,

        /// Medium name to use
        #[arg(short, long)]
        medium: String,

        /// Number of propositions to show
        #[arg(short, long, default_value = "5")]
        count: usize,
    },
}

fn parse_duration(s: &str) -> Option<f64> {
    if let Some((min_str, sec_str)) = s.split_once(':') {
        // MM:SS format
        if let (Ok(min), Ok(sec)) = (min_str.parse::<u32>(), sec_str.parse::<u32>()) {
            return Some(min as f64 + sec as f64 / 60.0);
        }
    }
    // fallback: decimal minutes
    s.parse::<f64>().ok()
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

            let constraint_kind_opt = match kind.to_lowercase().as_str() {
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
                        eprintln!(
                            "Adjacent constraint requires exactly 2 arguments: title1 title2"
                        );
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
                        eprintln!(
                            "OnSameSide constraint requires exactly 2 arguments: title1 title2"
                        );
                        None
                    }
                }
                _ => {
                    eprintln!("Unknown constraint kind: {}", kind);
                    None
                }
            };

            if let Some(kind) = constraint_kind_opt {
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

        Commands::Show => {
            let ctx = ProgramContext::load_or_create(&cli.context);

            if ctx.tracklists.is_empty() && ctx.mediums.is_empty() && ctx.constraints.is_empty() {
                println!("Context is empty.");
                return;
            }

            if !ctx.tracklists.is_empty() {
                println!("=== Tracklists ===");
                for tl in &ctx.tracklists {
                    println!("-- {} --", tl.name);
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

            if !ctx.mediums.is_empty() {
                println!("=== Mediums ===");
                for m in &ctx.mediums {
                    println!(
                        "{}: {} sides, max duration per side: {}",
                        m.name,
                        m.sides,
                        format_duration(m.max_duration_per_side)
                    );
                }
                println!();
            }

            if !ctx.constraints.is_empty() {
                println!("=== Constraints ===");
                for c in &ctx.constraints {
                    println!("{:?} (weight {})", c.kind, c.weight);
                }
                println!();
            }
        }

        Commands::Propose {
            tracklist: tracklist_name,
            medium: medium_name,
            count,
        } => {
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

            // Score permutations and keep top `count` by descending score
            let mut scored_perms: Vec<(usize, Tracklist)> = perms
                .map(|perm| {
                    let tl = Tracklist(perm.into_iter().cloned().collect());
                    let score = score_tracklist(&tl, &constraints, &medium);
                    (score, tl)
                })
                .filter(|(_, tl)| medium.fits(tl))
                .collect();

            scored_perms.sort_by(|a, b| b.0.cmp(&a.0)); // descending by score

            println!(
                "Top {} permutations for tracklist '{}' on medium '{}':",
                count, tracklist_name, medium_name
            );

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
    }
}
