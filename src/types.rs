//! Tipos y contratos de datos (serde camelCase) para `yeib_print`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PrintError>;

#[derive(Debug, Error)]
pub enum PrintError {
    #[error("Impresora no encontrada o no especificada: {0}")]
    PrinterNotFound(String),
    #[error("Rango de páginas inválido: {0}")]
    InvalidRange(String),
    #[error("Parámetro de impresión inválido: {0}")]
    InvalidSpec(String),
    #[error("Error decodificando imagen de página: {0}")]
    ImageDecode(String),
    #[error("Error del sistema de impresión de Windows: {0}")]
    Spooler(String),
}

impl Serialize for PrintError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrinterStatus {
    Ready,
    Offline,
    PaperOut,
    Paused,
    Busy,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterInfo {
    pub name: String,
    pub is_default: bool,
    pub status: PrinterStatus,
    pub driver_name: String,
    pub port_name: String,
    pub is_virtual: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrintOrientation {
    #[default]
    Auto,
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrintColorMode {
    #[default]
    Color,
    Grayscale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrintScaleMode {
    #[default]
    Fit,
    Actual,
    Fill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintJobSpec {
    pub printer_name: String,
    pub document_title: String,
    pub copies: u16,
    pub collate: bool,
    pub orientation: PrintOrientation,
    pub color_mode: PrintColorMode,
    pub scale_mode: PrintScaleMode,
    /// Rango 1-based ("", "all", "1-3, 5").
    pub page_range: String,
}

impl Default for PrintJobSpec {
    fn default() -> Self {
        Self {
            printer_name: String::new(),
            document_title: "Documento Yeib".to_string(),
            copies: 1,
            collate: true,
            orientation: PrintOrientation::Auto,
            color_mode: PrintColorMode::Color,
            scale_mode: PrintScaleMode::Fit,
            page_range: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintJobResult {
    pub printer_name: String,
    pub pages_printed: usize,
    pub copies: u16,
}
