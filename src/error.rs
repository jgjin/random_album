#[get("/error")]
pub fn error() -> String {
    "Oh no! An error happened (probably in getting Spotify user data)!".to_string()
}
