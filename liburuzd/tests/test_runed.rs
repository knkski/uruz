use liburuzd::{start, config::Config};
use liburuz::api::v1::{Client, ModelConfig, ModelConfigure, ModelCreate};
use std::thread::sleep;
use std::time::Duration;
use tokio::runtime::Runtime;

static URL: &'static str = "http://localhost:8000";

#[test]
fn test_main() {
    let tempdir = tempfile::tempdir().unwrap();
    let tempdir = tempdir.path().to_str().unwrap().to_string();
    let config = Config {
        database_path: tempdir,
        api_host: [0, 0, 0, 0],
        api_port: 8000,
    };
    let mut rt = Runtime::new().unwrap();

    // Start server
    rt.spawn(start(config));

    // Wait a bit for server to boot up before running tests
    sleep(Duration::from_secs(1));

    // Run tests
    rt.block_on(async {
        let client = Client::new(URL);
        let model = client
            .create_model(&ModelCreate {
                name: "test-model".into(),
                cloud: "dummy".into(),
            })
            .await
            .unwrap();

        // Configure model
        // Starts off with the default, then ensure we change it
        assert_eq!(model.state.config, ModelConfig { foo: "bar".into() });

        let model = client
            .configure_model_wait(&model.id, &ModelConfigure { foo: "baz".into() })
            .await
            .unwrap();

        assert_eq!(model.state.config, ModelConfig { foo: "baz".into() });
    });
}
