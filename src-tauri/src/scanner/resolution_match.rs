/// 4K / UHD from indexed stream dimensions only (not library folder layout).
/// Letterboxed UHD often reports height below 2160 while width stays 3840.
pub const SQL_IS_FOUR_K: &str = r#"(
    COALESCE(m.resolution_height, 0) >= 2160
    OR COALESCE(m.resolution_width, 0) >= 3840
)"#;

/// 1080p tier from stream dimensions; excludes titles classified as 4K above.
pub const SQL_IS_1080P: &str = r#"(
    NOT (
        COALESCE(m.resolution_height, 0) >= 2160
        OR COALESCE(m.resolution_width, 0) >= 3840
    )
    AND (
        (COALESCE(m.resolution_width, 0) >= 1920 AND COALESCE(m.resolution_width, 0) < 3840)
        OR (
            COALESCE(m.resolution_height, 0) >= 1080
            AND COALESCE(m.resolution_height, 0) < 2160
        )
    )
)"#;