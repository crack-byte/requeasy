#![deny(warnings)]

#[cfg(test)]
mod tests {
    use requeasy::get;

    #[test]
    fn get_test() {
        let  url = "https://dummyjson.com/products";
        let response = get(&url);
        assert!(!response.body.is_empty());
        assert!(!response.header.is_empty());
    }
    #[test]
    fn post_test() {
        let  url = "https://reqres.in/api/users";
        let body = "{\"name\":\"morpheus\",\"job\":\"leader\"}";
        let response = requeasy::post(&url, body);
        println!("{:?}", response);
        assert!(!response.body.is_empty());
        assert!(!response.header.is_empty());
    }
}
