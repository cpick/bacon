use {
    crate::*,
    anyhow::*,
    crossterm::{
        style::{Colorize, Styler},
    },
    std::io::Write,
};

/// either Warning or Error
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineType {
    /// the start of either an error or a warning
    Title(Kind),

    /// a line locating the problem
    Location,

    /// this line marks the end of the interesting content
    End,

    /// any other line
    Normal,
}

impl LineType {
    pub fn cols(self) -> usize {
        match self {
            Self::Title(_) => 3,
            _ => 0,
        }
    }
    pub fn draw(self, w: &mut W, item_idx:usize) -> Result<()> {
        match self {
            Self::Title(Kind::Error) => {
                write!(w, "{}", format!("{:^3}", item_idx).black().bold().on_red())?;
            }
            Self::Title(Kind::Warning) => {
                write!(w, "{}", format!("{:^3}", item_idx).black().bold().on_yellow())?;
            }
            _ => {}
        }
        Ok(())
    }
}

fn is_spaces(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_whitespace())
}

/// check if the string starts with something like "15 warnings emitted"
fn is_n_warnings_emitted(s: &str) -> bool {
    let mut tokens = s.split_ascii_whitespace();
    if let Some(t) = tokens.next() {
        if t.parse::<usize>().is_err() {
            return false;
        }
        if let Some(t) = tokens.next() {
            if t != "warnings" && t != "warning" {
                return false;
            }
            if let Some(t) = tokens.next() {
                if t.starts_with("emitted") {
                    return true;
                }
            }
        }
    }
    false
}

impl From<&TLine> for LineType {
    fn from(content: &TLine) -> Self {
        if let (Some(ts1), Some(ts2)) = (content.strings.get(0), content.strings.get(1)) {
            match (ts1.csi.as_ref(), ts1.raw.as_ref(), ts2.csi.as_ref(), ts2.raw.as_ref()) {
                (crate::CSI_BOLD_RED, "error", CSI_BOLD, r2) if r2.starts_with(": aborting due to") => {
                    LineType::End
                }
                (crate::CSI_BOLD_RED, "error", CSI_BOLD, _) => LineType::Title(Kind::Error),
                (crate::CSI_BOLD_YELLOW, "warning", _, r2) if is_n_warnings_emitted(&r2) => {
                    LineType::End
                }
                (crate::CSI_BOLD_YELLOW, "warning", _, _) => LineType::Title(Kind::Warning),
                ("", r1, crate::CSI_BOLD_BLUE, "--> ") if is_spaces(r1) => LineType::Location,
                _ => LineType::Normal,
            }
        } else {
            LineType::Normal // empty line
        }
    }
}

