pub mod decoder;
mod error;
mod metadata;

#[cfg(test)]
extern crate alloc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
