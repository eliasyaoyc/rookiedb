#[allow(dead_code)]
pub(crate) mod common;
mod datatypes;
mod index;
mod memory;
pub(crate) mod query;
mod table;

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
