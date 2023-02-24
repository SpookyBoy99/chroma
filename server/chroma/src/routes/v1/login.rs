use actix_web::web;
use serde::Deserialize;
use dal::database::{OAuthAccess, User};
use crate::koala::MemberInfo;
use crate::routes::appdata::WebData;
use crate::routes::error::{Error, WebResult};
use crate::routes::redirect::Redirect;

#[derive(Debug, Deserialize)]
pub struct Query {
    /// The OAuth code generated by Koala
    code: String
}

/// Complete the Koala login flow.
/// The provided code must be generated by Koala.
///
/// Creates a session for the requesting user.
///
/// # Errors
///
/// - If the provided `code` isn't valid
/// - If something went wrong
pub async fn login(data: WebData, query: web::Query<Query>) -> WebResult<Redirect> {
    // Complete the OAuth2 flow by exchanging the code for a token pair
    let oauth_tokens = crate::koala::exchange_code(&data.config, &query.code).await
        .map_err(|e| Error::Koala(e))?;

    // Exctract the member info and determine if the user is an admin
    let (member, is_admin) = match oauth_tokens.member {
        MemberInfo::Member(m) => (m, false),
        MemberInfo::Admin(m) => (m, true),
    };

    let expires_at = oauth_tokens.created_at - oauth_tokens.expires_in;
    let user = match User::get_by_id(&data.db, member.id).await? {
        Some(mut u) => {
            // Update the tokens for this user
            u.set_tokens(oauth_tokens.access_token, oauth_tokens.refresh_token, expires_at).await?;
            u
        },
        // No user exists yet, create one
        None => User::create(&data.db, member.id, OAuthAccess {
            access_token: oauth_tokens.access_token,
            refresh_token: oauth_tokens.refresh_token,
            expires_at,
        }, is_admin).await?
    };

    let session_id = user.create_session().await?;
    let redirect_to = format!("{}?session_id={}&is_admin={}", data.config.login_complete_redirect_uri, session_id, is_admin);
    Ok(Redirect::new(redirect_to))
}