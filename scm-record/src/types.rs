use std::io;
use std::path::Path;

use thiserror::Error;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub files: Vec<File<'a>>,
#[allow(missing_docs)]
#[derive(Debug, Error)]
    #[error("cancelled by user")]

    #[error("failed to set up terminal: {0}")]
    SetUpTerminal(#[source] io::Error),

    #[error("failed to render new frame: {0}")]
    RenderFrame(#[source] io::Error),

    #[error("failed to read user input: {0}")]
    ReadInput(#[source] crossterm::ErrorKind),

    #[error("bug: {0}")]
    Bug(String),
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct File<'a> {
    /// The path to the file.
    pub path: Cow<'a, Path>,

impl File<'_> {
            path: _,
            path: _,
                    is_toggled: false,
                    is_toggled: true,
            path: _,
                Section::Unchanged { lines } => {
                    for line in lines {
                        acc_selected.push('\n');
                        acc_unselected.push('\n');
                Section::Changed { lines } => {
                    for line in lines {
                        let SectionChangedLine {
                            is_toggled,
                            change_type,
                            line,
                        } = line;
                        match (change_type, is_toggled) {
                            (ChangeType::Added, true) | (ChangeType::Removed, false) => {
                                acc_selected.push_str(line);
                                acc_selected.push('\n');
                            }
                            (ChangeType::Added, false) | (ChangeType::Removed, true) => {
                                acc_unselected.push_str(line);
                                acc_unselected.push('\n');
                            }
                    is_toggled: _,
#[derive(Clone, Debug, Eq, PartialEq)]
    ///
    /// By default, only part of the context will be shown. However, all of the
    /// context lines should be provided so that they can be used to globally
    /// number the lines correctly.
        /// The contents of the lines in this section. Each line does *not*
        /// include a trailing newline character.
        lines: Vec<Cow<'a, str>>,
        /// The contents of the lines caused by the user change. Each line does
        /// *not* include a trailing newline character.
        lines: Vec<SectionChangedLine<'a>>,
    /// This indicates that the Unix file mode of the file changed, and that the
    /// user needs to accept that mode change or not. This is not part of the
    /// "contents" of the file per se, but it's rendered inline as if it were.
        is_toggled: bool,
impl Section<'_> {
    /// Whether or not this section contains user-editable content (as opposed
    /// to simply contextual content).
    pub fn is_editable(&self) -> bool {
        match self {
            Section::Unchanged { .. } => false,
            Section::Changed { .. } | Section::FileMode { .. } => true,
        }
    }
}

/// The type of change in the patch/diff.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChangeType {
    /// The line was added.
    Added,

    /// The line was removed.
    Removed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
    pub is_toggled: bool,

    /// The type of change this line was.
    pub change_type: ChangeType,