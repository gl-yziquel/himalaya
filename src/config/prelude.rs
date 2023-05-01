use pimalaya_email::{
    folder::sync::Strategy as SyncFoldersStrategy, EmailHooks, EmailSender, EmailTextPlainFormat,
    MaildirConfig, SendmailConfig, SmtpConfig,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};

#[cfg(feature = "imap-backend")]
use pimalaya_email::ImapConfig;

#[cfg(feature = "notmuch-backend")]
use pimalaya_email::NotmuchConfig;

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
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
    #[serde(rename = "smtp-passwd-cmd")]
    pub passwd_cmd: String,
}

#[cfg(feature = "imap-backend")]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
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
    #[serde(rename = "imap-passwd-cmd")]
    pub passwd_cmd: String,
    #[serde(rename = "imap-notify-cmd")]
    pub notify_cmd: Option<String>,
    #[serde(rename = "imap-notify-query")]
    pub notify_query: Option<String>,
    #[serde(rename = "imap-watch-cmds")]
    pub watch_cmds: Option<Vec<String>>,
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