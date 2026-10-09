//! What a sync would put on a stick, worked out without writing anything.
//!
//! The answer comes from the same record the sync itself trusts: a track the
//! manifest says is on the stick, from a source whose size and modification
//! time have not changed, is not copied again. Nothing is hashed here, so an
//! estimate stays quick on a large selection; a sync that finds the bytes
//! differ copies more than this says.

use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use crate::manifest::{track_key, Manifest};
use crate::{
    in_place_paths, owns_library_file, previous_in_place, source_stamp, under,
    CompatibilityFormat, SourceTrack,
};

/// The space a sync would use and give back on one stick.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Estimate {
    /// Source bytes of the tracks that would be copied: new ones, and ones
    /// whose source changed since they were written.
    pub copy_bytes: u64,
    /// Bytes of the tracks the stick already holds and the sync leaves alone.
    pub reuse_bytes: u64,
    /// Bytes of audio on the stick that the sync would remove.
    pub free_bytes: u64,
    pub tracks_new: usize,
    pub tracks_changed: usize,
    pub tracks_kept: usize,
    pub tracks_removed: usize,
    /// Selected tracks whose audio file could not be found.
    pub tracks_missing: usize,
    /// A copied track would be converted, so the bytes it takes on the stick
    /// are not known until it has been.
    pub approximate: bool,
}

/// What syncing `tracks` to `destination` would cost, given the stick's
/// manifest (ignored when it belongs to a library other than `db_id`).
pub fn estimate(
    destination: &Path,
    db_id: u64,
    tracks: &[SourceTrack],
    delete_unlisted_music: bool,
    compatibility: Option<CompatibilityFormat>,
) -> Estimate {
    let previous = Manifest::load(destination).filter(|m| m.db_id == db_id);
    let previous_place = previous_in_place(destination, previous.as_ref());
    let on_stick = in_place_paths(destination, tracks, &previous_place);
    let mut result = Estimate::default();
    let mut seen = HashSet::new();

    for (index, track) in tracks.iter().enumerate() {
        let key = track_key(track.id, &track.source_path.to_string_lossy());
        seen.insert(key.clone());
        let Ok(meta) = std::fs::metadata(&track.source_path) else {
            result.tracks_missing += 1;
            continue;
        };
        let (size, modified) = source_stamp(&meta);
        // The library's own file, already where the stick needs it.
        if on_stick.get(index).is_some_and(Option::is_some) {
            result.tracks_kept += 1;
            continue;
        }
        let conversion = compatibility
            .filter(|_| !track.device.as_ref().is_some_and(|d| d.preserve))
            .map(CompatibilityFormat::audio)
            .filter(|_| rbl_audio::compatibility::needs_conversion(&track.source_path).unwrap_or(false));
        let profile = conversion.map_or("", rbl_audio::compatibility::Format::profile);
        let carried = previous.as_ref().and_then(|m| m.tracks.iter().find(|t| t.key() == key));
        let unchanged = carried.is_some_and(|c| {
            !c.in_place && c.size == size && c.modified == modified && c.conversion == profile
                && under(destination, &c.audio).is_file()
        });
        if unchanged {
            result.tracks_kept += 1;
            result.reuse_bytes += size;
            continue;
        }
        if carried.is_some() {
            result.tracks_changed += 1;
        } else {
            result.tracks_new += 1;
        }
        result.copy_bytes += size;
        result.approximate |= conversion.is_some();
    }

    if let Some(manifest) = previous.as_ref() {
        let loose: BTreeSet<u64> = manifest.loose.iter().copied().collect();
        for entry in manifest.tracks.iter().filter(|t| !seen.contains(&t.key())) {
            // The library's own files are never the sync's to delete, and a
            // track put on the stick on its own stays unless cleanup is on.
            if owns_library_file(destination, entry) { continue; }
            if !delete_unlisted_music && loose.contains(&entry.library_id) { continue; }
            result.tracks_removed += 1;
            let on_disk = std::fs::metadata(under(destination, &entry.audio)).map_or(entry.size, |m| m.len());
            result.free_bytes += on_disk;
        }
    }
    result
}
