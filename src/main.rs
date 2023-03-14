use crate::config::AppConfig;
use crate::handler::RequestHandler;
use crate::repository::group::{DynGroupRepositoryTrait, GroupRepository};
use crate::repository::subcriber::{DynSubscriberRepositoryTrait, SubscriberRepository};
use crate::service::email::{DynEmailServiceTrait, EmailService};
use crate::service::group::{DynGroupServiceTrait, GroupService};
use crate::service::subscriber::{DynSubscriberServiceTrait, SubscriberService};
use clap::Parser;
use common::repository::connection_pool::ServiceConnectionManager;
use dotenv::dotenv;
use email::email_server::EmailServer;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod config;
mod handler;
mod repository;
mod service;
pub mod email {
    tonic::include_proto!("email");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect("Failed to read .env file, please add a .env file to the project root");

    let config = Arc::new(AppConfig::parse());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Environment loaded and configuration parsed, initializing Postgres connection and running migrations...");
    let pg_pool = ServiceConnectionManager::new_pool(&config.database_url)
        .await
        .expect("could not initialize the database connection pool");

    if *&config.seed {
        todo!("Migrations is not done yet")
        // info!("migrations enabled, running...");
        // sqlx::migrate!()
        //     .run(&pool)
        //     .await
        //     .context("error while running database migrations")?;
    }

    let app_host = &config.service_url;
    let app_port = &config.service_port;
    let app_url = format!("{}:{}", app_host, app_port).parse().unwrap();
    let subscriber_repository =
        Arc::new(SubscriberRepository::new(pg_pool.clone())) as DynSubscriberRepositoryTrait;
    let group_repository = Arc::new(GroupRepository::new(pg_pool)) as DynGroupRepositoryTrait;
    let email_service = Arc::new(EmailService::new(&config)) as DynEmailServiceTrait;
    let subscriber_service = Arc::new(SubscriberService::new(
        subscriber_repository,
        group_repository.clone(),
    )) as DynSubscriberServiceTrait;
    let group_service = Arc::new(GroupService::new(group_repository)) as DynGroupServiceTrait;
    let request_handler = RequestHandler::new(subscriber_service, group_service, email_service);

    info!("Service ready for request!");
    Server::builder()
        .add_service(EmailServer::new(request_handler))
        .serve(app_url)
        .await?;
    Ok(())
}
