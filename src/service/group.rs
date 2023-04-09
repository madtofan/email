use std::sync::Arc;

use async_trait::async_trait;
use common::errors::{ServiceError, ServiceResult};
use mockall::automock;
use tracing::log::{error, info};

use crate::repository::group::{DynGroupRepositoryTrait, GroupEntity};

#[automock]
#[async_trait]
pub trait GroupServiceTrait {
    async fn add_group(&self, name: String, description: String) -> ServiceResult<()>;
    async fn remove_group(&self, group: String) -> ServiceResult<Option<GroupEntity>>;
    async fn list_groups_by_sub(&self, email: String) -> ServiceResult<Vec<GroupEntity>>;
}

pub type DynGroupServiceTrait = Arc<dyn GroupServiceTrait + Sync + Send>;

pub struct GroupService {
    repository: DynGroupRepositoryTrait,
}

impl GroupService {
    pub fn new(repository: DynGroupRepositoryTrait) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl GroupServiceTrait for GroupService {
    async fn add_group(&self, name: String, description: String) -> ServiceResult<()> {
        let existing_group = self.repository.get_group(&name).await?;

        if existing_group.is_some() {
            error!("group {:?} already exists", &name);
            return Err(ServiceError::ObjectConflict(String::from(
                "group name is taken",
            )));
        }

        info!("creating group {:?}", &name);
        self.repository.add_group(&name, &description).await?;

        info!("group successfully created");

        Ok(())
    }

    async fn remove_group(&self, name: String) -> ServiceResult<Option<GroupEntity>> {
        let existing_group = self.repository.get_group(&name).await?;

        if existing_group.is_none() {
            error!("group {:?} does not exist", &name);
            return Err(ServiceError::ObjectConflict(String::from(
                "group does not exist",
            )));
        }

        info!("deleting group {:?}", &name);
        let removed_group = self.repository.remove_group(&name).await?;

        info!("group successfully removed");

        Ok(removed_group)
    }

    async fn list_groups_by_sub(&self, email: String) -> ServiceResult<Vec<GroupEntity>> {
        let groups = self.repository.list_groups_by_sub(&email).await?;

        Ok(groups)
    }
}
