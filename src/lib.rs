//! `yeib_print`: Motor nativo de enumeración de impresoras e impresión silenciosa Win32 GDI/Spooler.

pub mod devmode;
pub mod enum_win32;
pub mod gdi_win32;
pub mod range;
pub mod types;

pub use enum_win32::{get_default_printer_name, list_printers};
pub use gdi_win32::print_page_sources;
pub use range::{compute_target_rect, decode_page_source, parse_page_range, prepare_page_dib};
pub use types::{
    PrintColorMode, PrintError, PrintJobResult, PrintJobSpec, PrintOrientation, PrintPaperSize,
    PrintScaleMode, PrinterInfo, PrinterStatus, Result,
};
