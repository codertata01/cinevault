pub mod repository;
mod enumerate;
mod types;

pub use repository::sync_enumerated_volumes;
pub use types::{normalize_drive_letter, VolumeRecord, VolumeSyncResult};

use enumerate::enumerate_connected_volumes;

pub fn sync_volumes(conn: &rusqlite::Connection) -> Result<VolumeSyncResult, String> {
    let connected = enumerate_connected_volumes()?;
    let output = sync_enumerated_volumes(conn, &connected)?;
    Ok(VolumeSyncResult {
        volumes: output.volumes,
        remap_candidates: output.remap_candidates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_migrations;
    use rusqlite::Connection;

    #[test]
    fn sync_inserts_and_updates_volumes() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let first = vec![types::EnumeratedVolume {
            serial: "A1B2C3D4".to_string(),
            guid: "11111111-2222-3333-4444-555555555555".to_string(),
            label: "Movies A".to_string(),
            filesystem: "NTFS".to_string(),
            drive_letter: "E:".to_string(),
        }];

        repository::sync_enumerated_volumes(&conn, &first).unwrap();
        let volumes = repository::list_volume_records(&conn).unwrap();
        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes[0].last_drive_letter.as_deref(), Some("E:"));

        let moved = vec![types::EnumeratedVolume {
            drive_letter: "G:".to_string(),
            ..first[0].clone()
        }];

        let sync = repository::sync_enumerated_volumes(&conn, &moved).unwrap();
        assert_eq!(sync.remap_candidates.len(), 1);
        assert_eq!(
            sync.remap_candidates[0].current_drive_letter,
            "G:"
        );

        repository::apply_remap(&conn, volumes[0].id, "G:", true).unwrap();
        let after = repository::list_volume_records(&conn).unwrap();
        assert_eq!(after[0].last_drive_letter.as_deref(), Some("G:"));
        assert!(after[0].auto_remap);
    }
}