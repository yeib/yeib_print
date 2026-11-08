//! Envío silencioso de páginas a la cola de impresión de Windows vía GDI (`CreateDCW` + `StretchDIBits`).

use crate::range::parse_page_range;
use crate::types::{PrintJobResult, PrintJobSpec, Result};

#[cfg(target_os = "windows")]
use crate::{
    devmode::build_custom_devmode,
    enum_win32::get_default_printer_name,
    range::{compute_target_rect, decode_page_source, prepare_page_dib},
    types::PrintError,
};

#[cfg(target_os = "windows")]
use windows::{
    core::PCWSTR,
    Win32::Graphics::Gdi::{
        CreateDCW, DeleteDC, GetDeviceCaps, SetStretchBltMode, StretchDIBits, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DEVMODEW, DIB_RGB_COLORS, HALFTONE, HDC, HORZRES, SRCCOPY,
        VERTRES,
    },
    Win32::Storage::Xps::{AbortDoc, EndDoc, EndPage, StartDocW, StartPage, DOCINFOW},
};

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Imprime una lista de páginas (en base64, data-URI o rutas de archivo de imagen) directamente en la impresora.
#[cfg(target_os = "windows")]
pub fn print_page_sources(spec: &PrintJobSpec, page_sources: &[String]) -> Result<PrintJobResult> {
    if page_sources.is_empty() {
        return Err(PrintError::InvalidSpec("No hay páginas para imprimir".into()));
    }
    let printer_name = if spec.printer_name.trim().is_empty() {
        get_default_printer_name().ok_or_else(|| {
            PrintError::PrinterNotFound("No hay impresora predeterminada configurada".into())
        })?
    } else {
        spec.printer_name.trim().to_string()
    };

    let selected_indices = parse_page_range(&spec.page_range, page_sources.len())?;
    let copies = spec.copies.clamp(1, 99);

    // Pre-decodificar las páginas seleccionadas antes de abrir el documento en el Spooler
    let mut raw_pages = Vec::with_capacity(selected_indices.len());
    for &idx in &selected_indices {
        let bytes = decode_page_source(&page_sources[idx])?;
        raw_pages.push(bytes);
    }

    unsafe {
        let driver_w = to_wide("WINSPOOL");
        let printer_w = to_wide(&printer_name);
        let custom_devmode = build_custom_devmode(&printer_w, spec);
        let p_devmode = custom_devmode
            .as_ref()
            .map(|b| b.as_ptr() as *const DEVMODEW);

        let hdc: HDC = CreateDCW(
            PCWSTR(driver_w.as_ptr()),
            PCWSTR(printer_w.as_ptr()),
            PCWSTR::null(),
            p_devmode,
        );
        if hdc.is_invalid() {
            return Err(PrintError::PrinterNotFound(printer_name));
        }

        let result = execute_gdi_job(hdc, spec, &printer_name, copies, &raw_pages);
        let _ = DeleteDC(hdc);
        result
    }
}

#[cfg(target_os = "windows")]
unsafe fn execute_gdi_job(
    hdc: HDC,
    spec: &PrintJobSpec,
    printer_name: &str,
    copies: u16,
    raw_pages: &[Vec<u8>],
) -> Result<PrintJobResult> {
    let page_w = GetDeviceCaps(Some(hdc), HORZRES);
    let page_h = GetDeviceCaps(Some(hdc), VERTRES);
    if page_w <= 0 || page_h <= 0 {
        return Err(PrintError::Spooler(
            "La impresora devolvió dimensiones de página inválidas".into(),
        ));
    }
    let printer_is_landscape = page_w > page_h;

    let title = if spec.document_title.trim().is_empty() {
        "Documento Yeib"
    } else {
        spec.document_title.trim()
    };
    let title_w = to_wide(title);
    let doc_info = DOCINFOW {
        cbSize: std::mem::size_of::<DOCINFOW>() as i32,
        lpszDocName: PCWSTR(title_w.as_ptr()),
        lpszOutput: PCWSTR::null(),
        lpszDatatype: PCWSTR::null(),
        fwType: 0,
    };

    let job_id = StartDocW(hdc, &doc_info);
    if job_id <= 0 {
        return Err(PrintError::Spooler(
            "No se pudo iniciar el trabajo en la cola de impresión (StartDocW)".into(),
        ));
    }

    let _ = SetStretchBltMode(hdc, HALFTONE);
    let mut total_sent = 0usize;

    let order: Vec<usize> = if spec.collate {
        (0..copies)
            .flat_map(|_| 0..raw_pages.len())
            .collect()
    } else {
        (0..raw_pages.len())
            .flat_map(|i| std::iter::repeat_n(i, copies as usize))
            .collect()
    };

    for page_idx in order {
        let (img_w, img_h, bgra) = match prepare_page_dib(
            &raw_pages[page_idx],
            spec.color_mode,
            spec.orientation,
            printer_is_landscape,
        ) {
            Ok(data) => data,
            Err(e) => {
                let _ = AbortDoc(hdc);
                return Err(e);
            }
        };

        let (dst_x, dst_y, dst_w, dst_h) =
            compute_target_rect(img_w, img_h, page_w, page_h, spec.scale_mode);

        if StartPage(hdc) <= 0 {
            let _ = AbortDoc(hdc);
            return Err(PrintError::Spooler("Error al iniciar página (StartPage)".into()));
        }

        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: img_w as i32,
                biHeight: -(img_h as i32), // top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: bgra.len() as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let lines = StretchDIBits(
            hdc,
            dst_x,
            dst_y,
            dst_w,
            dst_h,
            0,
            0,
            img_w as i32,
            img_h as i32,
            Some(bgra.as_ptr() as *const _),
            &bmi,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
        if lines <= 0 {
            let _ = AbortDoc(hdc);
            return Err(PrintError::Spooler(
                "Error al enviar mapa de bits a la impresora (StretchDIBits)".into(),
            ));
        }

        if EndPage(hdc) <= 0 {
            let _ = AbortDoc(hdc);
            return Err(PrintError::Spooler("Error al finalizar página (EndPage)".into()));
        }
        total_sent += 1;
    }

    let _ = EndDoc(hdc);
    Ok(PrintJobResult {
        printer_name: printer_name.to_string(),
        pages_printed: total_sent,
        copies,
    })
}

#[cfg(not(target_os = "windows"))]
pub fn print_page_sources(spec: &PrintJobSpec, _page_sources: &[String]) -> Result<PrintJobResult> {
    Err(PrintError::Spooler(
        "La impresión nativa Win32 GDI solo es compatible con sistemas operativos Windows".into(),
    ))
}
