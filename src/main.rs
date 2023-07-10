use crate::config::AppConfig;
use crate::handler::email::RequestHandler;
use crate::repository::group::{DynGroupRepositoryTrait, GroupRepository};
use crate::repository::subcriber::{DynSubscriberRepositoryTrait, SubscriberRepository};
use crate::service::email::{DynEmailServiceTrait, EmailService};
use crate::service::group::{DynGroupServiceTrait, GroupService};
use crate::service::subscriber::{DynSubscriberServiceTrait, SubscriberService};
use clap::Parser;
use dotenv::dotenv;
use madtofan_microservice_common::{
    email::email_server::EmailServer, repository::connection_pool::ServiceConnectionManager,
};
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{error, info};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod config;
mod handler;
mod repository;
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let config = Arc::new(AppConfig::parse());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Environment loaded and configuration parsed, initializing Postgres connection...");
    let pg_pool = ServiceConnectionManager::new_pool(&config.database_url)
        .await
        .expect("could not initialize the database connection pool");

    if config.run_migrations {
        info!("migrations enabled, running...");
        sqlx::migrate!()
            .run(&pg_pool)
            .await
            .unwrap_or_else(|err| error!("There was an error during migration: {:?}", err));
    }
    info!("Database configured! initializing repositories...");

    let app_host = &config.service_url;
    let app_port = &config.service_port;
    let app_url = format!("{}:{}", app_host, app_port).parse().unwrap();
    let subscriber_repository =
        Arc::new(SubscriberRepository::new(pg_pool.clone())) as DynSubscriberRepositoryTrait;
    let group_repository = Arc::new(GroupRepository::new(pg_pool)) as DynGroupRepositoryTrait;
    info!("Repositories initialized, Initializing Services");
    let subscriber_service = Arc::new(SubscriberService::new(
        subscriber_repository,
        group_repository.clone(),
    )) as DynSubscriberServiceTrait;
    let group_service = Arc::new(GroupService::new(group_repository)) as DynGroupServiceTrait;
    let email_service = Arc::new(EmailService::new(&config)) as DynEmailServiceTrait;
    info!("Services initialized, Initializing Handler");
    let request_handler = RequestHandler::new(subscriber_service, group_service, email_service);

    info!("Service ready for request at {:#?}!", app_url);
    Server::builder()
        .add_service(EmailServer::new(request_handler))
        .serve(app_url)
        .await?;
    Ok(())
}
