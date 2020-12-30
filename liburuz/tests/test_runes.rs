use liburuz::Rune;

#[test]
fn parse_rune() {
    Rune::load("../example-runes/mariadb/").expect("Couldn't load mariadb rune");
    Rune::load("../example-runes/pipelines-api/").expect("Couldn't load pipelines-api rune");
    Rune::load("../example-runes/pipelines-ui/").expect("Couldn't load pipelines-ui rune");
}
