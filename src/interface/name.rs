use crate::interface::sample::Sample;

pub type DynSampleNamerTrait = Box<dyn SampleNamerTrait>;

pub trait SampleNamerTrait: Fn(&Sample, &Context, usize) -> String + Send + Sync {}

impl<T: Fn(&Sample, &Context, usize) -> String + Send + Sync> SampleNamerTrait for T {}

/// Provide context about the ripping process.
///
/// Should be used to make naming samples consistent.
pub struct Context<'a> {
    /// Total samples
    pub total: usize,

    /// File extension of audio format
    pub extension: &'a str,

    /// Highest sample index
    pub highest: usize,
}

impl<'a> Context<'a> {
    pub fn new(total: usize, extension: &'a str, highest: usize) -> Self {
        Self {
            total,
            extension,
            highest,
        }
    }
}

/// Struct to customise how samples are named
#[derive(Debug, Clone, Copy)]
pub struct SampleNamer {
    /// Only name samples with an index
    pub index_only: bool,

    /// Minimum amount of zeros to pad the index.
    ///
    /// If this value is less than the number of digits in the total,
    /// it will fallback to that.
    pub index_padding: u8,

    /// Sample index will match its internal position
    pub index_raw: bool,

    /// Prefer using the filename.
    /// Will fallback to ``name`` if ``filename`` is ``None``
    pub prefer_filename: bool,

    /// Name samples in lower case
    pub lower: bool,

    /// Name samples in upper case
    pub upper: bool,
}

impl Default for SampleNamer {
    fn default() -> Self {
        Self {
            index_only: false,
            index_padding: 2,
            index_raw: false,
            lower: false,
            upper: false,
            prefer_filename: false,
        }
    }
}

impl From<SampleNamer> for Box<dyn SampleNamerTrait> {
    fn from(val: SampleNamer) -> Self {
        Box::new(val.to_func())
    }
}

impl SampleNamer {
    pub fn to_func(self) -> impl SampleNamerTrait {
        move |smp: &Sample, ctx: &Context, index: usize| -> String {
            let index_component = {
                let (index, largest) = match self.index_raw {
                    true => (smp.index_raw(), ctx.highest),
                    false => (index + 1, ctx.total),
                };

                let total = largest;
                let padding = match self.index_padding {
                    n if n > 1 && digits(total) > n => digits(total),
                    n => n,
                } as usize;

                format!("{index:0padding$}",)
            };

            let extension = ctx.extension.trim_matches('.');

            let smp_name = || {
                let name = match self.prefer_filename {
                    true => smp.filename_pretty(),
                    false => smp.name_pretty(),
                };

                match name {
                    name if name.is_empty() => name.into(),
                    name => {
                        let name = name
                            .trim_end_matches(&format!(".{extension}"))
                            .replace('.', "_");

                        let name = match (self.upper, self.lower) {
                            (true, false) => name.to_ascii_uppercase(),
                            (false, true) => name.to_ascii_lowercase(),
                            _ => name,
                        };
                        format!(" - {name}")
                    }
                }
            };

            let name_component = match self.index_only {
                true => "".to_string(),
                false => smp_name(),
            };

            format!("{index_component}{name_component}.{extension}")
        }
    }
}

/// Calculate the number of digits for a given ``usize``
///
/// panics for values over 999 as it is unlikely for a module to contain 1000 samples.
fn digits(n: usize) -> u8 {
    match n {
        n if n < 10 => 1,
        n if n < 100 => 2,
        n if n < 1_000 => 3,
        _ => unimplemented!(),
    }
}
