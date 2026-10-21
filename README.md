# Stencil

Public beta 0.1.0. QR codes and contact cards. The encoder is Rust.

[Buy me a coffee](https://buymeacoffee.com/wilsonsamiano)

## Where it runs

| Platform | How you use this beta |
| --- | --- |
| Linux x86_64 | `stencil` command from the release archive |
| macOS Apple silicon and Intel | `stencil` command from the release archive |
| Windows x86_64 | `stencil.exe` from the release archive |
| Android, iOS, and desktop browsers | The Stencil studio. Add it to your home screen. The page loads this repo's Rust encoder as WebAssembly. |
| Updates | Studio: **Check for updates**. Command: `stencil update`. Both read the public GitHub releases, including betas. |

This beta does not ship an App Store or Play Store package. Android and iOS install the studio from the browser. When a newer beta is tagged, the studio says so. Reload the page after the studio itself is republished. `stencil update` downloads the archive for the computer you are on.

## Command

```bash
stencil qr --text "https://example.com" --out code.png
stencil qr --text "Meet at the south gate" --level H --fg "#f4f1e6" --bg "#141814" --out note.png
stencil vcard --first Jordan --last Hale --phone "+1 555 0142" --email jordan@example.com --out jordan.png
stencil vcard --first Jordan --last Hale --phone "+1 555 0142" --out jordan.vcf
stencil update
stencil --version
```

`--out` ending in `.svg` writes an SVG. `--out` ending in `.vcf` writes the vCard. Anything else is a PNG.

## Build

```bash
cargo build -p stencil-cli --release
rustup target add wasm32-unknown-unknown
cargo build -p stencil-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web/engine \
  target/wasm32-unknown-unknown/release/stencil_wasm.wasm
```

`web/engine` is the browser build used by the studio. GitHub Actions builds the Linux, macOS, and Windows archives when a `v*` tag is pushed.

## License

MIT. Wilson Samiano.
