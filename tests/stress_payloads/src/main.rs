#[cfg(test)]
mod a;
#[cfg(test)]
mod b;
#[cfg(test)]
mod c;
#[cfg(test)]
mod test;

#[cfg(test)]
pub(crate) use a::*;
#[cfg(test)]
pub(crate) use b::*;
#[cfg(test)]
pub(crate) use c::*;

fn main() {}
