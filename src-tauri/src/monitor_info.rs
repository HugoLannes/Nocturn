use std::collections::HashMap;
use std::mem::{size_of, zeroed};
use std::ptr;

use log::warn;
use windows_sys::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ONLY_ACTIVE_PATHS,
};
use windows_sys::Win32::Graphics::Gdi::{EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS};

/// Resolved monitor identity: manufacturer name and model extracted separately.
#[derive(Clone, Debug, Default)]
pub struct MonitorIdentity {
    pub manufacturer: String,
    pub model: String,
}

/// Query refresh rate and orientation for a GDI device name (e.g. `\\.\DISPLAY1`).
///
/// Returns `(refresh_rate_hz, orientation)` where orientation is:
/// - 0 = landscape (default)
/// - 1 = portrait (90°)
/// - 2 = landscape flipped (180°)
/// - 3 = portrait flipped (270°)
pub fn query_display_settings(device_name: &str) -> (u32, u32) {
    let wide: Vec<u16> = device_name.encode_utf16().chain(Some(0)).collect();

    unsafe {
        let mut dm: DEVMODEW = zeroed();
        dm.dmSize = size_of::<DEVMODEW>() as u16;

        if EnumDisplaySettingsW(wide.as_ptr(), ENUM_CURRENT_SETTINGS, &mut dm) == 0 {
            warn!("EnumDisplaySettingsW failed for {}", device_name);
            return (0, 0);
        }

        let refresh_rate = dm.dmDisplayFrequency;
        let orientation = dm.Anonymous1.Anonymous2.dmDisplayOrientation;

        (refresh_rate, orientation)
    }
}

/// Query monitor identities via the CCD API.
///
/// Returns a map from GDI device name (e.g. `\\.\DISPLAY1`) to a `MonitorIdentity`
/// with the manufacturer name decoded from the EDID PNP ID, and the model from the
/// friendly name (with the manufacturer prefix stripped if present).
pub fn query_monitor_identities() -> HashMap<String, MonitorIdentity> {
    unsafe {
        let mut path_count: u32 = 0;
        let mut mode_count: u32 = 0;

        let result =
            GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count);

        if result != 0 {
            warn!("GetDisplayConfigBufferSizes failed with error {}", result);
            return HashMap::new();
        }

        let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = vec![zeroed(); path_count as usize];
        let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = vec![zeroed(); mode_count as usize];

        let result = QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            ptr::null_mut(),
        );

        if result != 0 {
            warn!("QueryDisplayConfig failed with error {}", result);
            return HashMap::new();
        }

        paths.truncate(path_count as usize);

        let mut map = HashMap::new();

        for path in &paths {
            // Get GDI device name for this source
            let mut source: DISPLAYCONFIG_SOURCE_DEVICE_NAME = zeroed();
            source.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
            source.header.size = size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
            source.header.adapterId = path.sourceInfo.adapterId;
            source.header.id = path.sourceInfo.id;

            if DisplayConfigGetDeviceInfo(
                &mut source as *mut _ as *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
            ) != 0
            {
                continue;
            }

            // Get target device name (includes friendly name + EDID manufacturer ID)
            let mut target: DISPLAYCONFIG_TARGET_DEVICE_NAME = zeroed();
            target.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
            target.header.size = size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
            target.header.adapterId = path.targetInfo.adapterId;
            target.header.id = path.targetInfo.id;

            if DisplayConfigGetDeviceInfo(
                &mut target as *mut _ as *mut DISPLAYCONFIG_DEVICE_INFO_HEADER,
            ) != 0
            {
                continue;
            }

            let gdi_name = wide_to_string(&source.viewGdiDeviceName);
            let friendly_name = wide_to_string(&target.monitorFriendlyDeviceName);

            // Decode manufacturer from EDID PNP ID (always present)
            let pnp_code = decode_pnp_id(target.edidManufactureId);
            let manufacturer = resolve_manufacturer(&pnp_code);

            // Model = friendly name with manufacturer prefix stripped
            let model = extract_model(&friendly_name, &manufacturer, &pnp_code);

            map.insert(
                gdi_name,
                MonitorIdentity {
                    manufacturer,
                    model,
                },
            );
        }

        map
    }
}

/// Decode a 16-bit EDID manufacturer ID into a 3-letter PNP code.
///
/// The ID encodes three 5-bit characters (A=1, B=2, ..., Z=26) packed into
/// a big-endian 16-bit value: `0bXaaaa_abbb_bbcc_ccc0` where a/b/c are the letters.
fn decode_pnp_id(raw: u16) -> String {
    // EDID stores this big-endian but Windows may have already swapped it.
    // The standard encoding: bit 14-10 = first, 9-5 = second, 4-0 = third.
    // Windows provides it in the native byte order of the EDID block (big-endian),
    // so we need to swap bytes first.
    let swapped = raw.swap_bytes();
    let c1 = ((swapped >> 10) & 0x1F) as u8;
    let c2 = ((swapped >> 5) & 0x1F) as u8;
    let c3 = (swapped & 0x1F) as u8;

    let to_char = |v: u8| -> char {
        if v >= 1 && v <= 26 {
            (b'A' + v - 1) as char
        } else {
            '?'
        }
    };

    format!("{}{}{}", to_char(c1), to_char(c2), to_char(c3))
}

/// Resolve a 3-letter PNP code to a human-readable manufacturer name.
fn resolve_manufacturer(pnp: &str) -> String {
    match pnp {
        "ACI" => "ASUS",
        "ACR" => "Acer",
        "AOC" => "AOC",
        "AUS" => "ASUS",
        "BNQ" => "BenQ",
        "BOE" => "BOE",
        "CMN" => "Chimei Innolux",
        "DEL" => "Dell",
        "ENC" => "EIZO",
        "EIZ" => "EIZO",
        "GBT" => "Gigabyte",
        "GSM" => "LG",
        "HPN" => "HP",
        "HWP" => "HP",
        "IVM" => "iiyama",
        "LEN" => "Lenovo",
        "LGD" => "LG Display",
        "MEI" => "Panasonic",
        "MSI" => "MSI",
        "NEC" => "NEC",
        "PHL" => "Philips",
        "SAM" => "Samsung",
        "SDC" => "Samsung Display",
        "SEC" => "Samsung",
        "SHP" => "Sharp",
        "SNY" => "Sony",
        "VSC" => "ViewSonic",
        _ => return String::new(),
    }
    .to_string()
}

/// Extract the model name from the EDID friendly name.
///
/// If the friendly name starts with the manufacturer name or PNP code, strip it.
/// Otherwise return the whole friendly name as the model.
fn extract_model(friendly: &str, manufacturer: &str, pnp: &str) -> String {
    let trimmed = friendly.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Try stripping the manufacturer name prefix (e.g. "DELL P2317H" -> "P2317H")
    if !manufacturer.is_empty() {
        let upper = trimmed.to_uppercase();
        let mfr_upper = manufacturer.to_uppercase();
        if upper.starts_with(&mfr_upper) {
            let rest = trimmed[manufacturer.len()..].trim_start();
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }

    // Try stripping the PNP code prefix (e.g. "AOC 24G1WG4" -> "24G1WG4")
    {
        let upper = trimmed.to_uppercase();
        let pnp_upper = pnp.to_uppercase();
        if upper.starts_with(&pnp_upper) {
            let rest = trimmed[pnp.len()..].trim_start();
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }

    // No known prefix — return the whole string as model
    trimmed.to_string()
}

fn wide_to_string(wide: &[u16]) -> String {
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..len])
}
