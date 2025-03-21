#[cfg(test)]
mod content;
#[cfg(test)]
mod protocol;

#[cfg(test)]
pub(crate) use content::*;
#[cfg(test)]
pub(crate) use protocol::*;

#[cfg(test)]
mod test;

fn main() {}
