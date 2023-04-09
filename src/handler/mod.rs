pub mod email;

#[cfg(test)]
pub mod test {
    use std::sync::Arc;

    use clap::Parser;
    use sqlx::PgPool;
    use tonic::Request;

    use crate::{
        config::AppConfig,
        email::{
            email_server::Email, AddGroupRequest, AddSubscriberRequest, BlastEmailRequest,
            GetSubscriberGroupsRequest, GetSubscribersRequest, RemoveGroupRequest,
            RemoveSubscriberRequest, SendEmailRequest,
        },
        handler::email::RequestHandler,
        repository::{
            group::{DynGroupRepositoryTrait, GroupRepository},
            subcriber::{DynSubscriberRepositoryTrait, SubscriberRepository},
        },
        service::{
            email::{DynEmailServiceTrait, EmailService},
            group::{DynGroupServiceTrait, GroupService},
            subscriber::{DynSubscriberServiceTrait, SubscriberService},
        },
    };

    struct AllTraits {
        subscriber_repository: DynSubscriberRepositoryTrait,
        group_repository: DynGroupRepositoryTrait,
        handler: RequestHandler,
    }

    fn initialize_handler(pool: PgPool) -> AllTraits {
        let config = Arc::new(AppConfig::parse());

        let subscriber_repository =
            Arc::new(SubscriberRepository::new(pool.clone())) as DynSubscriberRepositoryTrait;
        let group_repository =
            Arc::new(GroupRepository::new(pool.clone())) as DynGroupRepositoryTrait;
        let subscriber_service = Arc::new(SubscriberService::new(
            subscriber_repository.clone(),
            group_repository.clone(),
        )) as DynSubscriberServiceTrait;
        let group_service =
            Arc::new(GroupService::new(group_repository.clone())) as DynGroupServiceTrait;
        let email_service = Arc::new(EmailService::new(&config)) as DynEmailServiceTrait;
        let handler = RequestHandler::new(
            subscriber_service.clone(),
            group_service.clone(),
            email_service.clone(),
        );

        AllTraits {
            subscriber_repository,
            group_repository,
            handler,
        }
    }

    #[sqlx::test]
    async fn send_email_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);
        let request = Request::new(SendEmailRequest {
            body: "test_email_body".to_string(),
            email: "test@address.com".to_string(),
            title: "test_email_title".to_string(),
        });

        all_traits.handler.send_email(request).await?;

        Ok(())
    }

    #[sqlx::test]
    #[ignore]
    async fn blast_email_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);
        let group_name = "group_name";
        let group = all_traits
            .group_repository
            .add_group(group_name, "group_description")
            .await?;

        let sub1_email = "sub1_email";
        all_traits
            .subscriber_repository
            .add_subscriber(sub1_email, &group)
            .await?;
        let sub2_email = "sub2_email";
        all_traits
            .subscriber_repository
            .add_subscriber(sub2_email, &group)
            .await?;

        let request = Request::new(BlastEmailRequest {
            group: group_name.to_string(),
            body: "email body".to_string(),
            title: "email title".to_string(),
        });

        all_traits.handler.blast_email(request).await?;

        Ok(())
    }

    #[sqlx::test]
    async fn add_subscriber_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group_name = "group_name";
        let group_description = "group_description";
        all_traits
            .group_repository
            .add_group(group_name, group_description)
            .await?;

        let sub_email = "test@address.com";
        let request = Request::new(AddSubscriberRequest {
            email: sub_email.to_string(),
            group: group_name.to_string(),
        });

        all_traits.handler.add_subscriber(request).await?;
        let added_sub = all_traits
            .group_repository
            .list_groups_by_sub(sub_email)
            .await?;
        assert_eq!(added_sub.first().unwrap().description, group_description);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_subscriber_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group_name = "group_name";
        let group = all_traits
            .group_repository
            .add_group(group_name, "group_description")
            .await?;

        let sub1_email = "sub1_email";
        all_traits
            .subscriber_repository
            .add_subscriber(sub1_email, &group)
            .await?;
        let sub2_email = "sub2_email";
        all_traits
            .subscriber_repository
            .add_subscriber(sub2_email, &group)
            .await?;

        let request = Request::new(RemoveSubscriberRequest {
            email: sub1_email.to_string(),
            group: group_name.to_string(),
        });

        all_traits.handler.remove_subscriber(request).await?;
        let subs_list = all_traits
            .subscriber_repository
            .list_subs_by_group(&group)
            .await?;

        assert_eq!(subs_list.len(), 1);
        assert_eq!(subs_list.first().unwrap().email, sub2_email);

        Ok(())
    }

    #[sqlx::test]
    async fn add_group_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group_name = "group_name";
        let group_description = "group_description";

        let request = Request::new(AddGroupRequest {
            name: group_name.to_string(),
            description: group_description.to_string(),
        });

        all_traits.handler.add_group(request).await?;

        let group = all_traits.group_repository.get_group(group_name).await?;

        assert_eq!(group.unwrap().description, group_description);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_group_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group1_name = "group1";
        let group1_description = "description1";
        all_traits
            .group_repository
            .add_group(group1_name, group1_description)
            .await?;

        let group_to_remove_name = "group_to_remove";
        let group_to_remove_description = "group_to_remove_description";
        all_traits
            .group_repository
            .add_group(group_to_remove_name, group_to_remove_description)
            .await?;

        let request = Request::new(RemoveGroupRequest {
            name: group_to_remove_name.to_string(),
        });

        all_traits.handler.remove_group(request).await?;

        let groups_list = all_traits.group_repository.list_groups().await?;

        assert_eq!(groups_list.len(), 1);
        assert_eq!(group1_description, groups_list.first().unwrap().description);

        Ok(())
    }

    #[sqlx::test]
    async fn get_subscribers_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group1_name = "group1_name";
        let group1 = all_traits
            .group_repository
            .add_group(group1_name, "group1_description")
            .await?;
        let group2 = all_traits
            .group_repository
            .add_group("group2_name", "group2_description")
            .await?;

        let sub1_email = "sub1_email";
        all_traits
            .subscriber_repository
            .add_subscriber(sub1_email, &group1)
            .await?;
        let sub2_email = "sub2_email";
        all_traits
            .subscriber_repository
            .add_subscriber(sub2_email, &group2)
            .await?;

        let request = Request::new(GetSubscribersRequest {
            group: group1_name.to_string(),
        });

        let subs_list = all_traits
            .handler
            .get_subscribers(request)
            .await?
            .into_inner()
            .subscribers;

        assert_eq!(subs_list.len(), 1);
        assert_eq!(subs_list.first().unwrap().email, sub1_email);

        Ok(())
    }

    #[sqlx::test]
    async fn get_subscriber_groups_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let group1_name = "group1_name";
        let group1_description = "group1_description";
        let group2_name = "group2_name";
        let group2_description = "group2_description";
        let group1 = all_traits
            .group_repository
            .add_group(group1_name, group1_description)
            .await?;
        all_traits
            .group_repository
            .add_group(group2_name, group2_description)
            .await?;

        let subscriber_email = "subscriber_email";
        all_traits
            .subscriber_repository
            .add_subscriber(subscriber_email, &group1)
            .await?;

        let request = Request::new(GetSubscriberGroupsRequest {
            email: subscriber_email.to_string(),
        });

        let groups_list = all_traits
            .handler
            .get_subscriber_groups(request)
            .await?
            .into_inner()
            .groups;

        assert_eq!(groups_list.len(), 1);
        assert_eq!(groups_list.first().unwrap().description, group1_description);

        Ok(())
    }
}
