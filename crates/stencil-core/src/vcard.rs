#[derive(Clone, Debug, Default)]
pub struct Contact {
    pub first: String,
    pub last: String,
    pub phone: String,
    pub email: String,
    pub org: String,
    pub title: String,
    pub url: String,
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace("\r\n", "\\n")
        .replace('\n', "\\n")
        .replace('\r', "\\n")
        .replace(';', "\\;")
        .replace(',', "\\,")
}

pub fn build_vcard(contact: &Contact) -> String {
    let first = contact.first.trim();
    let last = contact.last.trim();
    let phone = contact.phone.trim();
    let email = contact.email.trim();
    let org = contact.org.trim();
    let title = contact.title.trim();
    let url = contact.url.trim();
    let named = [first, last]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let fallback = [org, phone, email, title, url]
        .into_iter()
        .find(|part| !part.is_empty());
    let Some(fn_value) = (if named.is_empty() {
        fallback
    } else {
        Some(named.as_str())
    }) else {
        return String::new();
    };

    let mut lines = vec![
        "BEGIN:VCARD".to_string(),
        "VERSION:3.0".to_string(),
        if named.is_empty() {
            format!("N:{};;;;", escape(fn_value))
        } else {
            format!("N:{};{};;;", escape(last), escape(first))
        },
        format!("FN:{}", escape(fn_value)),
    ];
    if !org.is_empty() {
        lines.push(format!("ORG:{}", escape(org)));
    }
    if !title.is_empty() {
        lines.push(format!("TITLE:{}", escape(title)));
    }
    if !phone.is_empty() {
        lines.push(format!("TEL;TYPE=CELL:{}", escape(phone)));
    }
    if !email.is_empty() {
        lines.push(format!("EMAIL;TYPE=INTERNET:{}", escape(email)));
    }
    if !url.is_empty() {
        lines.push(format!("URL:{}", escape(url)));
    }
    lines.push("END:VCARD".to_string());
    lines.join("\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_contact_matches_phones() {
        let card = build_vcard(&Contact {
            first: "Jordan".into(),
            last: "Hale".into(),
            phone: "+1 555 0142".into(),
            email: "jordan@example.com".into(),
            org: "North Station".into(),
            title: "Producer".into(),
            url: "https://example.com".into(),
        });
        assert!(card.starts_with("BEGIN:VCARD\r\nVERSION:3.0\r\n"));
        assert!(card.contains("N:Hale;Jordan;;;"));
        assert!(card.contains("FN:Jordan Hale"));
        assert!(card.contains("TEL;TYPE=CELL:+1 555 0142"));
        assert!(card.ends_with("END:VCARD"));
    }

    #[test]
    fn escapes_specials() {
        let card = build_vcard(&Contact {
            first: "Ana".into(),
            last: "Hale, Jr.".into(),
            org: "North; Station".into(),
            title: "Line\nLead".into(),
            ..Contact::default()
        });
        assert!(card.contains("N:Hale\\, Jr.;Ana;;;"));
        assert!(card.contains("ORG:North\\; Station"));
        assert!(card.contains("TITLE:Line\\nLead"));
    }

    #[test]
    fn empty_contact_is_empty() {
        assert!(build_vcard(&Contact::default()).is_empty());
    }
}
