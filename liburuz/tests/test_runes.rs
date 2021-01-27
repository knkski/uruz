use liburuz::rune::v1::Rune;

#[test]
fn parse_rune() {
    let runes = ["mariadb", "pipelines-api", "pipelines-ui"];

    for rune in &runes {
        let loaded = Rune::load(&format!("../example-runes/{}/", rune)).unwrap();
        let zipped = loaded.zip().unwrap();
        let unzipped = Rune::unzip(&zipped).unwrap();
        assert_eq!(loaded, unzipped);
    }
}
