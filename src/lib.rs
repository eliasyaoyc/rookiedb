#[allow(dead_code)]
mod concurrency;
mod datatypes;
mod index;
mod memory;
pub(crate) mod query;
mod table;
mod utils;

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
