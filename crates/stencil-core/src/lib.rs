mod qr;
mod vcard;

pub use qr::{encode_contact_png, encode_png, encode_svg, QrImage};
pub use vcard::{build_vcard, Contact};

pub const VERSION: &str = "0.2.2-beta";
pub const COFFEE_URL: &str = "https://buymeacoffee.com/wilsonsamiano";
pub const RELEASE_REPO: &str = "wilsonsamiano/stencil";

/// `0.1.0-beta` compared to a GitHub tag such as `v0.1.1-beta`.
pub fn is_newer_release(current: &str, tag: &str) -> bool {
    fn parts(value: &str) -> (u32, u32, u32, bool) {
        let bare = value.trim().trim_start_matches('v');
        let (num, pre) = bare.split_once('-').unwrap_or((bare, ""));
        let mut bits = num.split('.');
        let major = bits.next().unwrap_or("0").parse().unwrap_or(0);
        let minor = bits.next().unwrap_or("0").parse().unwrap_or(0);
        let patch = bits.next().unwrap_or("0").parse().unwrap_or(0);
        (major, minor, patch, !pre.is_empty())
    }
    let (cmaj, cmin, cpat, cpre) = parts(current);
    let (tmaj, tmin, tpat, tpre) = parts(tag);
    if (tmaj, tmin, tpat) != (cmaj, cmin, cpat) {
        return (tmaj, tmin, tpat) > (cmaj, cmin, cpat);
    }
    cpre && !tpre
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_beta_is_not_newer() {
        assert!(!is_newer_release("0.1.0-beta", "v0.1.0-beta"));
    }

    #[test]
    fn stable_and_next_beta_are_newer() {
        assert!(is_newer_release("0.1.0-beta", "v0.1.0"));
        assert!(is_newer_release("0.1.0-beta", "v0.1.1-beta"));
        assert!(!is_newer_release("0.1.1", "v0.1.0-beta"));
    }
}
