# 🖨️ yeib_print

Fast, clean, zero-margin Win32 GDI direct document printing engine for Rust and **Tauri** on Windows.

[![Crates.io](https://img.shields.io/crates/v/yeib_print.svg)](https://crates.io/crates/yeib_print)
[![Docs.rs](https://docs.rs/yeib_print/badge.svg)](https://docs.rs/yeib_print)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

---

## Why `yeib_print`?

In desktop frameworks like **Tauri** on Windows, `window.print()` triggers the built-in WebView2 / Microsoft Edge print dialog. This introduces unwanted header/footer text (URLs, dates), forced margins, and poor UI cohesion.

`yeib_print` solves this by talking directly to the **Windows Print Spooler** via modern Win32 GDI (`CreateDCW`, `StartDocW`, `StretchDIBits`). You can design your own bespoke, branded print preview UI and dispatch the actual printing job silently or directly with full control over paper sizes, orientation, copies, and color modes.

---

## ✨ Features

- **Direct GDI & Win32 Spooler Dispatch:** High-resolution page rendering without browser dialogs or margin artifacts.
- **Modern Windows Integration:** Powered by Microsoft's official `windows` crate (0.62) without legacy `winapi` overhead.
- **Printer Enumeration:** Discover installed printers, identify the system default printer, and retrieve port/driver information (`list_printers`).
- **Rich DEVMODE Configuration:** Configure paper sizes (`Letter`, `A4`, `Legal`, `A3`, `A5`), orientation (`Portrait`, `Landscape`, `Auto`), copies, and collation.
- **Flexible Page Inputs:** Accepts Base64 strings, Data-URIs (`data:image/png;base64,...`), raw bytes, or image file paths.
- **Aspect-Ratio Fitting:** Automatically centers and fits pages onto the target printable area without distortion.
- **Page Range Parsing:** Supports flexible range specs like `"all"`, `"1-3"`, `"1, 3, 5"`.

---

## 🚀 Quick Start

Add `yeib_print` to your `Cargo.toml`:

```toml
[dependencies]
yeib_print = "0.1"
```

### Basic Example

```rust
use yeib_print::{list_printers, print_page_sources, PrintJobSpec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. List installed printers
    let printers = list_printers()?;
    for p in &printers {
        println!("Printer: {} (Default: {})", p.name, p.is_default);
    }

    // 2. Configure print job
    let spec = PrintJobSpec {
        printer_name: "Microsoft Print to PDF".to_string(),
        document_title: "Yeib Invoice".to_string(),
        copies: 1,
        paper_size: "letter".to_string(),
        orientation: "portrait".to_string(),
        color_mode: "color".to_string(),
        scale_mode: "fit".to_string(),
        ..Default::default()
    };

    // 3. Dispatch pages (PNG, JPEG, Data-URI or Base64)
    let pages = vec![
        "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
    ];

    let result = print_page_sources(&spec, &pages)?;
    println!("Successfully printed {} pages to {}!", result.pages_printed, result.printer_name);

    Ok(())
}
```

---

## 🌐 Tauri 2 Command Integration

```rust
use tauri::command;
use yeib_print::{list_printers, print_page_sources, PrintJobResult, PrintJobSpec, PrinterInfo};

#[command]
pub async fn list_available_printers() -> Result<Vec<PrinterInfo>, String> {
    tauri::async_runtime::spawn_blocking(|| list_printers().map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[command]
pub async fn print_pages(spec: PrintJobSpec, pages: Vec<String>) -> Result<PrintJobResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        print_page_sources(&spec, &pages).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
```

---

## 📄 License

Dual-licensed under either:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

---

Developed with ❤️ by **[Yeib](https://github.com/yeib)**.
