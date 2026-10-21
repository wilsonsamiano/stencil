use wasm_bindgen::prelude::*;

use stencil_core::{encode_png, encode_svg, VERSION};

#[wasm_bindgen]
pub fn engine_version() -> String {
    VERSION.to_string()
}

#[wasm_bindgen]
pub struct QrView {
    version: u32,
    modules: u32,
    svg: String,
}

#[wasm_bindgen]
impl QrView {
    #[wasm_bindgen(getter)]
    pub fn version(&self) -> u32 {
        self.version
    }

    #[wasm_bindgen(getter)]
    pub fn modules(&self) -> u32 {
        self.modules
    }

    #[wasm_bindgen(getter)]
    pub fn svg(&self) -> String {
        self.svg.clone()
    }
}

#[wasm_bindgen]
pub fn qr_svg(text: &str, level: &str, fg: &str, bg: &str) -> Result<QrView, JsValue> {
    let (version, modules, svg) = encode_svg(text, level, fg, bg).map_err(JsValue::from)?;
    Ok(QrView {
        version,
        modules,
        svg,
    })
}

#[wasm_bindgen]
pub fn qr_png(text: &str, level: &str, fg: &str, bg: &str, size: u32) -> Result<Vec<u8>, JsValue> {
    encode_png(text, level, fg, bg, size)
        .map(|image| image.png)
        .map_err(JsValue::from)
}
