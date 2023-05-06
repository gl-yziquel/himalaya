use pimalaya_email::{
    folder::sync::Strategy as SyncFoldersStrategy, EmailHooks, EmailSender, EmailTextPlainFormat,
    ImapAuthConfig, MaildirConfig, OAuth2Config, OAuth2Method, OAuth2Scopes, SendmailConfig,
    SmtpAuthConfig, SmtpConfig,
};
use pimalaya_keyring::Entry;
use pimalaya_process::Cmd;
use pimalaya_secret::Secret;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};

#[cfg(feature = "imap-backend")]
use pimalaya_email::ImapConfig;

#[cfg(feature = "notmuch-backend")]
use pimalaya_email::NotmuchConfig;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "Cmd", from = "String")]
pub struct CmdDef;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "Entry", from = "String")]
pub struct EntryDef;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "OAuth2Method")]
pub enum OAuth2MethodDef {
    #[serde(rename = "xoauth2", alias = "XOAUTH2")]
    XOAuth2,
    #[serde(rename = "oauthbearer", alias = "OAUTHBEARER")]
    OAuthBearer,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "SmtpConfig")]
struct SmtpConfigDef {
    #[serde(rename = "smtp-host")]
    pub host: String,
    #[serde(rename = "smtp-port")]
    pub port: u16,
    #[serde(rename = "smtp-ssl")]
    pub ssl: Option<bool>,
    #[serde(rename = "smtp-starttls")]
    pub starttls: Option<bool>,
    #[serde(rename = "smtp-insecure")]
    pub insecure: Option<bool>,
    #[serde(rename = "smtp-login")]
    pub login: String,
    #[serde(flatten, with = "SmtpAuthConfigDef")]
    pub auth: SmtpAuthConfig,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "SmtpAuthConfig", tag = "smtp-auth")]
pub enum SmtpAuthConfigDef {
    #[serde(rename = "passwd", alias = "password", with = "SmtpPasswdDef")]
    Passwd(Secret),
    #[serde(rename = "oauth2", with = "SmtpOAuth2ConfigDef")]
    OAuth2(OAuth2Config),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "Secret", rename_all = "kebab-case")]
pub enum SmtpPasswdDef {
    #[serde(rename = "smtp-passwd")]
    Raw(String),
    #[serde(rename = "smtp-passwd-cmd", with = "CmdDef")]
    Cmd(Cmd),
    #[serde(rename = "smtp-passwd-keyring", with = "EntryDef")]
    Keyring(Entry),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "OAuth2Config")]
pub struct SmtpOAuth2ConfigDef {
    #[serde(rename = "smtp-oauth2-method", with = "OAuth2MethodDef")]
    pub method: OAuth2Method,
    #[serde(rename = "smtp-oauth2-client-id")]
    pub client_id: String,
    #[serde(flatten, with = "SmtpOAuth2ClientSecretDef")]
    pub client_secret: Secret,
    #[serde(rename = "smtp-oauth2-auth-url")]
    pub auth_url: String,
    #[serde(rename = "smtp-oauth2-token-url")]
    pub token_url: String,
    #[serde(flatten, with = "SmtpOAuth2AccessTokenDef")]
    pub access_token: Secret,
    #[serde(flatten, with = "SmtpOAuth2RefreshTokenDef")]
    pub refresh_token: Secret,
    #[serde(flatten, with = "SmtpOAuth2ScopesDef")]
    pub scopes: OAuth2Scopes,
    #[serde(rename = "smtp-oauth2-pkce", default)]
    pub pkce: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "Secret")]
pub enum SmtpOAuth2ClientSecretDef {
    #[serde(rename = "smtp-oauth2-client-secret")]
    Raw(String),
    #[serde(rename = "smtp-oauth2-client-secret-cmd", with = "CmdDef")]
    Cmd(Cmd),
    #[serde(rename = "smtp-oauth2-client-secret-keyring", with = "EntryDef")]
    Keyring(Entry),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "Secret")]
pub enum SmtpOAuth2AccessTokenDef {
    #[serde(rename = "smtp-oauth2-access-token")]
    Raw(String),
    #[serde(rename = "smtp-oauth2-access-token-cmd", with = "CmdDef")]
    Cmd(Cmd),
    #[serde(rename = "smtp-oauth2-access-token-keyring", with = "EntryDef")]
    Keyring(Entry),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "Secret")]
pub enum SmtpOAuth2RefreshTokenDef {
    #[serde(rename = "smtp-oauth2-refresh-token")]
    Raw(String),
    #[serde(rename = "smtp-oauth2-refresh-token-cmd", with = "CmdDef")]
    Cmd(Cmd),
    #[serde(rename = "smtp-oauth2-refresh-token-keyring", with = "EntryDef")]
    Keyring(Entry),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "OAuth2Scopes")]
pub enum SmtpOAuth2ScopesDef {
    #[serde(rename = "smtp-oauth2-scope")]
    Scope(String),
    #[serde(rename = "smtp-oauth2-scopes")]
    Scopes(Vec<String>),
}

#[cfg(feature = "imap-backend")]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "ImapConfig")]
pub struct ImapConfigDef {
    #[serde(rename = "imap-host")]
    pub host: String,
    #[serde(rename = "imap-port")]
    pub port: u16,
    #[serde(rename = "imap-ssl")]
    pub ssl: Option<bool>,
    #[serde(rename = "imap-starttls")]
    pub starttls: Option<bool>,
    #[serde(rename = "imap-insecure")]
    pub insecure: Option<bool>,
    #[serde(rename = "imap-login")]
    pub login: String,
    #[serde(flatten, with = "ImapAuthConfigDef")]
    pub auth: ImapAuthConfig,
    #[serde(rename = "imap-notify-cmd")]
    pub notify_cmd: Option<String>,
    #[serde(rename = "imap-notify-query")]
    pub notify_query: Option<String>,
    #[serde(rename = "imap-watch-cmds")]
    pub watch_cmds: Option<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "ImapAuthConfig", tag = "imap-auth")]
pub enum ImapAuthConfigDef {
    #[serde(rename = "passwd", alias = "password", with = "ImapPasswdDef")]
    Passwd(Secret),
    #[serde(rename = "oauth2", with = "ImapOAuth2ConfigDef")]
    OAuth2(OAuth2Config),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "Secret", rename_all = "kebab-case")]
pub enum ImapPasswdDef {
    #[serde(rename = "imap-passwd")]
    Raw(String),
    #[serde(rename = "imap-passwd-cmd", with = "CmdDef")]
    Cmd(Cmd),
    #[serde(rename = "imap-passwd-keyring", with = "EntryDef")]
    Keyring(Entry),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "OAuth2Config")]
pub struct ImapOAuth2ConfigDef {
    #[serde(rename = "imap-oauth2-method", with = "OAuth2MethodDef")]
    pub method: OAuth2Method,
    #[serde(rename = "imap-oauth2-client-id")]
    pub client_id: String,
    #[serde(flatten, with = "ImapOAuth2ClientSecretDef")]
    pub client_secret: Secret,
    #[serde(rename = "imap-oauth2-auth-url")]
    pub auth_url: String,
    #[serde(rename = "imap-oauth2-token-url")]
    pub token_url: String,
    #[serde(flatten, with = "ImapOAuth2AccessTokenDef")]
    pub access_token: Secret,
    #[serde(flatten, with = "ImapOAuth2RefreshTokenDef")]
    pub refresh_token: Secret,
    #[serde(flatten, with = "ImapOAuth2ScopesDef")]
    pub scopes: OAuth2Scopes,
    #[serde(rename = "imap-oauth2-pkce", default)]
    pub pkce: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "Secret")]
pub enum ImapOAuth2ClientSecretDef {
    #[serde(rename = "imap-oauth2-client-secret")]
    Raw(String),
    #[serde(rename = "imap-oauth2-client-secret-cmd", with = "CmdDef")]
    Cmd(Cmd),
    #[serde(rename = "imap-oauth2-client-secret-keyring", with = "EntryDef")]
    Keyring(Entry),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "Secret")]
pub enum ImapOAuth2AccessTokenDef {
    #[serde(rename = "imap-oauth2-access-token")]
    Raw(String),
    #[serde(rename = "imap-oauth2-access-token-cmd", with = "CmdDef")]
    Cmd(Cmd),
    #[serde(rename = "imap-oauth2-access-token-keyring", with = "EntryDef")]
    Keyring(Entry),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "Secret")]
pub enum ImapOAuth2RefreshTokenDef {
    #[serde(rename = "imap-oauth2-refresh-token")]
    Raw(String),
    #[serde(rename = "imap-oauth2-refresh-token-cmd", with = "CmdDef")]
    Cmd(Cmd),
    #[serde(rename = "imap-oauth2-refresh-token-keyring", with = "EntryDef")]
    Keyring(Entry),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "OAuth2Scopes")]
pub enum ImapOAuth2ScopesDef {
    #[serde(rename = "imap-oauth2-scope")]
    Scope(String),
    #[serde(rename = "imap-oauth2-scopes")]
    Scopes(Vec<String>),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "MaildirConfig", rename_all = "kebab-case")]
pub struct MaildirConfigDef {
    #[serde(rename = "maildir-root-dir")]
    pub root_dir: PathBuf,
}

#[cfg(feature = "notmuch-backend")]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "NotmuchConfig", rename_all = "kebab-case")]
pub struct NotmuchConfigDef {
    #[serde(rename = "notmuch-db-path")]
    pub db_path: PathBuf,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(
    remote = "EmailTextPlainFormat",
    tag = "type",
    content = "width",
    rename_all = "kebab-case"
)]
pub enum EmailTextPlainFormatDef {
    #[default]
    Auto,
    Flowed,
    Fixed(usize),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "EmailSender", tag = "sender", rename_all = "kebab-case")]
pub enum EmailSenderDef {
    #[default]
    None,
    #[serde(with = "SmtpConfigDef")]
    Smtp(SmtpConfig),
    #[serde(with = "SendmailConfigDef")]
    Sendmail(SendmailConfig),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "SendmailConfig", rename_all = "kebab-case")]
pub struct SendmailConfigDef {
    #[serde(rename = "sendmail-cmd")]
    cmd: String,
}

/// Represents the email hooks. Useful for doing extra email
/// processing before or after sending it.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "EmailHooks", rename_all = "kebab-case")]
pub struct EmailHooksDef {
    /// Represents the hook called just before sending an email.
    pub pre_send: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(remote = "SyncFoldersStrategy", rename_all = "kebab-case")]
pub enum SyncFoldersStrategyDef {
    #[default]
    All,
    #[serde(alias = "only")]
    Include(HashSet<String>),
    #[serde(alias = "except")]
    #[serde(alias = "ignore")]
    Exclude(HashSet<String>),
}
