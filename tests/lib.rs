#![deny(warnings)]

#[cfg(test)]
mod tests {
    use requeasy::get;

    #[test]
    fn it_works() {
        let  url = "https://dummyjson.com/products";
        let map = get(&url);
        println!("{:?}", map.body);
        println!("{:?}", map.header);
        assert!(!map.body.is_empty());
        assert!(!map.header.is_empty());
    }
}
