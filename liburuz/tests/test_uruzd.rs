use futures::join;
use liburuz::api::v1::{Action, ModelConfig, ModelConfigure, ModelCreate, RuneConfigure};
use liburuz::client::api::v1::Client;
use liburuz::clouds::Cloud;
use liburuz::rune::v1::Rune;
use liburuz::server::{config::Config, start};
use std::collections::HashMap;
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
    rt.block_on(async { join!(test_model_config(), test_runes()) });
}

async fn test_model_config() {
    let client = Client::new(URL);
    let model = client
        .create_model(&ModelCreate {
            name: "test-model-config".into(),
            cloud: Cloud::Dummy,
        })
        .await
        .unwrap();

    // Configure model
    // Starts off with the default, then ensure we change it
    assert_eq!(model.state.config, ModelConfig { foo: None });

    let model = client
        .configure_model_wait(
            &model.id,
            &ModelConfigure {
                foo: Some("bar".into()),
            },
        )
        .await
        .unwrap();

    assert_eq!(
        model.state.config,
        ModelConfig {
            foo: Some("bar".into())
        }
    );

    // Test two simultaneous-ish requests
    client
        .configure_model(
            &model.id,
            &ModelConfigure {
                foo: Some("baz1".into()),
            },
        )
        .await
        .unwrap();
    let action_id = client
        .configure_model(
            &model.id,
            &ModelConfigure {
                foo: Some("baz2".into()),
            },
        )
        .await
        .unwrap();
    client.wait_for_action(&model.id, action_id).await.unwrap();
    let model = client.get_model(&model.id).await.unwrap();
    assert_eq!(
        model.state.config,
        ModelConfig {
            foo: Some("baz2".into())
        }
    );
    assert_eq!(
        model.requests.iter().map(|r| &r.action).collect::<Vec<_>>(),
        vec![
            &Action::ConfigureModel {
                foo: Some("bar".into())
            },
            &Action::ConfigureModel {
                foo: Some("baz1".into())
            },
            &Action::ConfigureModel {
                foo: Some("baz2".into())
            }
        ]
    );
    client.destroy_model_wait(&model.id).await.unwrap();

    // After the model's destroyed, can't change it
    assert!(client
        .configure_model_wait(
            &model.id,
            &ModelConfigure {
                foo: Some("baz".into())
            }
        )
        .await
        .is_err());
}

async fn test_runes() {
    let client = Client::new(URL);
    let model = client
        .create_model(&ModelCreate {
            name: "test-runes".into(),
            cloud: Cloud::Dummy,
        })
        .await
        .unwrap();
    let rune = Rune::load("../example-runes/mariadb/").unwrap();
    client
        .add_rune_wait(&model.id, "mariadb", &rune)
        .await
        .unwrap();

    let model = client.get_model(&model.id).await.unwrap();
    let mariadb = model.state.runes.get("mariadb").unwrap();
    let mut expected: HashMap<String, Option<String>> = HashMap::new();
    expected.insert("database".into(), Some("mysql-db".into()));
    expected.insert("user".into(), Some("mysql-user".into()));
    expected.insert("password".into(), None);
    expected.insert("root-password".into(), None);
    assert_eq!(mariadb.state, expected);

    client
        .configure_rune_wait(
            &model.id,
            "mariadb",
            &RuneConfigure {
                attribute: "password".into(),
                value: "password".into(),
            },
        )
        .await
        .unwrap();

    let model = client.get_model(&model.id).await.unwrap();
    let mariadb = model.state.runes.get("mariadb").unwrap();
    assert_eq!(
        mariadb.state.get("password").unwrap(),
        &Some("password".into())
    );
}
