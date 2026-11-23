use super::types::{EnumeratedVolume, RemapCandidate, VolumeRecord};
use rusqlite::{params, Connection, OptionalExtension};

pub struct SyncOutput {
    pub volumes: Vec<VolumeRecord>,
    pub remap_candidates: Vec<RemapCandidate>,
}

pub fn sync_enumerated_volumes(
    conn: &Connection,
    connected: &[EnumeratedVolume],
) -> Result<SyncOutput, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to start volume sync transaction: {e}"))?;

    let (connected_ids, remap_candidates) = sync_connected(&tx, connected)?;
    mark_disconnected(&tx, &connected_ids)?;

    tx.commit()
        .map_err(|e| format!("Failed to commit volume sync: {e}"))?;

    let volumes = list_volume_records(conn)?;

    Ok(SyncOutput {
        volumes,
        remap_candidates,
    })
}

fn sync_connected(
    conn: &Connection,
    connected: &[EnumeratedVolume],
) -> Result<(Vec<i64>, Vec<RemapCandidate>), String> {
    let mut ids = Vec::with_capacity(connected.len());
    let mut remap_candidates = Vec::new();

    for volume in connected {
        let existing: Option<(i64, Option<String>, i64)> = conn
            .query_row(
                "SELECT id, last_drive_letter, auto_remap FROM volumes WHERE serial = ?1 AND guid = ?2",
                params![volume.serial, volume.guid],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|e| format!("Volume lookup failed: {e}"))?;

        let auto_remap = existing.as_ref().map(|(_, _, flag)| *flag == 1).unwrap_or(false);
        let previous = existing
            .as_ref()
            .and_then(|(_, letter, _)| letter.clone())
            .filter(|value| !value.is_empty());

        let letter_changed = previous
            .as_ref()
            .is_some_and(|prev| prev != &volume.drive_letter);

        let drive_letter_to_store = if existing.is_none() || auto_remap || !letter_changed {
            Some(volume.drive_letter.as_str())
        } else {
            None
        };

        if let Some((id, Some(previous), _)) = existing.as_ref() {
            if letter_changed && !auto_remap && !previous.is_empty() {
                remap_candidates.push(RemapCandidate {
                    volume_id: *id,
                    serial: volume.serial.clone(),
                    guid: volume.guid.clone(),
                    label: volume.label.clone(),
                    previous_drive_letter: previous.clone(),
                    current_drive_letter: volume.drive_letter.clone(),
                });
            }
        }

        let volume_id = if let Some((id, _, _)) = existing {
            if let Some(letter) = drive_letter_to_store {
                conn.execute(
                    "UPDATE volumes SET
                        label = ?1,
                        filesystem = ?2,
                        last_drive_letter = ?3,
                        last_seen = datetime('now'),
                        updated_at = datetime('now'),
                        last_remap_at = CASE WHEN ?4 = 1 THEN datetime('now') ELSE last_remap_at END
                     WHERE id = ?5",
                    params![
                        volume.label,
                        volume.filesystem,
                        letter,
                        if letter_changed && auto_remap { 1 } else { 0 },
                        id
                    ],
                )
                .map_err(|e| format!("Failed to update volume {id}: {e}"))?;
            } else {
                conn.execute(
                    "UPDATE volumes SET
                        label = ?1,
                        filesystem = ?2,
                        last_seen = datetime('now'),
                        updated_at = datetime('now')
                     WHERE id = ?3",
                    params![volume.label, volume.filesystem, id],
                )
                .map_err(|e| format!("Failed to refresh volume {id}: {e}"))?;
            }
            id
        } else {
            conn.execute(
                "INSERT INTO volumes (serial, guid, label, filesystem, last_drive_letter, last_seen)
                 VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
                params![
                    volume.serial,
                    volume.guid,
                    volume.label,
                    volume.filesystem,
                    volume.drive_letter
                ],
            )
            .map_err(|e| format!("Failed to insert volume: {e}"))?;
            conn.last_insert_rowid()
        };

        ids.push(volume_id);
    }

    Ok((ids, remap_candidates))
}

fn mark_disconnected(conn: &Connection, connected_ids: &[i64]) -> Result<(), String> {
    if connected_ids.is_empty() {
        return Ok(());
    }

    let placeholders = connected_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "UPDATE volumes SET last_seen = NULL, updated_at = datetime('now')
         WHERE id NOT IN ({placeholders})"
    );

    let params: Vec<&dyn rusqlite::types::ToSql> = connected_ids
        .iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();

    conn.execute(&sql, params.as_slice())
        .map_err(|e| format!("Failed to mark disconnected volumes: {e}"))?;
    Ok(())
}

pub fn list_volume_records(conn: &Connection) -> Result<Vec<VolumeRecord>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, serial, guid, label, filesystem, last_drive_letter, auto_remap, last_seen
             FROM volumes
             ORDER BY COALESCE(last_drive_letter, ''), label",
        )
        .map_err(|e| format!("Failed to prepare volume list: {e}"))?;

    let online: std::collections::HashSet<i64> = online_volume_ids(conn)
        .unwrap_or_default()
        .into_iter()
        .collect();

    let rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            Ok(VolumeRecord {
                id,
                serial: row.get(1)?,
                guid: row.get(2)?,
                label: row.get(3)?,
                filesystem: row.get(4)?,
                last_drive_letter: row.get(5)?,
                auto_remap: row.get::<_, i64>(6)? == 1,
                is_online: online.contains(&id),
            })
        })
        .map_err(|e| format!("Failed to query volumes: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read volumes: {e}"))
}

pub fn apply_remap(
    conn: &Connection,
    volume_id: i64,
    new_drive_letter: &str,
    auto_remap: bool,
) -> Result<(), String> {
    let rows = conn
        .execute(
            "UPDATE volumes SET
                last_drive_letter = ?1,
                auto_remap = ?2,
                last_remap_at = datetime('now'),
                last_seen = datetime('now'),
                updated_at = datetime('now')
             WHERE id = ?3",
            params![new_drive_letter, if auto_remap { 1 } else { 0 }, volume_id],
        )
        .map_err(|e| format!("Failed to apply remap: {e}"))?;

    if rows == 0 {
        return Err(format!("Volume {volume_id} not found"));
    }

    Ok(())
}

pub fn online_volume_ids(conn: &Connection) -> Result<Vec<i64>, String> {
    let connected = super::enumerate::enumerate_connected_volumes()?;
    let mut ids = Vec::new();

    for volume in connected {
        let id: Option<i64> = conn
            .query_row(
                "SELECT id FROM volumes WHERE serial = ?1 AND guid = ?2",
                params![volume.serial, volume.guid],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("Online lookup failed: {e}"))?;
        if let Some(id) = id {
            ids.push(id);
        }
    }

    Ok(ids)
}