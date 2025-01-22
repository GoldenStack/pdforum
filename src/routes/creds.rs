use std::fmt::{self, Display};

use axum::{
    extract::Path,
    response::{IntoResponse, Redirect},
    Extension,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::{database, render::PDF, Context};

use super::{render_into, Auth, Page, Return, AUTH, COMMON_STR, KEYBOARD_STR};

const CREDENTIALS_STR: &str = include_str!("../../templates/credentials.typ");

static CREDENTIALS: Page = Page::new(|| {
    PDF::make(
        "credentials.typ",
        [
            ("credentials.typ", CREDENTIALS_STR),
            ("common.typ", COMMON_STR),
            ("keyboard.typ", KEYBOARD_STR),
        ],
    )
});

const REGISTRATION: &str = "register";
const LOGIN: &str = "login";

#[derive(Debug, Default, Deserialize, Serialize)]
struct Credentials {
    field: CredentialsField,
    username: String,
    password: String,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
enum CredentialsField {
    #[default]
    Username,
    Password,
}

impl Credentials {
    fn next(&mut self) -> bool {
        if self.field == CredentialsField::Username {
            self.field = CredentialsField::Password;
            true
        } else {
            false
        }
    }

    fn into_data(self) -> String {
        format!("{}\u{0}{}", self.username, self.password)
    }

    fn field_mut(&mut self) -> &mut String {
        match self.field {
            CredentialsField::Username => &mut self.username,
            CredentialsField::Password => &mut self.password,
        }
    }
}

impl Display for CredentialsField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialsField::Username => write!(f, "username"),
            CredentialsField::Password => write!(f, "password"),
        }
    }
}

pub async fn register(
    ctx: Extension<Context>,
    session: Session,
    Path(suffix): Path<String>,
) -> Return {
    let mut register = session
        .get::<Credentials>(REGISTRATION)
        .await
        .map(Option::unwrap_or_default)?;

    if suffix == "next" {
        if !register.next() {
            match database::register(&ctx.db, &register.username, &register.password).await? {
                Some(id) => {
                    let register = session
                        .remove::<Credentials>(REGISTRATION)
                        .await
                        .map(Option::unwrap_or_default)?;

                    session
                        .insert(
                            AUTH,
                            Auth {
                                id,
                                username: register.username,
                            },
                        )
                        .await?;
                    return Ok(Redirect::temporary("/").into_response());
                }
                None => {
                    todo!()
                }
            }
        }
    } else if suffix.len() == 1 {
        register.field_mut().push_str(suffix.as_str());
    } else {
        register = Credentials::default();
    }

    session.insert(REGISTRATION, &register).await?;

    let auth = session.get::<Auth>(AUTH).await.ok().flatten().is_some();

    let mut page = CREDENTIALS.lock();

    let data = format!(
        r#"url: {}
type: "register"
auth: {auth}
field: {}"#,
        ctx.base_url, register.field
    );

    page.write("info.yml", data);

    render_into(&mut page, register.into_data())
}

pub async fn register_empty(ctx: Extension<Context>, session: Session) -> Return {
    register(ctx, session, Path(String::new())).await
}

pub async fn login(
    ctx: Extension<Context>,
    session: Session,
    Path(suffix): Path<String>,
) -> Return {
    let mut register = session
        .get::<Credentials>(LOGIN)
        .await
        .map(Option::unwrap_or_default)?;

    if suffix == "next" {
        if !register.next() {
            match database::login(&ctx.db, &register.username, &register.password).await? {
                Some(id) => {
                    let register = session
                        .remove::<Credentials>(LOGIN)
                        .await
                        .map(Option::unwrap_or_default)?;

                    session
                        .insert(
                            AUTH,
                            Auth {
                                id,
                                username: register.username,
                            },
                        )
                        .await
                        .unwrap();
                    return Ok(Redirect::temporary("/").into_response());
                }
                None => {
                    todo!()
                }
            }
        }
    } else if suffix.len() == 1 {
        register.field_mut().push_str(suffix.as_str());
    } else {
        register = Credentials::default();
    }

    session.insert(LOGIN, &register).await?;

    let auth = session.get::<Auth>(AUTH).await.ok().flatten().is_some();

    let mut page = CREDENTIALS.lock();

    let data = format!(
        r#"url: {}
type: "login"
auth: {auth}
field: {}"#,
        ctx.base_url, register.field
    );

    page.write("info.yml", data);

    render_into(&mut page, register.into_data())
}

pub async fn login_empty(ctx: Extension<Context>, session: Session) -> Return {
    login(ctx, session, Path(String::new())).await
}

pub async fn logout(session: Session) -> Return {
    session.remove::<Auth>(AUTH).await?;

    Ok(Redirect::temporary("/").into_response())
}