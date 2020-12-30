
#[derive(Deserialize, Serialize)]
pub struct User {
    id: Uuid,
    name: String,
    password: String,
}
