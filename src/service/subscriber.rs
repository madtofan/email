use std::sync::Arc;

use async_trait::async_trait;
use madtofan_microservice_common::{
    email::{subscribers_response::Subscriber, SubscribersResponse},
    errors::{ServiceError, ServiceResult},
};
use mockall::automock;
use tracing::log::{error, info};

use crate::repository::{group::DynGroupRepositoryTrait, subcriber::DynSubscriberRepositoryTrait};

#[automock]
#[async_trait]
pub trait SubscriberServiceTrait {
    async fn list_subs_by_group(
        &self,
        group_name: String,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> ServiceResult<SubscribersResponse>;
    async fn add_subscriber(&self, email: String, group_name: String) -> ServiceResult<()>;
    async fn remove_subscriber_from_group(
        &self,
        email: String,
        group_name: String,
    ) -> ServiceResult<()>;
}

pub type DynSubscriberServiceTrait = Arc<dyn SubscriberServiceTrait + Sync + Send>;

pub struct SubscriberService {
    subscriber_repository: DynSubscriberRepositoryTrait,
    group_repository: DynGroupRepositoryTrait,
}

impl SubscriberService {
    pub fn new(
        subscriber_repository: DynSubscriberRepositoryTrait,
        group_repository: DynGroupRepositoryTrait,
    ) -> Self {
        Self {
            subscriber_repository,
            group_repository,
        }
    }
}

#[async_trait]
impl SubscriberServiceTrait for SubscriberService {
    async fn list_subs_by_group(
        &self,
        group_name: String,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> ServiceResult<SubscribersResponse> {
        let existing_group = self.group_repository.get_group(&group_name).await?;

        match existing_group {
            Some(group) => {
                info!("listing subscriber from group {:?}", &group_name);
                let subscriber_entity = self
                    .subscriber_repository
                    .list_subs_by_group(&group, offset, limit)
                    .await?;
                let count = self
                    .subscriber_repository
                    .get_subs_by_group_count(&group)
                    .await?;

                info!("successfully obtained list of subscriber from group");
                Ok(SubscribersResponse {
                    count,
                    subscribers: subscriber_entity
                        .into_iter()
                        .map(|sub| sub.into_subscriber_response())
                        .collect::<Vec<Subscriber>>(),
                })
            }
            None => {
                error!("group {:?} does not exists", &group_name);
                Err(ServiceError::ObjectConflict(String::from(
                    "group name does not exist",
                )))
            }
        }
    }

    async fn add_subscriber(&self, email: String, group_name: String) -> ServiceResult<()> {
        let existing_group = self.group_repository.get_group(&group_name).await?;

        match existing_group {
            Some(group) => {
                info!("add subscriber into group {:?}", &group_name);
                self.subscriber_repository
                    .add_subscriber(&email, &group)
                    .await?;

                info!("successfully added subscriber into group");
                Ok(())
            }
            None => {
                error!("group {:?} does not exists", &group_name);
                Err(ServiceError::ObjectConflict(String::from(
                    "group name does not exist",
                )))
            }
        }
    }

    async fn remove_subscriber_from_group(
        &self,
        email: String,
        group_name: String,
    ) -> ServiceResult<()> {
        let existing_group = self.group_repository.get_group(&group_name).await?;

        match existing_group {
            Some(group) => {
                info!("removed subscriber from group {:?}", &group_name);
                self.subscriber_repository
                    .remove_subscriber_from_group(&email, &group)
                    .await?;

                info!("successfully removed subscriber from group");
                Ok(())
            }
            None => {
                error!("group {:?} does not exists", &group_name);
                Err(ServiceError::ObjectConflict(String::from(
                    "group name does not exist",
                )))
            }
        }
    }
}
