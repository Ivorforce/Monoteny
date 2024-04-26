use std::fmt::{Debug, Display};
use std::fs;
use std::ops::Range;
use std::path::PathBuf;
use annotate_snippets::{Annotation, Level, Message, Renderer, Snippet};
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub level: Level,
    pub title: String,
    pub path: Option<PathBuf>,
    pub range: Option<Range<usize>>,
    pub notes: Vec<Box<RuntimeError>>,
}

pub type RResult<V> = Result<V, Vec<RuntimeError>>;

impl RuntimeError {
    pub fn print(&self) {
        match &self.path {
            None => self.print_snippet(Snippet::source(&self.title)),
            Some(path) => match fs::read_to_string(path) {
                Ok(source) => self.print_snippet(
                    Snippet::source(source.as_str())
                        .origin(path.to_str().unwrap())
                        .fold(true)
                ),
                Err(err) => self.print_snippet(Snippet::source(err.to_string().as_str())),
            }
        };
    }

    /// This could be inline with print, but Snippet doesn't copy its string...
    fn print_snippet(&self, mut snippet: Snippet) {
        if let Some(range) = &self.range {
            snippet = snippet.annotation(
                Level::Error.span(range.clone())
            );
        }

        let mut footers = vec![];
        let mut annotations = vec![];
        for annotation in self.notes.iter() {
            annotation.add_to_snippet(&mut annotations, &mut footers);
        }

        let msg = Level::Error.title(&self.title)
            .snippet(snippet.annotations(annotations))
            .footers(footers.into_iter());

        let renderer = Renderer::styled();

        println!("{}", renderer.render(msg));
    }

    pub fn new(level: Level, title: &str) -> RuntimeError {
        RuntimeError {
            level,
            title: title.to_string(),
            path: None,
            range: None,
            notes: vec![],
        }
    }

    pub fn error(title: &str) -> RuntimeError {
        Self::new(Level::Error, title)
    }

    pub fn warning(title: &str) -> RuntimeError {
        Self::new(Level::Warning, title)
    }

    pub fn note(title: &str) -> RuntimeError {
        Self::new(Level::Note, title)
    }

    pub fn info(title: &str) -> RuntimeError {
        Self::new(Level::Info, title)
    }

    pub fn to_array(self) -> Vec<Self> {
        vec![self]
    }

    pub fn add_to_snippet<'a>(&'a self, annotations: &mut Vec<Annotation<'a>>, footers: &mut Vec<Message<'a>>) {
        let Some(span) = &self.range else {
            let mut our_footers = vec![];

            // If any notes have spans, they aren't childed to us, instead put into the snippet.
            //  ... that's rare, I think it's ok?
            for note in self.notes.iter() {
                note.add_to_snippet(annotations, &mut our_footers);
            }

            // TODO Having nested footers does not seem to show on the error, e.g. with indentation.
            footers.push(
                self.level
                    .title(&self.title)
                    .footers(our_footers.into_iter())
            );

            return
        };

        annotations.push(
            self.level.span(span.clone())
                .label(&self.title)
        )
    }

    pub fn in_range(mut self, range: Range<usize>) -> RuntimeError {
        if self.range.is_some() {
            return self;
        }

        self.range = Some(range);
        self
    }

    pub fn in_file(mut self, path: PathBuf) -> RuntimeError {
        if self.path.is_some() {
            return self;
        }

        self.path = Some(path);
        self
    }

    pub fn with_note(mut self, note: RuntimeError) -> Self {
        self.notes.push(Box::new(note));
        self
    }

    pub fn with_notes(mut self, notes: impl Iterator<Item=RuntimeError>) -> Self {
        self.notes.extend(notes.into_iter().map(Box::new).collect_vec());
        self
    }
}

pub trait ErrInRange<R> {
    fn err_in_range(self, range: &Range<usize>) -> R;
}

impl<V> ErrInRange<RResult<V>> for RResult<V> {
    fn err_in_range(self, range: &Range<usize>) -> RResult<V> {
        self.map_err(|e| e.into_iter().map(|e| e.in_range(range.clone())).collect())
    }
}

pub trait TryCollectMany<R> {
    fn try_collect_many(self) -> RResult<R>;
}

impl<V, I: Iterator<Item=RResult<V>>, R: FromIterator<V>> TryCollectMany<R> for I {
    fn try_collect_many(self) -> RResult<R> {
        let mut values = vec![];
        let mut errors = vec![];

        for result in self {
            match result {
                Ok(result) => values.push(result),
                Err(result) => errors.extend(result),
            }
        }

        return match errors.is_empty() {
            true => Ok(R::from_iter(values)),
            false => Err(errors),
        }
    }
}

pub fn print_errors(errors: &Vec<RuntimeError>) {
    for error in errors.iter() {
        error.print();
        println!("\n");
    }
}

impl Eq for RuntimeError {

}
