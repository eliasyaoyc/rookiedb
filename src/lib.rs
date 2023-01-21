#[allow(dead_code)]
mod datatypes;
pub mod error;
mod index;
mod memory;
mod page;
pub(crate) mod query;
mod table;
mod utils;
mod database;
mod catalog;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
