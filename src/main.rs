use ::email::account::{sync::AccountSyncBuilder, DEFAULT_INBOX_FOLDER};
use anyhow::{anyhow, Context, Result};
use clap::Command;
use log::{debug, warn};
use std::env;
use url::Url;

#[cfg(feature = "imap-backend")]
use himalaya::imap;
use himalaya::{
    account,
    backend::BackendBuilder,
    cache, compl,
    config::{self, DeserializedConfig},
    email, flag, folder, man, output,
    printer::StdoutPrinter,
    tpl, IdMapper,
};

fn create_app() -> Command {
    let app = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .propagate_version(true)
        .infer_subcommands(true)
        .arg(config::args::arg())
        .arg(account::args::arg())
        .arg(cache::args::arg())
        .args(output::args::args())
        .arg(folder::args::source_arg())
        .subcommand(compl::args::subcmd())
        .subcommand(man::args::subcmd())
        .subcommand(account::args::subcmd())
        .subcommand(folder::args::subcmd())
        .subcommands(email::args::subcmds());

    #[cfg(feature = "imap-backend")]
    let app = app.subcommands(imap::args::subcmds());

    app
}

#[allow(clippy::single_match)]
#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    if let Err((_, err)) = coredump::register_panic_handler() {
        warn!("cannot register custom panic handler: {err}");
        debug!("cannot register custom panic handler: {err:?}");
    }

    let default_env_filter = env_logger::DEFAULT_FILTER_ENV;
    env_logger::init_from_env(env_logger::Env::default().filter_or(default_env_filter, "off"));

    // checks mailto command before app initialization
    let raw_args: Vec<String> = env::args().collect();
    if raw_args.len() > 1 && raw_args[1].starts_with("mailto:") {
        let url = Url::parse(&raw_args[1])?;
        let (toml_account_config, account_config) = DeserializedConfig::from_opt_path(None)
            .await?
            .into_account_configs(None)?;
        let backend_builder =
            BackendBuilder::new(toml_account_config, account_config.clone(), false).await?;
        let backend = backend_builder.build().await?;
        let mut printer = StdoutPrinter::default();

        return email::handlers::mailto(&account_config, &backend, &mut printer, &url).await;
    }

    let app = create_app();
    let m = app.get_matches();

    // check completion command before configs
    // https://github.com/soywod/himalaya/issues/115
    match compl::args::matches(&m)? {
        Some(compl::args::Cmd::Generate(shell)) => {
            return compl::handlers::generate(create_app(), shell);
        }
        _ => (),
    }

    // check also man command before configs
    match man::args::matches(&m)? {
        Some(man::args::Cmd::GenerateAll(dir)) => {
            return man::handlers::generate(dir, create_app());
        }
        _ => (),
    }

    let maybe_account_name = account::args::parse_arg(&m);
    let toml_config = DeserializedConfig::from_opt_path(config::args::parse_arg(&m)).await?;
    let (toml_account_config, account_config) = toml_config
        .clone()
        .into_account_configs(maybe_account_name)?;

    let folder = folder::args::parse_source_arg(&m);
    let disable_cache = cache::args::parse_disable_cache_flag(&m);

    let mut printer = StdoutPrinter::try_from(&m)?;

    // FIXME
    // #[cfg(feature = "imap-backend")]
    // if let BackendConfig::Imap(imap_config) = &account_config.backend {
    //     let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
    //     match imap::args::matches(&m)? {
    //         Some(imap::args::Cmd::Notify(keepalive)) => {
    //             let backend =
    //                 ImapBackend::new(account_config.clone(), imap_config.clone(), None).await?;
    //             imap::handlers::notify(&mut backend, &folder, keepalive).await?;
    //             return Ok(());
    //         }
    //         Some(imap::args::Cmd::Watch(keepalive)) => {
    //             let backend =
    //                 ImapBackend::new(account_config.clone(), imap_config.clone(), None).await?;
    //             imap::handlers::watch(&mut backend, &folder, keepalive).await?;
    //             return Ok(());
    //         }
    //         _ => (),
    //     }
    // }

    match account::args::matches(&m)? {
        Some(account::args::Cmd::List(max_width)) => {
            account::handlers::list(max_width, &account_config, &toml_config, &mut printer)?;
            return Ok(());
        }
        Some(account::args::Cmd::Sync(strategy, dry_run)) => {
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), true).await?;
            let sync_builder = AccountSyncBuilder::new(backend_builder.0)
                .await?
                .with_some_folders_strategy(strategy)
                .with_dry_run(dry_run);
            account::handlers::sync(&mut printer, sync_builder, dry_run).await?;
            return Ok(());
        }
        Some(account::args::Cmd::Configure(reset)) => {
            account::handlers::configure(&account_config, reset).await?;
            return Ok(());
        }
        _ => (),
    }

    // checks folder commands
    match folder::args::matches(&m)? {
        Some(folder::args::Cmd::Create) => {
            let folder = folder
                .ok_or_else(|| anyhow!("the folder argument is missing"))
                .context("cannot create folder")?;
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;

            return folder::handlers::create(&mut printer, &backend, &folder).await;
        }
        Some(folder::args::Cmd::List(max_width)) => {
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;

            return folder::handlers::list(&account_config, &mut printer, &backend, max_width)
                .await;
        }
        Some(folder::args::Cmd::Expunge) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;

            return folder::handlers::expunge(&mut printer, &backend, &folder).await;
        }
        Some(folder::args::Cmd::Delete) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;

            return folder::handlers::delete(&mut printer, &backend, &folder).await;
        }
        _ => (),
    }

    // checks email commands
    match email::args::matches(&m)? {
        Some(email::args::Cmd::Attachments(ids)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::attachments(
                &account_config,
                &mut printer,
                &id_mapper,
                &backend,
                &folder,
                ids,
            )
            .await;
        }
        Some(email::args::Cmd::Copy(ids, to_folder)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::copy(
                &mut printer,
                &id_mapper,
                &backend,
                &folder,
                to_folder,
                ids,
            )
            .await;
        }
        Some(email::args::Cmd::Delete(ids)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::delete(&mut printer, &id_mapper, &backend, &folder, ids).await;
        }
        Some(email::args::Cmd::Forward(id, headers, body)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::forward(
                &account_config,
                &mut printer,
                &id_mapper,
                &backend,
                &folder,
                id,
                headers,
                body,
            )
            .await;
        }
        Some(email::args::Cmd::List(max_width, page_size, page)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::list(
                &account_config,
                &mut printer,
                &id_mapper,
                &backend,
                &folder,
                max_width,
                page_size,
                page,
            )
            .await;
        }
        Some(email::args::Cmd::Move(ids, to_folder)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::move_(
                &mut printer,
                &id_mapper,
                &backend,
                &folder,
                to_folder,
                ids,
            )
            .await;
        }
        Some(email::args::Cmd::Read(ids, text_mime, raw, headers)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::read(
                &account_config,
                &mut printer,
                &id_mapper,
                &backend,
                &folder,
                ids,
                text_mime,
                raw,
                headers,
            )
            .await;
        }
        Some(email::args::Cmd::Reply(id, all, headers, body)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::reply(
                &account_config,
                &mut printer,
                &id_mapper,
                &backend,
                &folder,
                id,
                all,
                headers,
                body,
            )
            .await;
        }
        Some(email::args::Cmd::Save(raw_email)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::save(&mut printer, &id_mapper, &backend, &folder, raw_email)
                .await;
        }
        Some(email::args::Cmd::Search(query, max_width, page_size, page)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::search(
                &account_config,
                &mut printer,
                &id_mapper,
                &backend,
                &folder,
                query,
                max_width,
                page_size,
                page,
            )
            .await;
        }
        Some(email::args::Cmd::Sort(criteria, query, max_width, page_size, page)) => {
            let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

            return email::handlers::sort(
                &account_config,
                &mut printer,
                &id_mapper,
                &backend,
                &folder,
                criteria,
                query,
                max_width,
                page_size,
                page,
            )
            .await;
        }
        Some(email::args::Cmd::Send(raw_email)) => {
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            return email::handlers::send(&account_config, &mut printer, &backend, raw_email).await;
        }
        Some(email::args::Cmd::Flag(m)) => match m {
            Some(flag::args::Cmd::Set(ids, ref flags)) => {
                let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
                let backend_builder =
                    BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                        .await?;
                let backend = backend_builder.build().await?;
                let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;
                return flag::handlers::set(
                    &mut printer,
                    &id_mapper,
                    &backend,
                    &folder,
                    ids,
                    flags,
                )
                .await;
            }
            Some(flag::args::Cmd::Add(ids, ref flags)) => {
                let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
                let backend_builder =
                    BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                        .await?;
                let backend = backend_builder.build().await?;
                let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;
                return flag::handlers::add(
                    &mut printer,
                    &id_mapper,
                    &backend,
                    &folder,
                    ids,
                    flags,
                )
                .await;
            }
            Some(flag::args::Cmd::Remove(ids, ref flags)) => {
                let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
                let backend_builder =
                    BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                        .await?;
                let backend = backend_builder.build().await?;
                let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;

                return flag::handlers::remove(
                    &mut printer,
                    &id_mapper,
                    &backend,
                    &folder,
                    ids,
                    flags,
                )
                .await;
            }
            _ => (),
        },
        Some(email::args::Cmd::Tpl(m)) => match m {
            Some(tpl::args::Cmd::Forward(id, headers, body)) => {
                let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
                let backend_builder =
                    BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                        .await?;
                let backend = backend_builder.build().await?;
                let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;
                return tpl::handlers::forward(
                    &account_config,
                    &mut printer,
                    &id_mapper,
                    &backend,
                    &folder,
                    id,
                    headers,
                    body,
                )
                .await;
            }
            Some(tpl::args::Cmd::Write(headers, body)) => {
                tpl::handlers::write(&account_config, &mut printer, headers, body).await?;
                return Ok(());
            }
            Some(tpl::args::Cmd::Reply(id, all, headers, body)) => {
                let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
                let backend_builder =
                    BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                        .await?;
                let backend = backend_builder.build().await?;
                let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;
                return tpl::handlers::reply(
                    &account_config,
                    &mut printer,
                    &id_mapper,
                    &backend,
                    &folder,
                    id,
                    all,
                    headers,
                    body,
                )
                .await;
            }
            Some(tpl::args::Cmd::Save(tpl)) => {
                let folder = folder.unwrap_or(DEFAULT_INBOX_FOLDER);
                let backend_builder =
                    BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                        .await?;
                let backend = backend_builder.build().await?;
                let id_mapper = IdMapper::new(&backend, &account_config, &folder)?;
                return tpl::handlers::save(
                    &account_config,
                    &mut printer,
                    &id_mapper,
                    &backend,
                    &folder,
                    tpl,
                )
                .await;
            }
            Some(tpl::args::Cmd::Send(tpl)) => {
                let backend_builder =
                    BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                        .await?;
                let backend = backend_builder.build().await?;
                return tpl::handlers::send(&account_config, &mut printer, &backend, tpl).await;
            }
            _ => (),
        },
        Some(email::args::Cmd::Write(headers, body)) => {
            let backend_builder =
                BackendBuilder::new(toml_account_config, account_config.clone(), disable_cache)
                    .await?;
            let backend = backend_builder.build().await?;
            return email::handlers::write(&account_config, &mut printer, &backend, headers, body)
                .await;
        }
        _ => (),
    }

    Ok(())
}
