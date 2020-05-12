use chrono::{DateTime, Utc};
use oauth2::reqwest::http_client;
use oauth2::{RefreshToken, TokenResponse};

use std::convert::TryFrom;

use crate::oauth;

pub struct OAuthToken {
    token: String,
    expiration: DateTime<Utc>,
    refresh_token: String,
}

impl TryFrom<oauth::TokenResponse> for OAuthToken {
    type Error = &'static str;

    fn try_from(token_response: oauth::TokenResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            token: token_response.access_token().secret().to_string(),
            expiration: Utc::now()
                .checked_add_signed(
                    chrono::Duration::from_std(token_response.expires_in().ok_or(
                        "Spotify authorization code flow should always return expiration period!",
                    )?)
                    .map_err(|_| "Could not interpret expiration period")?,
                )
                .ok_or("Could not interpret expiration time")?,
            refresh_token: token_response
                .refresh_token()
                .ok_or("Spotify authorization code flow should always return refresh token!")?
                .secret()
                .to_string(),
        })
    }
}

impl OAuthToken {
    pub fn token_checked(&mut self) -> Result<String, &'static str> {
        if Utc::now() >= self.expiration {
            return self.refresh().map(|_| self.token.clone());
        }

        return Ok(self.token.clone());
    }

    fn refresh(&mut self) -> Result<(), &'static str> {
        let client = oauth::create_client();

        client
            .exchange_refresh_token(&RefreshToken::new(self.refresh_token.clone()))
            .request(http_client)
            .map_err(|_| "Error in refresh token request")
            .and_then(|token_response| {
                Self::try_from(token_response).map(|oauth_token| {
                    self.token = oauth_token.token;
                    self.expiration = oauth_token.expiration;
                    self.refresh_token = oauth_token.refresh_token;
                })
            })
    }
}

impl oauth::UserState {
    pub fn token_checked(&mut self) -> Result<String, &'static str> {
        match self {
            oauth::UserState::FinishedAuth(oauth_token) => oauth_token.token_checked(),
            _ => Err("User has not finished OAuth"),
        }
    }
}
