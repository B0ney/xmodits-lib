// use crate::interface::Sample;

use crate::interface::sample::Sample;

/// Configure how exported samples are named
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
pub struct Info {
    total_samples: usize,
    extension: String,
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

// impl From<SampleNamer> for Box<dyn Fn(&Sample, usize, &str) -> String> {
//     fn from(val: SampleNamer) -> Self {
//         Box::new(val.to_func())
//     }
// }

impl SampleNamer {
    pub fn to_func(self) -> impl Fn(&Sample, &Info, usize) -> String {
        move |smp: &Sample, info: &Info, index: usize| -> String {
            let index_component = {
                let index = match self.index_raw {
                    true => smp.index_raw(),
                    false => index + 1,
                };
                // BUG: There is a potential loophole for using index_raw.
                //
                // Let's say we have 3 samples, the first two have a raw index of 1 and 4, but the last sample has a raw index of 100.
                // In this case, we do not have enough information to enforce a consistant padding for all samples.
                // We could add another parameter just like "total" to make sure this doesn't happen, but is doing so worth it?
                // Instead of &Sample, we *could* use &[Sample], that way we have enough information. but the problem arises
                // when &[sample] has gaps in beteween them, index_pretty will no longer work.
                let total = info.total_samples;
                let padding = match self.index_padding {
                    n if n > 1 && digits(total) > n => digits(total),
                    n => n,
                } as usize;

                format!("{index:0padding$}",)
            };
            let extension = &info.extension;
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
                            // .trim_end_matches(&format!(".{}", extension.to_uppercase()))
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
        // n if n < 10_000 => 4,
        // n => calc_digits(n),
        _ => unimplemented!(),
    }
}

// #[cold]
// /// Calculate the number of digits for a usize.
// ///
// /// Unlikely to be called
// fn calc_digits(n: usize) -> usize {
//     dbg!((n as f32).log10().floor() as usize + 1)
// }
