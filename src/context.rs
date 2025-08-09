use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use albumseq::{
    Constraint as AlbumConstraint, ConstraintKind as AlbumConstraintKind, Duration,
    Medium as AlbumMedium, Track, Tracklist,
};

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

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NamedSerTracklist {
    pub name: String,
    pub tracks: SerTracklist,
}

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", content = "data")]
pub enum SerConstraintKind {
    AtPosition(String, usize),
    Adjacent(String, String),
    OnSameSide(String, String),
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgramContext {
    pub tracklists: Vec<NamedSerTracklist>,
    pub mediums: Vec<SerMedium>,
    pub constraints: Vec<SerConstraint>,
}

impl ProgramContext {
    pub fn load_or_create(path: &Path) -> Self {
        if path.exists() {
            let data = fs::read_to_string(path).expect("Failed to read context file");
            serde_json::from_str(&data).expect("Failed to parse context file")
        } else {
            let ctx = Self::default();
            ctx.save(path);
            ctx
        }
    }

    pub fn save(&self, path: &Path) {
        let json = serde_json::to_string_pretty(self).expect("Failed to serialize context");
        fs::write(path, json).expect("Failed to write context file");
    }

    /// Add or replace a tracklist by name
    pub fn add_or_replace_tracklist(&mut self, name: String, tracks: Vec<Track>) {
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
    pub fn add_or_replace_medium(
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
    pub fn add_or_replace_constraint(&mut self, constraint: AlbumConstraint) {
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

impl Default for ProgramContext {
    fn default() -> Self {
        let path = Path::new("context.json");
        Self::load_or_create(path)
    }
}
