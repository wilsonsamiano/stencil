use std::fs;
use std::io::{self, Write};
use std::process::Command;

use clap::{Parser, Subcommand};
use stencil_core::{
    build_vcard, encode_contact_png, encode_png, encode_svg, is_newer_release, Contact, COFFEE_URL,
    RELEASE_REPO, VERSION,
};

#[derive(Parser)]
#[command(
    name = "stencil",
    version = VERSION,
    about = "Stencil makes QR codes and contact cards.",
    long_about = "Stencil makes QR codes and contact cards on Linux, macOS, and Windows.\nAndroid, iOS, and the installable web app use the same beta and check GitHub for updates."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode text or a URL as a PNG or SVG.
    Qr {
        #[arg(long)]
        text: String,
        #[arg(long, default_value = "M")]
        level: String,
        #[arg(long, default_value = "#141814")]
        fg: String,
        #[arg(long, default_value = "#f4f1e6")]
        bg: String,
        #[arg(long, default_value_t = 320)]
        size: u32,
        #[arg(long)]
        out: String,
    },
    /// Encode a contact as a vCard QR PNG.
    Vcard {
        #[arg(long, default_value = "")]
        first: String,
        #[arg(long, default_value = "")]
        last: String,
        #[arg(long, default_value = "")]
        phone: String,
        #[arg(long, default_value = "")]
        email: String,
        #[arg(long, default_value = "")]
        org: String,
        #[arg(long, default_value = "")]
        title: String,
        #[arg(long, default_value = "")]
        url: String,
        #[arg(long, default_value = "M")]
        level: String,
        #[arg(long, default_value = "#141814")]
        fg: String,
        #[arg(long, default_value = "#f4f1e6")]
        bg: String,
        #[arg(long, default_value_t = 360)]
        size: u32,
        #[arg(long)]
        out: String,
    },
    /// Download a newer public beta for this computer.
    Update,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match Cli::parse().command {
        Commands::Qr {
            text,
            level,
            fg,
            bg,
            size,
            out,
        } => {
            if out.ends_with(".svg") {
                let (_version, _modules, svg) = encode_svg(&text, &level, &fg, &bg)?;
                fs::write(&out, svg).map_err(|err| err.to_string())?;
            } else {
                let image = encode_png(&text, &level, &fg, &bg, size)?;
                fs::write(&out, image.png).map_err(|err| err.to_string())?;
            }
            println!("Wrote {out}");
        }
        Commands::Vcard {
            first,
            last,
            phone,
            email,
            org,
            title,
            url,
            level,
            fg,
            bg,
            size,
            out,
        } => {
            if out.ends_with(".vcf") {
                let card = build_vcard(&Contact {
                    first,
                    last,
                    phone,
                    email,
                    org,
                    title,
                    url,
                });
                if card.is_empty() {
                    return Err("Add a name, phone, or email.".into());
                }
                fs::write(&out, card).map_err(|err| err.to_string())?;
            } else {
                let image = encode_contact_png(
                    &first, &last, &phone, &email, &org, &title, &url, &level, &fg, &bg, size,
                )?;
                fs::write(&out, image.png).map_err(|err| err.to_string())?;
            }
            println!("Wrote {out}");
        }
        Commands::Update => update()?,
    }
    Ok(())
}

fn update() -> Result<(), String> {
    println!("Stencil {VERSION}");
    println!("Buy me a coffee: {COFFEE_URL}");
    let body = curl(&format!(
        "https://api.github.com/repos/{RELEASE_REPO}/releases?per_page=10"
    ))?;
    let value: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| "GitHub did not return a release.".to_string())?;
    let Some(release) = value
        .as_array()
        .and_then(|list| list.iter().find(|item| item["draft"].as_bool() != Some(true)))
    else {
        return Err("No public beta has been published yet.".into());
    };
    let tag = release["tag_name"].as_str().unwrap_or("");
    if tag.is_empty() {
        return Err("No public beta has been published yet.".into());
    }
    if !is_newer_release(VERSION, tag) {
        println!("You're on the latest public beta ({tag}).");
        return Ok(());
    }
    let asset_name = asset_name(tag);
    let assets = release["assets"].as_array().cloned().unwrap_or_default();
    let Some(url) = assets.iter().find_map(|asset| {
        let name = asset["name"].as_str()?;
        if name == asset_name {
            asset["browser_download_url"].as_str().map(str::to_string)
        } else {
            None
        }
    }) else {
        return Err(format!(
            "{tag} is published, but {asset_name} is not attached yet. See https://github.com/{RELEASE_REPO}/releases"
        ));
    };
    let bytes = curl_bytes(&url)?;
    fs::write(&asset_name, &bytes).map_err(|err| err.to_string())?;
    println!("Downloaded {asset_name}");
    println!("Replace this stencil binary with the one inside that archive, then run stencil --version.");
    Ok(())
}

fn asset_name(tag: &str) -> String {
    let version = tag.trim_start_matches('v');
    if cfg!(windows) {
        format!("stencil-{version}-windows-x86_64.zip")
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            format!("stencil-{version}-macos-aarch64.tar.gz")
        } else {
            format!("stencil-{version}-macos-x86_64.tar.gz")
        }
    } else {
        format!("stencil-{version}-linux-x86_64.tar.gz")
    }
}

fn curl(url: &str) -> Result<String, String> {
    let bytes = curl_bytes(url)?;
    String::from_utf8(bytes).map_err(|_| "GitHub returned a file, not text.".into())
}

fn curl_bytes(url: &str) -> Result<Vec<u8>, String> {
    let output = Command::new("curl")
        .args(["-fsSL", "-A", &format!("stencil/{VERSION}"), url])
        .output()
        .map_err(|_| "curl is required to check for updates.".to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Could not reach GitHub. {err}"));
    }
    let _ = io::stdout().flush();
    Ok(output.stdout)
}
