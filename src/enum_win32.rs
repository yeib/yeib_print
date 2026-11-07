//! Enumeración nativa de impresoras en Windows mediante Win32 (`Winspool`).

use crate::types::{PrinterInfo, PrinterStatus, Result};

#[cfg(target_os = "windows")]
use windows::{
    core::PWSTR,
    Win32::Graphics::Printing::{
        EnumPrintersW, GetDefaultPrinterW, PRINTER_ATTRIBUTE_WORK_OFFLINE, PRINTER_ENUM_CONNECTIONS,
        PRINTER_ENUM_LOCAL, PRINTER_INFO_2W, PRINTER_STATUS_BUSY, PRINTER_STATUS_ERROR,
        PRINTER_STATUS_OFFLINE, PRINTER_STATUS_PAPER_JAM, PRINTER_STATUS_PAPER_OUT,
        PRINTER_STATUS_PAUSED,
    },
};

#[cfg(target_os = "windows")]
unsafe fn pwstr_to_string(ptr: PWSTR) -> String {
    if ptr.is_null() {
        return String::new();
    }
    ptr.to_string().unwrap_or_default()
}

#[cfg(target_os = "windows")]
pub fn get_default_printer_name() -> Option<String> {
    unsafe {
        let mut len: u32 = 0;
        let _ = GetDefaultPrinterW(None, &mut len);
        if len == 0 {
            return None;
        }
        let mut buf = vec![0u16; len as usize];
        if GetDefaultPrinterW(Some(PWSTR(buf.as_mut_ptr())), &mut len).as_bool() {
            let s = String::from_utf16_lossy(&buf);
            let clean = s.trim_end_matches('\0').trim().to_string();
            if !clean.is_empty() {
                return Some(clean);
            }
        }
        None
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_default_printer_name() -> Option<String> {
    None
}

fn is_virtual_printer(name: &str, driver: &str, port: &str) -> bool {
    let combined = format!(
        "{} {} {}",
        name.to_lowercase(),
        driver.to_lowercase(),
        port.to_lowercase()
    );
    combined.contains("pdf")
        || combined.contains("xps")
        || combined.contains("onenote")
        || combined.contains("fax")
        || combined.contains("portprompt")
}

#[cfg(target_os = "windows")]
fn map_status(status: u32, attributes: u32) -> PrinterStatus {
    if (attributes & PRINTER_ATTRIBUTE_WORK_OFFLINE) != 0 || (status & PRINTER_STATUS_OFFLINE) != 0 {
        PrinterStatus::Offline
    } else if (status & PRINTER_STATUS_PAPER_OUT) != 0 || (status & PRINTER_STATUS_PAPER_JAM) != 0 {
        PrinterStatus::PaperOut
    } else if (status & PRINTER_STATUS_PAUSED) != 0 {
        PrinterStatus::Paused
    } else if (status & PRINTER_STATUS_ERROR) != 0 {
        PrinterStatus::Error
    } else if (status & PRINTER_STATUS_BUSY) != 0 {
        PrinterStatus::Busy
    } else {
        PrinterStatus::Ready
    }
}

/// Lista todas las impresoras físicas y virtuales instaladas en Windows, poniendo primero la predeterminada.
#[cfg(target_os = "windows")]
pub fn list_printers() -> Result<Vec<PrinterInfo>> {
    unsafe {
        let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
        let mut needed: u32 = 0;
        let mut returned: u32 = 0;
        let _ = EnumPrintersW(flags, None, 2, None, &mut needed, &mut returned);
        if needed == 0 {
            return Ok(Vec::new());
        }

        let mut buffer = vec![0u8; needed as usize];
        let ok = EnumPrintersW(
            flags,
            None,
            2,
            Some(&mut buffer),
            &mut needed,
            &mut returned,
        );
        if ok.is_err() || returned == 0 {
            return Ok(Vec::new());
        }

        let default_name = get_default_printer_name().unwrap_or_default();
        let items = std::slice::from_raw_parts(
            buffer.as_ptr() as *const PRINTER_INFO_2W,
            returned as usize,
        );

        let mut list = Vec::with_capacity(returned as usize);
        for info in items {
            let name = pwstr_to_string(info.pPrinterName);
            if name.is_empty() {
                continue;
            }
            let driver_name = pwstr_to_string(info.pDriverName);
            let port_name = pwstr_to_string(info.pPortName);
            let is_default = !default_name.is_empty() && name.eq_ignore_ascii_case(&default_name);
            let is_virtual = is_virtual_printer(&name, &driver_name, &port_name);
            let status = map_status(info.Status, info.Attributes);

            list.push(PrinterInfo {
                name,
                is_default,
                status,
                driver_name,
                port_name,
                is_virtual,
            });
        }

        // Ordenar: predeterminada primero, luego físicas antes que virtuales, luego alfabético
        list.sort_by(|a, b| {
            b.is_default
                .cmp(&a.is_default)
                .then_with(|| a.is_virtual.cmp(&b.is_virtual))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Ok(list)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn list_printers() -> Result<Vec<PrinterInfo>> {
    Ok(Vec::new())
}
