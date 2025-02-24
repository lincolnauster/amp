mod displayable_path;
pub mod exclusions;

use std::fmt;
use std::path::PathBuf;
use std::slice::Iter;
use bloodhound::ExclusionPattern;
use crate::util::SelectableVec;
use crate::models::application::modes::{SearchSelectMode, SearchSelectConfig};
use crate::models::application::Event;
use std::sync::mpsc::Sender;
use std::thread;
pub use bloodhound::Index;
pub use self::displayable_path::DisplayablePath;

#[derive(PartialEq)]
pub enum OpenModeIndex {
    Complete(Index),
    Indexing(PathBuf)
}

pub struct OpenMode {
    pub insert: bool,
    pub input: String,
    index: OpenModeIndex,
    pub results: SelectableVec<DisplayablePath>,
    config: SearchSelectConfig,
}

impl OpenMode {
    pub fn new(
        path: PathBuf,
        exclusions: Option<Vec<ExclusionPattern>>,
        events: Sender<Event>,
        config: SearchSelectConfig
    ) -> OpenMode {
        // Build and populate the index in a separate thread.
        let index_path = path.clone();
        thread::spawn(move || {
            let mut index = Index::new(index_path);
            index.populate(exclusions, false);
            let _ = events.send(
                Event::OpenModeIndexComplete(index)
            );
        });

        OpenMode {
            insert: true,
            input: String::new(),
            index: OpenModeIndex::Indexing(path),
            results: SelectableVec::new(Vec::new()),
            config,
        }
    }

    pub fn set_index(&mut self, index: Index) {
        self.index = OpenModeIndex::Complete(index)
    }

    fn paths(&self) -> SelectableVec<DisplayablePath> {
        if let OpenModeIndex::Complete(ref idx) = self.index {
            let results: Vec<DisplayablePath> = idx.find(
                    &self.input.to_lowercase(),
                    self.config.max_results,
                ).into_iter()
                .map(|path| DisplayablePath(path.to_path_buf()))
                .collect();

            if results.len() == 0 {
                SelectableVec::new_custom(
                    DisplayablePath(PathBuf::from(self.input.clone()))
                )
            } else {
                SelectableVec::new(results)
            }
        } else {
            SelectableVec::new(vec![])
        }
    }
}

impl fmt::Display for OpenMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OPEN")
    }
}

impl SearchSelectMode<DisplayablePath> for OpenMode {
    fn search(&mut self) {
        self.results = self.paths();
    }

    fn query(&mut self) -> &mut String {
        &mut self.input
    }

    fn insert_mode(&self) -> bool {
        self.insert
    }

    fn set_insert_mode(&mut self, insert_mode: bool) {
        self.insert = insert_mode;
    }

    fn results(&self) -> Iter<DisplayablePath> {
        self.results.iter()
    }

    fn selection(&self) -> Option<&DisplayablePath> {
        self.results.selection()
    }

    fn selected_index(&self) -> Option<usize> {
        self.results.selected_index()
    }

    fn select_previous(&mut self) {
        self.results.select_previous();
    }

    fn select_next(&mut self) {
        self.results.select_next();
    }

    fn config(&self) -> &SearchSelectConfig {
        &self.config
    }

    fn message(&mut self) -> Option<String> {
        if let OpenModeIndex::Indexing(ref path) = self.index {
            Some(format!("Indexing {}", path.to_string_lossy()))
        } else if self.query().is_empty() {
            Some(String::from("Enter a search query to start."))
        } else if self.results().count() == 0 {
            Some(String::from("No matching entries found."))
        } else {
            None
        }
    }
}
