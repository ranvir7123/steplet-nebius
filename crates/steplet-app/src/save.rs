//! The report as a plain text file (Task 13, S4): every section in the report's order, the outcome first and "The
//! agent says" last. Every piece of text the screens draw as file text (commands, what they printed, model names,
//! changed files, what the agent says) goes on lines of its own, each set off with "| ", and Steplet's own lines
//! never start so: text from the run can never read as Steplet's words. Every character that shows nothing is
//! written as its code, so the file hides nothing. The name Steplet offers comes from the time only, never from the
//! file.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::views::{CommandView, Part, ReportView, Seg, Shown};

/// A moment, as the name of a saved report uses it (this computer's own time).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub year: u16,
    pub month: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
}

impl Stamp {
    /// Now, in this computer's time.
    pub fn now() -> Stamp {
        use windows_sys::Win32::System::SystemInformation::GetLocalTime;
        // SAFETY: GetLocalTime only fills the structure it is given.
        let t = unsafe {
            let mut t = std::mem::zeroed();
            GetLocalTime(&mut t);
            t
        };
        Stamp { year: t.wYear, month: t.wMonth, day: t.wDay, hour: t.wHour, minute: t.wMinute }
    }
}

/// The name the Save dialog offers (S4): `steplet-report-YYYY-MM-DD-HHMM.txt`.
pub fn default_name(at: &Stamp) -> String {
    format!("steplet-report-{:04}-{:02}-{:02}-{:02}{:02}.txt", at.year, at.month, at.day, at.hour, at.minute)
}

/// The report as text, one `\n` between lines.
pub fn report_text(r: &ReportView) -> String {
    let mut out = Text::default();
    out.line("Steplet report");
    out.line("");
    out.line(
        "A line with \"|\" in front can hold text from the file or the run. Steplet's own words never have \"|\" in \
         front.",
    );

    out.heading("How the run ended");
    out.line(&format!("  {}. {}", r.end_words.title, r.end_words.text));
    if let crate::views::EndView::StoppedBySteplet { about, .. } = &r.end {
        out.quoted("  ", about);
    }

    out.heading("The commands, each with its check, its exit code and everything it printed");
    if r.commands.is_empty() {
        out.line("  The agent ran none.");
    }
    for (i, c) in r.commands.iter().enumerate() {
        command(&mut out, i + 1, c);
    }

    out.heading("What Steplet refused");
    list(&mut out, &r.refused, "Nothing.");

    out.heading("Your answers");
    if r.answers.is_empty() {
        out.line("  Steplet asked you nothing.");
    }
    for a in &r.answers {
        out.line("  - Tool:");
        out.quoted("    ", &a.tool);
        out.line("    What it asked for:");
        out.quoted("    ", &a.what);
        out.line(&format!("    {}", a.reply));
    }

    out.heading("The models");
    if r.models.is_empty() {
        out.line("  None recorded.");
    } else {
        out.line("  Used:");
        for m in &r.models {
            out.item("  ", m);
        }
    }
    for refusal in &r.refusals {
        out.line("  A model refused:");
        out.item("  ", &refusal.model);
        out.line("    The kind of request it refused:");
        out.quoted("    ", &refusal.category);
        out.line("    What the model said:");
        out.quoted("    ", &refusal.explanation);
        if let Some(to) = &refusal.switched_to {
            out.line("    Claude Code went on with:");
            out.quoted("    ", to);
        }
    }

    out.heading("The files the run changed in its working copy (the copy is deleted now)");
    match &r.changed_files {
        None => out.line("  Steplet could not look: what went wrong around the run says why."),
        Some(files) => list(&mut out, files, "None."),
    }

    out.heading("The trust check");
    out.line(&format!("  {}", r.trust));

    out.heading("What went wrong around the run");
    list(&mut out, &r.faults, "Nothing.");

    out.heading("Steplet's notes");
    for n in &r.notes {
        out.line(&format!("  - {n}"));
    }

    out.heading("The agent says (its own words, not Steplet's)");
    if plain(&r.agent_says).trim().is_empty() {
        out.line("  Nothing.");
    } else {
        out.quoted("  ", &r.agent_says);
    }
    out.0
}

/// Writes the report to `path`: UTF-8, with Windows line endings, through a file of its own that replaces `path`
/// only once it is whole.
pub fn write_report(path: &Path, r: &ReportView) -> io::Result<()> {
    let text = report_text(r).replace('\n', "\r\n");
    let part = PathBuf::from(format!("{}.part", path.display()));
    let written = (|| {
        let mut f = std::fs::File::create(&part)?;
        f.write_all(text.as_bytes())?;
        f.sync_all()
    })();
    let moved = written.and_then(|()| std::fs::rename(&part, path));
    if moved.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    moved
}

/// Asks where to save the report, with Windows' own Save dialog and the name of S4. `None` when the receiver
/// cancelled.
pub fn ask_where(owner: isize, at: &Stamp) -> Option<PathBuf> {
    use windows_sys::Win32::UI::Controls::Dialogs::{
        GetSaveFileNameW, OFN_EXPLORER, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
    };
    let mut file: Vec<u16> = default_name(at).encode_utf16().collect();
    file.resize(32_768, 0);
    let filter: Vec<u16> = "Text files (*.txt)\0*.txt\0\0".encode_utf16().collect();
    let extension: Vec<u16> = "txt\0".encode_utf16().collect();
    let title: Vec<u16> = "Save the report\0".encode_utf16().collect();
    // SAFETY: every pointer points into a buffer above that outlives the call; the sizes are the buffers' own.
    let chosen = unsafe {
        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = owner as _;
        ofn.lpstrFilter = filter.as_ptr();
        ofn.lpstrFile = file.as_mut_ptr();
        ofn.nMaxFile = file.len() as u32;
        ofn.lpstrDefExt = extension.as_ptr();
        ofn.lpstrTitle = title.as_ptr();
        ofn.Flags = OFN_EXPLORER | OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
        GetSaveFileNameW(&mut ofn) != 0
    };
    if !chosen {
        return None;
    }
    let end = file.iter().position(|&c| c == 0).unwrap_or(file.len());
    Some(PathBuf::from(String::from_utf16_lossy(&file[..end])))
}

fn command(out: &mut Text, n: usize, c: &CommandView) {
    out.line(&format!("  {n}. Tool:"));
    out.quoted("     ", &c.tool);
    out.line("     What it asked for:");
    out.quoted("     ", &c.command);
    out.line(&format!("     Steplet's check: {}", c.check.words));
    out.line(&format!("     {}", c.ended));
    if c.after_stop {
        out.line("     Steplet was already stopping the run: this was cut short or refused by the stop.");
    }
    match c.lines {
        0 => out.line("     What it printed: nothing"),
        1 => {
            out.line("     What it printed (1 line):");
            out.quoted("     ", &c.output);
        }
        n => {
            out.line(&format!("     What it printed ({n} lines):"));
            out.quoted("     ", &c.output);
        }
    }
}

fn list(out: &mut Text, items: &[Shown], none: &str) {
    if items.is_empty() {
        out.line(&format!("  {none}"));
    }
    for item in items {
        out.item("  ", item);
    }
}

/// The report's text, built line by line.
#[derive(Default)]
struct Text(String);

impl Text {
    fn line(&mut self, l: &str) {
        self.0.push_str(l);
        self.0.push('\n');
    }

    fn heading(&mut self, h: &str) {
        self.0.push('\n');
        self.line(h);
    }

    /// `s`, each of its lines set off with "| " after `indent`.
    fn quoted(&mut self, indent: &str, s: &[Seg]) {
        self.set_off(&format!("{indent}| "), &format!("{indent}| "), s);
    }

    /// `s` as an item of a list: "- | " before its first line, its other lines set off under it.
    fn item(&mut self, indent: &str, s: &[Seg]) {
        self.set_off(&format!("{indent}- | "), &format!("{indent}  | "), s);
    }

    fn set_off(&mut self, first: &str, rest: &str, s: &[Seg]) {
        let t = plain(s);
        for (i, l) in t.strip_suffix('\n').unwrap_or(&t).split('\n').enumerate() {
            self.line(format!("{}{l}", if i == 0 { first } else { rest }).trim_end());
        }
    }
}

/// The text of display pieces as the file shows it: a character that shows nothing as its code, a link as its
/// shown text and where it really goes.
fn plain(s: &[Seg]) -> String {
    let mut out = String::new();
    for seg in s {
        match seg {
            Seg::Text { text } => out.push_str(&clean(text)),
            Seg::Invisible { codepoint } => out.push_str(&code(*codepoint)),
            Seg::Link { shown, real } => match shown {
                Some(shown) => out.push_str(&format!("{} [goes to {}]", parts(shown), parts(real))),
                None => out.push_str(&parts(real)),
            },
        }
    }
    out
}

fn parts(p: &[Part]) -> String {
    p.iter()
        .map(|p| match p {
            Part::Text { text } => clean(text),
            Part::Invisible { codepoint } => code(*codepoint),
        })
        .collect()
}

fn code(cp: u32) -> String {
    format!("[U+{cp:04X}]")
}

/// Text pieces are already cleaned by Rust's display rules; anything that still shows nothing is written as its
/// code, so the saved file holds no hidden character at all (a tab stays a tab).
fn clean(t: &str) -> String {
    t.chars()
        .map(|c| {
            if c == '\n' || c == '\t' || !(c.is_control() || steplet_core::display::is_invisible(c)) {
                c.to_string()
            } else {
                code(c as u32)
            }
        })
        .collect()
}
