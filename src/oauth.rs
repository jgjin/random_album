use chashmap::CHashMap;
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    RedirectUrl, Scope, StandardTokenResponse, TokenUrl,
};
use rocket::http::{Cookie, Cookies, RawStr, SameSite};
use rocket::response::Redirect;
use rocket::State;

use std::convert::TryFrom;
use std::env;

use crate::oauth_token::OAuthToken;

pub type TokenResponse = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

pub enum UserState {
    NeedsUserAuth(BasicClient),
    FinishedAuth(OAuthToken),
}

#[get("/")]
pub fn begin_oauth(oauth_tokens: State<CHashMap<String, UserState>>) -> Redirect {
    let client = create_client();

    let (auth_url, temp_username) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user-library-read".to_string()))
        .url();

    oauth_tokens.insert(
        temp_username.secret().to_string(),
        UserState::NeedsUserAuth(client),
    );

    return Redirect::found(auth_url.into_string());
}

pub fn create_client() -> BasicClient {
    BasicClient::new(
        ClientId::new(env::var("CLIENT_ID").expect("CLIENT_ID env var")),
        Some(ClientSecret::new(
            env::var("CLIENT_SECRET").expect("CLIENT_SECRET env var"),
        )),
        AuthUrl::new("https://accounts.spotify.com/authorize".to_string()).expect("AuthUrl"),
        Some(
            TokenUrl::new("https://accounts.spotify.com/api/token".to_string()).expect("TokenUrl"),
        ),
    )
    .set_redirect_url(
        RedirectUrl::new(env::var("REDIRECT").expect("REDIRECT env var")).expect("RedirectUrl"),
    )
}

#[get("/oauth_callback?<code>&<state>", rank = 1)]
pub fn end_oauth_accept(
    code: &RawStr,
    state: &RawStr,
    oauth_tokens: State<CHashMap<String, UserState>>,
    mut cookies: Cookies,
) -> Redirect {
    let temp_username = state.url_decode_lossy();

    let redirect_url = oauth_tokens
        .get_mut(&temp_username)
        .map(|mut user_state| match &*user_state {
            UserState::NeedsUserAuth(client) => client
                .exchange_code(AuthorizationCode::new(code.url_decode_lossy()))
                .request(http_client)
                .ok()
                .and_then(|token_response| {
                    OAuthToken::try_from(token_response)
                        .ok()
                        .map(|oauth_token| {
                            *user_state = UserState::FinishedAuth(oauth_token);

                            let mut cookie = Cookie::new("temp_username", temp_username.clone());
                            cookie.set_same_site(SameSite::Lax);
                            cookies.add_private(cookie);

                            "/random_album"
                        })
                })
                .unwrap_or_else(|| {
                    oauth_tokens.remove(&temp_username);

                    "/error"
                }),
            _ => "/error",
        })
        .unwrap_or("/error");

    return Redirect::found(redirect_url);
}

#[get("/oauth_callback?<error>&<state>", rank = 2)]
pub fn end_oauth_deny(
    error: &RawStr,
    state: &RawStr,
    oauth_tokens: State<CHashMap<String, UserState>>,
) -> Redirect {
    let temp_username = state.url_decode_lossy();
    oauth_tokens.remove(&temp_username);

    return Redirect::found(format!("/error?reason={}", error));
}
