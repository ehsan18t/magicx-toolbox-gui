//! Personal details out of every log line before any sink sees it, and the manual tests' report
//! scrubber. Runs inside the logger: nothing here may panic or call `log::`.

use std::collections::HashSet;

use regex_lite::Regex;

/// Account and machine names that are never redacted: they name nobody.
const STOPLIST: &[&str] = &[
    "administrator",
    "admin",
    "user",
    "users",
    "default",
    "defaultuser0",
    "public",
    "system",
    "guest",
    "owner",
    "test",
    "dev",
    "pc",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// Anywhere, even inside a word: the manual tests' report scrubber.
    #[cfg(any(test, feature = "test-build"))]
    Substring,
    /// Not followed by a word character, so `C:\Users\Tim` leaves `C:\Users\Timothy` alone.
    Path,
    /// A word on its own: "Tim" never breaks "Optimize", "app" never breaks `app_lib`.
    Name,
}

struct Literal {
    /// ASCII-lowercased. Unicode lowercasing changes byte lengths and would break the slicing.
    lower: String,
    label: &'static str,
    kind: Kind,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// PowerShell's CLIXML `_x000D_` escape glued to a name.
fn is_clixml_escape(s: &[u8]) -> bool {
    s.len() == 7
        && s.starts_with(b"_x")
        && s[6] == b'_'
        && s[2..6].iter().all(u8::is_ascii_hexdigit)
}

/// Longest identifier a cut can split: a SID, a GUID, a profile folder.
pub const FRAGMENT_SPAN: usize = 64;

fn is_separator(c: char) -> bool {
    c.is_whitespace() || r#"\/"'(),;<>[]{}|="#.contains(c)
}

fn floor_char(text: &str, at: usize) -> usize {
    (0..=at.min(text.len()))
        .rev()
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(0)
}

fn ceil_char(text: &str, at: usize) -> usize {
    (at.min(text.len())..=text.len())
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(text.len())
}

/// Where to end text cut at `at`: after the last separator within [`FRAGMENT_SPAN`] bytes, else
/// that span earlier, so no fragment of an identifier survives to dodge redaction.
pub fn cut_end(text: &str, at: usize) -> usize {
    if at >= text.len() {
        return text.len();
    }
    let at = floor_char(text, at);
    if text[at..].starts_with(is_separator) {
        return at;
    }
    let from = floor_char(text, at.saturating_sub(FRAGMENT_SPAN));
    text[from..at]
        .char_indices()
        .rev()
        .find(|(_, c)| is_separator(*c))
        .map_or(from, |(i, c)| from + i + c.len_utf8())
}

/// Where to start text cut at `at`: the mirror of [`cut_end`].
pub fn cut_start(text: &str, at: usize) -> usize {
    if at == 0 {
        return 0;
    }
    let at = ceil_char(text, at);
    if text[..at].ends_with(is_separator) {
        return at;
    }
    let to = ceil_char(text, at + FRAGMENT_SPAN);
    text[at..to]
        .char_indices()
        .find(|(_, c)| is_separator(*c))
        .map_or(to, |(i, c)| at + i + c.len_utf8())
}

impl Literal {
    fn fits(&self, hay: &str, at: usize) -> bool {
        let end = at + self.lower.len();
        let bytes = hay.as_bytes();
        let after = || {
            !bytes.get(end..end + 7).is_some_and(is_clixml_escape)
                && hay
                    .get(end..)
                    .and_then(|s| s.chars().next())
                    .is_some_and(is_word)
        };
        let before = || {
            !(at >= 7 && is_clixml_escape(&bytes[at - 7..at]))
                && hay
                    .get(..at)
                    .and_then(|s| s.chars().next_back())
                    .is_some_and(is_word)
        };
        match self.kind {
            #[cfg(any(test, feature = "test-build"))]
            Kind::Substring => true,
            Kind::Path => !after(),
            Kind::Name => !before() && !after(),
        }
    }
}

/// One left-to-right pass, longest literal first at each position, so a placeholder already
/// written is never matched again.
fn scan(hay: &str, literals: &[Literal], swallow_path_tail: bool) -> String {
    let lower = hay.to_ascii_lowercase();
    let (bytes, folded) = (hay.as_bytes(), lower.as_bytes());
    let mut out = String::with_capacity(hay.len());
    let (mut copied, mut at) = (0, 0);
    while at < bytes.len() {
        let hit = hay.is_char_boundary(at).then(|| {
            literals
                .iter()
                .find(|l| folded[at..].starts_with(l.lower.as_bytes()) && l.fits(hay, at))
        });
        let Some(Some(literal)) = hit else {
            at += 1;
            continue;
        };
        out.push_str(hay.get(copied..at).unwrap_or_default());
        out.push_str(literal.label);
        let mut end = at + literal.lower.len();
        let tail = hay.get(end..).unwrap_or_default();
        if swallow_path_tail && tail.starts_with(['\\', '/']) {
            end += tail
                .find(|c: char| c.is_whitespace() || "\"'(),;".contains(c))
                .unwrap_or(tail.len());
        }
        copied = end;
        at = end;
    }
    out.push_str(hay.get(copied..).unwrap_or_default());
    out
}

/// What identifies this user and machine, gathered once before the logger is installed.
#[derive(Debug, Default, Clone)]
pub struct Identity {
    pub paths: Vec<(String, &'static str)>,
    pub names: Vec<(String, &'static str)>,
    pub ids: Vec<(String, &'static str)>,
}

impl Identity {
    /// The session account comes from WTS as a string: `LookupAccountNameW` can block on a domain.
    pub fn from_machine() -> Self {
        use crate::tweaks::engine::context::{RealSidProbe, SidProbe};
        let mut paths = Vec::new();
        if let Some(dir) = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.display().to_string()))
        {
            paths.push((dir, "<app-dir>"));
        }
        let (env_paths, mut names) = env_literals(&|var| std::env::var(var).ok());
        paths.extend(env_paths);
        if let Some(account) = RealSidProbe.session_account_name() {
            if let Some((_, user)) = account.rsplit_once('\\') {
                names.push((user.to_string(), "<user>"));
            }
            names.push((account, "<user>"));
        }
        let mut ids = Vec::new();
        ids.extend(RealSidProbe.process_token_sid().map(|v| (v, "<sid>")));
        ids.extend(
            crate::services::system_info_service::machine_guid().map(|v| (v, "<machine-guid>")),
        );
        Self { paths, names, ids }
    }
}

type Literals = Vec<(String, &'static str)>;

/// A OneDrive folder names the organisation; USERDOMAIN equals COMPUTERNAME on a local account.
fn env_literals(env: &dyn Fn(&str) -> Option<String>) -> (Literals, Literals) {
    let mut paths = Vec::new();
    for (var, label) in [
        ("USERPROFILE", "%USERPROFILE%"),
        ("LOCALAPPDATA", "%LOCALAPPDATA%"),
        ("APPDATA", "%APPDATA%"),
        ("TEMP", "%TEMP%"),
        ("TMP", "%TEMP%"),
        ("OneDrive", "%OneDrive%"),
        ("OneDriveCommercial", "%OneDrive%"),
        ("OneDriveConsumer", "%OneDrive%"),
    ] {
        paths.extend(env(var).map(|v| (v, label)));
    }
    let mut names = Vec::new();
    names.extend(env("USERNAME").map(|v| (v, "<user>")));
    let computer = env("COMPUTERNAME");
    names.extend(env("USERDNSDOMAIN").map(|v| (v, "<domain>")));
    names.extend(
        env("USERDOMAIN")
            .filter(|d| computer.as_ref().is_none_or(|c| !c.eq_ignore_ascii_case(d)))
            .map(|v| (v, "<domain>")),
    );
    names.extend(computer.map(|v| (v, "<computer>")));
    (paths, names)
}

pub struct Redactor {
    literals: Vec<Literal>,
    sid: Regex,
    email: Regex,
    profile: Regex,
}

impl Redactor {
    pub fn new(identity: &Identity) -> Self {
        let mut literals = Vec::new();
        for (path, label) in &identity.paths {
            let path = path.trim_end_matches(['\\', '/']);
            if path.len() < 4 {
                continue;
            }
            for form in [
                path.to_string(),
                path.replace('\\', "\\\\"),
                path.replace('\\', "/"),
            ] {
                literals.push((form, *label, Kind::Path));
            }
        }
        for (name, label) in &identity.names {
            let own = name.rsplit('\\').next().unwrap_or(name);
            if own.chars().count() < 3 || STOPLIST.contains(&own.to_ascii_lowercase().as_str()) {
                continue;
            }
            literals.push((name.clone(), *label, Kind::Name));
            if name.contains('\\') {
                literals.push((name.replace('\\', "\\\\"), *label, Kind::Name));
            }
        }
        for (id, label) in &identity.ids {
            if id.len() >= 4 {
                literals.push((id.clone(), *label, Kind::Path));
            }
        }
        let mut seen = HashSet::new();
        let mut literals: Vec<Literal> = literals
            .into_iter()
            .map(|(text, label, kind)| Literal {
                lower: text.to_ascii_lowercase(),
                label,
                kind,
            })
            .filter(|l| seen.insert(l.lower.clone()))
            .collect();
        // Stable: the exe directory, pushed first, wins a tie.
        literals.sort_by_key(|l| std::cmp::Reverse(l.lower.len()));
        let regex = |pattern: &str| Regex::new(pattern).expect("constant pattern");
        Self {
            literals,
            sid: regex(r"(?i)S-1-5-21-\d+-\d+-\d+(?:-\d+)?|S-1-12-1-\d+-\d+-\d+(?:-\d+)?"),
            email: regex(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}"),
            // Drive, UNC or \Device prefix; the profile name runs to the next separator or quote.
            profile: regex(
                r#"(?i)(?:[a-z]:|[\\/]{2,}[^\\/\s"']+|[\\/]+device[\\/]+[^\\/\s"']+)[\\/]+users[\\/]+[^\\/"'\r\n]+"#,
            ),
        }
    }

    /// Emails first, on the raw text: the name pass would split `smith.alice@contoso.com`. SIDs and
    /// profiles after the literals, so the user's own profile reads `%USERPROFILE%`.
    pub fn redact(&self, text: &str) -> String {
        let text = if text.contains('@') {
            self.email.replace_all(text, "<email>")
        } else {
            std::borrow::Cow::Borrowed(text)
        };
        let out = scan(&text, &self.literals, false);
        let lower = out.to_ascii_lowercase();
        if !(lower.contains("s-1-") || lower.contains("users\\") || lower.contains("users/")) {
            return out;
        }
        let out = self.sid.replace_all(&out, "<sid>");
        self.profile.replace_all(&out, "<profile>").into_owned()
    }
}

/// The manual tests' report scrubber: a match followed by a path separator takes the rest of that
/// path with it, and every match reads `<redacted>`.
#[cfg(any(test, feature = "test-build"))]
pub struct Scrubber(Vec<Literal>);

#[cfg(any(test, feature = "test-build"))]
impl Scrubber {
    pub fn new(list: &[&str]) -> Self {
        let mut literals: Vec<Literal> = list
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| Literal {
                lower: s.to_ascii_lowercase(),
                label: "<redacted>",
                kind: Kind::Substring,
            })
            .collect();
        literals.sort_by_key(|l| std::cmp::Reverse(l.lower.len()));
        Self(literals)
    }

    /// The account name and the profile and temp folders.
    pub fn for_reports() -> Self {
        let mut list: Vec<String> = ["USERPROFILE", "TEMP", "TMP", "LOCALAPPDATA", "APPDATA"]
            .iter()
            .filter_map(|v| std::env::var(v).ok())
            .collect();
        list.extend(std::env::var("USERNAME").ok().filter(|u| u.len() >= 3));
        Self::new(&list.iter().map(String::as_str).collect::<Vec<_>>())
    }

    pub fn apply(&self, msg: &str) -> String {
        scan(msg, &self.0, true)
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    pub fn identity(user: &str) -> Identity {
        Identity {
            paths: vec![
                (format!(r"C:\Users\{user}"), "%USERPROFILE%"),
                (format!(r"C:\Users\{user}\AppData\Local"), "%LOCALAPPDATA%"),
                (format!(r"C:\Users\{user}\AppData\Local\Temp"), "%TEMP%"),
                (format!(r"C:\Users\{user}\Downloads\MagicX"), "<app-dir>"),
            ],
            names: vec![
                (user.to_string(), "<user>"),
                ("DESKTOP-K2J9".to_string(), "<computer>"),
                (format!(r"DESKTOP-K2J9\{user}"), "<user>"),
            ],
            ids: vec![
                ("S-1-5-21-111-222-333-1001".to_string(), "<sid>"),
                (
                    "6f1d2c3b-aaaa-4bbb-8ccc-0123456789ab".to_string(),
                    "<machine-guid>",
                ),
            ],
        }
    }

    fn redact(user: &str, text: &str) -> String {
        Redactor::new(&identity(user)).redact(text)
    }

    #[test]
    fn a_path_keeps_its_tail_under_the_placeholder() {
        assert_eq!(
            redact("Alice", r"read c:\users\alice\Documents\x.txt"),
            r"read %USERPROFILE%\Documents\x.txt"
        );
        assert_eq!(
            redact("Alice", r"C:\Users\Alice\AppData\Local\Temp\req.json"),
            r"%TEMP%\req.json"
        );
        assert_eq!(
            redact(
                "Alice",
                r#"path: "C:\\Users\\Alice\\Downloads\\MagicX\\a.exe""#
            ),
            r#"path: "<app-dir>\\a.exe""#
        );
        assert_eq!(
            redact("Alice", "C:/Users/Alice/AppData/Local/x"),
            "%LOCALAPPDATA%/x"
        );
    }

    #[test]
    fn names_match_whole_words_only() {
        assert_eq!(redact("Tim", "Optimize for Tim"), "Optimize for <user>");
        assert_eq!(
            redact("dev", r"\Device\HarddiskVolume3"),
            r"\Device\HarddiskVolume3"
        );
        assert_eq!(
            redact("Admin", "Administrator and Admin"),
            "Administrator and Admin"
        );
        assert_eq!(
            redact("app", "app_lib::commands app"),
            "app_lib::commands <user>"
        );
        assert_eq!(
            redact("Alice", r"DESKTOP-K2J9\Alice on DESKTOP-K2J9"),
            "<user> on <computer>"
        );
    }

    #[test]
    fn a_placeholder_is_never_matched_again() {
        assert_eq!(redact("app", r"C:\Users\app\x"), r"%USERPROFILE%\x");
        assert_eq!(redact("user", "user"), "user");
    }

    #[test]
    fn a_non_ascii_name_is_redacted_without_panicking() {
        assert_eq!(redact("Jörg", "Jörg und JÖRG, ö"), "<user> und JÖRG, ö");
        assert_eq!(
            redact("Jörg", r"C:\USERS\Jörg\x and é"),
            r"%USERPROFILE%\x and é"
        );
        for text in ["", "ö", "Jö", "Jörg", "€€€Jörg€", "\u{10FFFF}Jörg"] {
            redact("Jörg", text);
        }
    }

    #[test]
    fn ids_and_other_profiles_are_redacted() {
        assert_eq!(
            redact(
                "Alice",
                "sid S-1-5-21-111-222-333-1001 guid 6F1D2C3B-AAAA-4BBB-8CCC-0123456789AB"
            ),
            "sid <sid> guid <machine-guid>"
        );
        assert_eq!(
            redact(
                "Alice",
                "other S-1-5-21-9-8-7-500 and s-1-12-1-1-2-3-4, me@example.com"
            ),
            "other <sid> and <sid>, <email>"
        );
        assert_eq!(
            redact("Alice", r#"C:\Users\Bob\x "c:\\users\\Carol\\y""#),
            r#"<profile>\x "<profile>\\y""#
        );
        assert_eq!(redact("Alice", r"C:\Users\Alice\x"), r"%USERPROFILE%\x");
    }

    #[test]
    fn an_email_holding_the_user_name_is_redacted_whole() {
        assert_eq!(
            redact("Alice", "mail smith.alice@contoso.com and Alice"),
            "mail <email> and <user>"
        );
    }

    #[test]
    fn a_clixml_escape_is_a_name_boundary() {
        assert_eq!(
            redact("Alice", "for Alice_x000D__x000A_ and _x000A_Alice"),
            "for <user>_x000D__x000A_ and _x000A_<user>"
        );
        assert_eq!(redact("Alice", "Alice_x00"), "Alice_x00");
    }

    #[test]
    fn other_profiles_are_redacted_in_every_path_form() {
        for (text, want) in [
            ("C:/Users/Bob/Documents", "<profile>/Documents"),
            (r"\\server\Users\Bob\x", r"<profile>\x"),
            (r"\Device\HarddiskVolume3\Users\Bob\x", r"<profile>\x"),
            (r"C:\\\\Users\\\\Bob\\\\x", r"<profile>\\\\x"),
            (r"C:\Users\Bob Jones\x", r"<profile>\x"),
            (
                r#"open "C:\Users\Bob Jones" now"#,
                r#"open "<profile>" now"#,
            ),
        ] {
            assert_eq!(redact("Alice", text), want, "{text}");
        }
    }

    #[test]
    fn a_machine_sid_without_its_rid_is_redacted() {
        assert_eq!(
            redact("Alice", "domain S-1-5-21-9-8-7 and S-1-12-1-1-2-3"),
            "domain <sid> and <sid>"
        );
    }

    #[test]
    fn onedrive_folders_and_the_domain_are_redacted() {
        let vars = [
            ("USERPROFILE", r"C:\Users\Alice"),
            ("OneDrive", r"C:\Users\Alice\OneDrive - Contoso Ltd"),
            (
                "OneDriveCommercial",
                r"C:\Users\Alice\OneDrive - Contoso Ltd",
            ),
            ("OneDriveConsumer", r"C:\Users\Alice\OneDrive"),
            ("USERNAME", "Alice"),
            ("COMPUTERNAME", "DESKTOP-K2J9"),
            ("USERDOMAIN", "CONTOSO"),
            ("USERDNSDOMAIN", "CORP.CONTOSO.COM"),
        ];
        let (paths, names) = env_literals(&|var| {
            vars.iter()
                .find(|(k, _)| *k == var)
                .map(|(_, v)| v.to_string())
        });
        let r = Redactor::new(&Identity {
            paths,
            names,
            ids: Vec::new(),
        });
        assert_eq!(
            r.redact(r"C:\Users\Alice\OneDrive - Contoso Ltd\a.docx, C:\Users\Alice\OneDrive\b"),
            r"%OneDrive%\a.docx, %OneDrive%\b"
        );
        assert_eq!(
            r.redact(r"CONTOSO\x on corp.contoso.com"),
            r"<domain>\x on <domain>"
        );
        let (_, names) = env_literals(&|var| match var {
            "USERDOMAIN" | "COMPUTERNAME" => Some("DESKTOP-K2J9".into()),
            _ => None,
        });
        assert!(
            names.iter().all(|(_, label)| *label != "<domain>"),
            "{names:?}"
        );
    }

    #[test]
    fn a_cut_never_leaves_a_fragment_of_an_identifier() {
        let text = "x S-1-5-21-111-222-333-1001 y";
        let at = text.find("333").unwrap();
        assert_eq!(&text[..cut_end(text, at)], "x ");
        assert_eq!(&text[cut_start(text, at)..], "y");
        let path = r"C:\Users\Alice\AppData";
        assert_eq!(&path[..cut_end(path, 12)], r"C:\Users\");
        let long = format!("{}tail", "z".repeat(200));
        assert_eq!(cut_end(&long, 150), 150 - FRAGMENT_SPAN);
        assert_eq!(cut_start(&long, 50), 50 + FRAGMENT_SPAN);
        assert_eq!(cut_end("a b", 3), 3);
        assert_eq!(cut_start("a b", 0), 0);
        let wide = "ééé ééé";
        for at in 0..=wide.len() {
            assert!(wide.is_char_boundary(cut_end(wide, at)));
            assert!(wide.is_char_boundary(cut_start(wide, at)));
        }
    }

    #[test]
    fn the_report_scrubber_is_case_insensitive_and_takes_the_whole_path() {
        let scrubber = Scrubber::new(&[r"C:\Users\Alice", "Alice"]);
        assert_eq!(
            scrubber.apply(r"read c:\users\alice\x and ALICE"),
            "read <redacted> and <redacted>"
        );
        assert_eq!(
            scrubber.apply(r"spawn C:\Users\Alice\Temp\req-9f3a.json (os error 5)"),
            "spawn <redacted> (os error 5)"
        );
        assert_eq!(scrubber.apply("é Alice ü"), "é <redacted> ü");
    }
}
