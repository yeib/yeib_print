//! Configuración de `DEVMODEW` (tamaño de papel, orientación) vía Win32 Spooler.

use crate::types::{PrintJobSpec, PrintOrientation, PrintPaperSize};

#[cfg(target_os = "windows")]
use windows::{
    core::PCWSTR,
    Win32::Graphics::Gdi::{
        DEVMODEW, DM_ORIENTATION, DM_PAPERSIZE, DMORIENT_LANDSCAPE, DMORIENT_PORTRAIT,
        DMPAPER_A3, DMPAPER_A4, DMPAPER_A5, DMPAPER_LEGAL, DMPAPER_LETTER,
    },
    Win32::Graphics::Printing::{ClosePrinter, DocumentPropertiesW, OpenPrinterW, PRINTER_HANDLE},
};

#[cfg(target_os = "windows")]
pub unsafe fn build_custom_devmode(
    printer_name_w: &[u16],
    spec: &PrintJobSpec,
) -> Option<Vec<u8>> {
    let mut hprinter = PRINTER_HANDLE::default();
    if OpenPrinterW(PCWSTR(printer_name_w.as_ptr()), &mut hprinter, None).is_err() {
        return None;
    }

    let size = DocumentPropertiesW(
        None,
        hprinter,
        PCWSTR(printer_name_w.as_ptr()),
        None,
        None,
        0,
    );

    if size <= 0 {
        let _ = ClosePrinter(hprinter);
        return None;
    }

    let mut buf = vec![0u8; size as usize];
    let p_devmode = buf.as_mut_ptr() as *mut DEVMODEW;

    let res = DocumentPropertiesW(
        None,
        hprinter,
        PCWSTR(printer_name_w.as_ptr()),
        Some(p_devmode),
        None,
        2, // DM_OUT_BUFFER
    );

    let _ = ClosePrinter(hprinter);

    if res < 0 {
        return None;
    }

    let devmode = &mut *p_devmode;
    match spec.paper_size {
        PrintPaperSize::Letter => {
            devmode.dmFields |= DM_PAPERSIZE;
            devmode.Anonymous1.Anonymous1.dmPaperSize = DMPAPER_LETTER as i16;
        }
        PrintPaperSize::A4 => {
            devmode.dmFields |= DM_PAPERSIZE;
            devmode.Anonymous1.Anonymous1.dmPaperSize = DMPAPER_A4 as i16;
        }
        PrintPaperSize::Legal => {
            devmode.dmFields |= DM_PAPERSIZE;
            devmode.Anonymous1.Anonymous1.dmPaperSize = DMPAPER_LEGAL as i16;
        }
        PrintPaperSize::A3 => {
            devmode.dmFields |= DM_PAPERSIZE;
            devmode.Anonymous1.Anonymous1.dmPaperSize = DMPAPER_A3 as i16;
        }
        PrintPaperSize::A5 => {
            devmode.dmFields |= DM_PAPERSIZE;
            devmode.Anonymous1.Anonymous1.dmPaperSize = DMPAPER_A5 as i16;
        }
        PrintPaperSize::Auto => {}
    }

    match spec.orientation {
        PrintOrientation::Portrait => {
            devmode.dmFields |= DM_ORIENTATION;
            devmode.Anonymous1.Anonymous1.dmOrientation = DMORIENT_PORTRAIT as i16;
        }
        PrintOrientation::Landscape => {
            devmode.dmFields |= DM_ORIENTATION;
            devmode.Anonymous1.Anonymous1.dmOrientation = DMORIENT_LANDSCAPE as i16;
        }
        PrintOrientation::Auto => {}
    }

    Some(buf)
}
