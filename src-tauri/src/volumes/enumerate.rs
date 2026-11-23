#[cfg(target_os = "windows")]
mod platform {
    use super::super::types::{format_serial, normalize_drive_letter, EnumeratedVolume};
    use windows::core::{Error, PCWSTR, HRESULT};
    use windows::Win32::Foundation::{
        ERROR_INVALID_FUNCTION, ERROR_NO_MORE_FILES, HANDLE, MAX_PATH,
    };
    use windows::Win32::Storage::FileSystem::{
        FindFirstVolumeW, FindNextVolumeW, FindVolumeClose, GetVolumeInformationW,
        GetVolumePathNamesForVolumeNameW,
    };

    const CCH_VOLUME: usize = (MAX_PATH + 1) as usize;

    fn volume_guid(volume_path: &str) -> Option<String> {
        let open = volume_path.find('{')?;
        let close = volume_path.find('}')?;
        Some(volume_path[open + 1..close].to_ascii_uppercase())
    }

    unsafe fn wide_nul_to_string(buf: &[u16]) -> Result<String, Error> {
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        if end == 0 {
            return Ok(String::new());
        }
        String::from_utf16(&buf[..end]).map_err(|_| Error::from_hresult(HRESULT(-1)))
    }

    fn parse_multi_sz(buf: &[u16]) -> Result<Vec<String>, Error> {
        let mut out = Vec::new();
        let mut start = 0usize;
        for (i, &ch) in buf.iter().enumerate() {
            if ch == 0 {
                if i > start {
                    out.push(String::from_utf16(&buf[start..i]).map_err(|_| {
                        Error::from_hresult(HRESULT(-1))
                    })?);
                }
                start = i + 1;
                if i + 1 >= buf.len() || buf[i + 1] == 0 {
                    break;
                }
            }
        }
        Ok(out)
    }

    unsafe fn volume_mount_paths(volume_name: PCWSTR) -> Result<Vec<String>, Error> {
        unsafe {
            let mut cch: u32 = 0;
            let _ = GetVolumePathNamesForVolumeNameW(volume_name, None, &mut cch);
            if cch == 0 {
                return Ok(vec![]);
            }
            let mut buf = vec![0u16; cch as usize];
            GetVolumePathNamesForVolumeNameW(volume_name, Some(&mut buf), &mut cch)?;
            parse_multi_sz(&buf)
        }
    }

    fn primary_drive_letter(paths: &[String]) -> String {
        for path in paths {
            if let Some(ch) = path.chars().next() {
                if ch.is_ascii_alphabetic() {
                    return normalize_drive_letter(&format!("{ch}:"));
                }
            }
        }
        String::new()
    }

    pub fn enumerate_connected_volumes() -> Result<Vec<EnumeratedVolume>, String> {
        unsafe {
            let mut vol_buf = vec![0u16; CCH_VOLUME];
            let find = FindFirstVolumeW(&mut vol_buf)
                .map_err(|e| format!("FindFirstVolumeW failed: {e}"))?;

            let mut volumes = Vec::new();
            loop {
                let volume_pc = PCWSTR::from_raw(vol_buf.as_ptr());
                let volume_path = volume_pc
                    .to_string()
                    .map_err(|e| format!("Failed to read volume path: {e}"))?;

                let guid = match volume_guid(&volume_path) {
                    Some(value) => value,
                    None => {
                        if advance(find, &mut vol_buf)? {
                            break;
                        }
                        continue;
                    }
                };

                let mount_paths = volume_mount_paths(volume_pc)
                    .map_err(|e| format!("GetVolumePathNamesForVolumeNameW failed: {e}"))?;
                let drive_letter = primary_drive_letter(&mount_paths);

                if drive_letter.is_empty() {
                    if advance(find, &mut vol_buf)? {
                        break;
                    }
                    continue;
                }

                let root = windows::core::HSTRING::from(format!("{drive_letter}\\"));
                let mut label = vec![0u16; MAX_PATH as usize + 1];
                let mut filesystem = vec![0u16; MAX_PATH as usize + 1];
                let mut serial = 0u32;

                match GetVolumeInformationW(
                    PCWSTR::from_raw(root.as_ptr()),
                    Some(&mut label),
                    Some(&mut serial),
                    None,
                    None,
                    Some(&mut filesystem),
                ) {
                    Ok(()) => {
                        let label = wide_nul_to_string(&label)
                            .map_err(|e| format!("Volume label decode failed: {e}"))?;
                        let filesystem = wide_nul_to_string(&filesystem)
                            .map_err(|e| format!("Filesystem decode failed: {e}"))?;

                        volumes.push(EnumeratedVolume {
                            serial: format_serial(serial),
                            guid,
                            label,
                            filesystem,
                            drive_letter,
                        });
                    }
                    Err(e) => {
                        let code = e.code().0 as u32;
                        if code != ERROR_INVALID_FUNCTION.0 {
                            eprintln!("Skipping volume {guid}: {e}");
                        }
                    }
                }

                if advance(find, &mut vol_buf)? {
                    break;
                }
            }

            FindVolumeClose(find).map_err(|e| format!("FindVolumeClose failed: {e}"))?;
            volumes.sort_by(|a, b| a.drive_letter.cmp(&b.drive_letter));
            Ok(volumes)
        }
    }

    fn is_no_more_files(error: &Error) -> bool {
        // HRESULT_FROM_WIN32(ERROR_NO_MORE_FILES) is 0x80070012 — not the raw 18u32.
        let code = error.code();
        code.0 as u32 == (0x80070000 | ERROR_NO_MORE_FILES.0)
            || (code.0 as u32 & 0xFFFF) == ERROR_NO_MORE_FILES.0
    }

    unsafe fn advance(find: HANDLE, vol_buf: &mut [u16]) -> Result<bool, String> {
        unsafe {
            match FindNextVolumeW(find, vol_buf) {
                Ok(()) => Ok(false),
                Err(e) if is_no_more_files(&e) => Ok(true),
                Err(e) => Err(format!("FindNextVolumeW failed: {e}")),
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub use platform::enumerate_connected_volumes;

#[cfg(not(target_os = "windows"))]
pub fn enumerate_connected_volumes() -> Result<Vec<super::types::EnumeratedVolume>, String> {
    Err("Volume enumeration is only supported on Windows in this MVP.".to_string())
}