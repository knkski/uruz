use libruned::config::Config;
use libruned::start;
use tokio::runtime::Runtime;

static URL: &'static str = "http://localhost:8000";

#[test]
fn test_main() {
    let tempdir = tempfile::tempdir().unwrap();
    let tempdir = tempdir.path().to_str().unwrap().to_string();
    let config = Config {
        database_path: tempdir,
        ..Default::default()
    };
    let mut rt = Runtime::new().unwrap();
    rt.spawn(start(config));
    use std::thread::sleep;
    use std::time::Duration;
    sleep(Duration::from_secs(1));
    let models = rt.block_on(async {
        reqwest::get(&format!("{}/api/v1/models", URL))
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    });
    assert_eq!(models, "[0]");
}
