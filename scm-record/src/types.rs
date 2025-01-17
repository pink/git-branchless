//! Data types for the change selector interface.

use std::borrow::Cow;
use std::fmt::Display;
use std::io;
use std::num::TryFromIntError;
use std::path::Path;

use thiserror::Error;

/// The state used to render the changes. This is passed into [`Recorder::new`]
/// and then updated and returned with [`Recorder::run`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RecordState<'a> {
    /// The state of each file. This is rendered in order, so you may want to
    /// sort this list by path before providing it.
    pub files: Vec<File<'a>>,
}

/// An error which occurred when attempting to record changes.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum RecordError {
    /// The user cancelled the operation.
    #[error("cancelled by user")]
    Cancelled,

    #[error("failed to set up terminal: {0}")]
    SetUpTerminal(#[source] io::Error),

    #[error("failed to clean up terminal: {0}")]
    CleanUpTerminal(#[source] io::Error),

    #[error("failed to render new frame: {0}")]
    RenderFrame(#[source] io::Error),

    #[error("failed to read user input: {0}")]
    ReadInput(#[source] crossterm::ErrorKind),

    #[error("failed to serialize JSON: {0}")]
    SerializeJson(#[source] serde_json::Error),

    #[error("failed to wrote file: {0}")]
    WriteFile(#[source] io::Error),

    #[error("bug: {0}")]
    Bug(String),
}

/// The Unix file mode. The special mode `0` indicates that the file did not exist.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FileMode(pub usize);

impl FileMode {
    /// Get the file mode corresponding to an absent file. This typically
    /// indicates that the file was created (if the "before" mode) or deleted
    /// (if the "after" mode).
    pub fn absent() -> Self {
        Self(0)
    }
}

impl Display for FileMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(mode) = self;
        write!(f, "{mode:o}")
    }
}

impl From<usize> for FileMode {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<FileMode> for usize {
    fn from(value: FileMode) -> Self {
        let FileMode(value) = value;
        value
    }
}

impl TryFrom<u32> for FileMode {
    type Error = TryFromIntError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl TryFrom<FileMode> for u32 {
    type Error = TryFromIntError;

    fn try_from(value: FileMode) -> Result<Self, Self::Error> {
        let FileMode(value) = value;
        value.try_into()
    }
}

impl TryFrom<i32> for FileMode {
    type Error = TryFromIntError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl TryFrom<FileMode> for i32 {
    type Error = TryFromIntError;

    fn try_from(value: FileMode) -> Result<Self, Self::Error> {
        let FileMode(value) = value;
        value.try_into()
    }
}

/// The state of a file to be recorded.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct File<'a> {
    /// The path to the file.
    pub path: Cow<'a, Path>,

    /// The Unix file mode of the file (before any changes), if available. This
    /// may be rendered by the UI.
    ///
    /// This value is not directly modified by the UI; instead, construct a
    /// [`Section::FileMode`] and use the [`FileState::get_file_mode`] function
    /// to read a user-provided updated to the file mode function to read a
    /// user-provided updated to the file mode.
    pub file_mode: Option<FileMode>,

    /// The set of [`Section`]s inside the file.
    pub sections: Vec<Section<'a>>,
}

/// The contents of a file selected as part of the record operation.
#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum SelectedContents<'a> {
    /// The file didn't exist or was deleted.
    Absent,

    /// The file contents have not changed.
    Unchanged,

    /// The file contains binary contents.
    Binary {
        /// The UI description of the old version of the file.
        old_description: Option<Cow<'a, str>>,
        /// The UI description of the new version of the file.
        new_description: Option<Cow<'a, str>>,
    },

    /// The file contained the following text contents.
    Present {
        /// The contents of the file.
        contents: String,
    },
}

impl SelectedContents<'_> {
    fn push_str(&mut self, s: &str) {
        match self {
            SelectedContents::Absent | SelectedContents::Unchanged => {
                *self = SelectedContents::Present {
                    contents: s.to_owned(),
                };
            }
            SelectedContents::Binary {
                old_description: _,
                new_description: _,
            } => {
                // Do nothing.
            }
            SelectedContents::Present { contents } => {
                contents.push_str(s);
            }
        }
    }
}

impl File<'_> {
    /// Get the new Unix file mode. If the user selected a
    /// [`Section::FileMode`], then returns that file mode. Otherwise, returns
    /// the `file_mode` value that this [`FileState`] was constructed with.
    pub fn get_file_mode(&self) -> Option<FileMode> {
        let Self {
            path: _,
            file_mode,
            sections,
        } = self;
        sections
            .iter()
            .find_map(|section| match section {
                Section::Unchanged { .. }
                | Section::Changed { .. }
                | Section::FileMode {
                    is_toggled: false,
                    before: _,
                    after: _,
                }
                | Section::Binary { .. } => None,

                Section::FileMode {
                    is_toggled: true,
                    before: _,
                    after,
                } => Some(*after),
            })
            .or(*file_mode)
    }

    /// Calculate the `(selected, unselected)` contents of the file. For
    /// example, the first value would be suitable for staging or committing,
    /// and the second value would be suitable for potentially recording again.
    pub fn get_selected_contents(&self) -> (SelectedContents, SelectedContents) {
        let mut acc_selected = SelectedContents::Unchanged;
        let mut acc_unselected = SelectedContents::Unchanged;
        let Self {
            path: _,
            file_mode: _,
            sections,
        } = self;
        for section in sections {
            match section {
                Section::Unchanged { lines } => {
                    for line in lines {
                        acc_selected.push_str(line);
                        acc_unselected.push_str(line);
                    }
                }

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
                            }
                            (ChangeType::Added, false) | (ChangeType::Removed, true) => {
                                acc_unselected.push_str(line);
                            }
                        }
                    }
                }

                Section::FileMode {
                    is_toggled,
                    before,
                    after,
                } => {
                    if *is_toggled && after == &FileMode::absent() {
                        acc_selected = SelectedContents::Absent;
                    } else if !is_toggled && before == &FileMode::absent() {
                        acc_unselected = SelectedContents::Absent;
                    }
                }

                Section::Binary {
                    is_toggled,
                    old_description,
                    new_description,
                } => {
                    let selected_contents = SelectedContents::Binary {
                        old_description: old_description.clone(),
                        new_description: new_description.clone(),
                    };
                    if *is_toggled {
                        acc_selected = selected_contents;
                    } else {
                        acc_unselected = selected_contents;
                    }
                }
            }
        }
        (acc_selected, acc_unselected)
    }
}

/// A section of a file to be rendered and recorded.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Section<'a> {
    /// This section of the file is unchanged and just used for context.
    ///
    /// By default, only part of the context will be shown. However, all of the
    /// context lines should be provided so that they can be used to globally
    /// number the lines correctly.
    Unchanged {
        /// The contents of the lines, including their trailing newline
        /// character(s), if any.
        lines: Vec<Cow<'a, str>>,
    },

    /// This section of the file is changed, and the user needs to select which
    /// specific changed lines to record.
    Changed {
        /// The contents of the lines, including their trailing newline
        /// character(s), if any.
        lines: Vec<SectionChangedLine<'a>>,
    },

    /// This indicates that the Unix file mode of the file changed, and that the
    /// user needs to accept that mode change or not. This is not part of the
    /// "contents" of the file per se, but it's rendered inline as if it were.
    FileMode {
        /// Whether or not the file mode change was accepted.
        is_toggled: bool,

        /// The old file mode.
        before: FileMode,

        /// The new file mode.
        after: FileMode,
    },

    /// This file contains binary contents.
    Binary {
        /// Whether or not the binary contents change was accepted.
        is_toggled: bool,

        /// The description of the old binary contents, for use in the UI only.
        old_description: Option<Cow<'a, str>>,

        /// The description of the new binary contents, for use in the UI only.
        new_description: Option<Cow<'a, str>>,
    },
}

impl Section<'_> {
    /// Whether or not this section contains user-editable content (as opposed
    /// to simply contextual content).
    pub fn is_editable(&self) -> bool {
        match self {
            Section::Unchanged { .. } => false,
            Section::Changed { .. } | Section::FileMode { .. } | Section::Binary { .. } => true,
        }
    }
}

/// The type of change in the patch/diff.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ChangeType {
    /// The line was added.
    Added,

    /// The line was removed.
    Removed,
}

/// A changed line inside a `Section`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SectionChangedLine<'a> {
    /// Whether or not this line was selected to be recorded.
    pub is_toggled: bool,

    /// The type of change this line was.
    pub change_type: ChangeType,

    /// The contents of the line, including its trailing newline character(s),
    /// if any.
    pub line: Cow<'a, str>,
}
