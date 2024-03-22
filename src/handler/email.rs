use crate::service::{
    email::DynEmailServiceTrait, group::DynGroupServiceTrait, subscriber::DynSubscriberServiceTrait,
};
use tonic::{Request, Response, Status};

use madtofan_microservice_common::email::{
    email_server::Email, AddGroupRequest, AddSubscriberRequest, BlastEmailRequest, EmailResponse,
    GetSubscriberGroupsRequest, GetSubscribersRequest, GroupsResponse, RemoveGroupRequest,
    RemoveSubscriberRequest, SendEmailRequest, SubscribersResponse,
};

pub struct RequestHandler {
    subscriber_service: DynSubscriberServiceTrait,
    group_service: DynGroupServiceTrait,
    email_service: DynEmailServiceTrait,
}

impl RequestHandler {
    pub fn new(
        subscriber_service: DynSubscriberServiceTrait,
        group_service: DynGroupServiceTrait,
        email_service: DynEmailServiceTrait,
    ) -> Self {
        Self {
            subscriber_service,
            group_service,
            email_service,
        }
    }
}

#[tonic::async_trait]
impl Email for RequestHandler {
    async fn send_email(
        &self,
        request: Request<SendEmailRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        self.email_service
            .send_email(req.email, req.title, req.body)
            .await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Success sending email!"),
        }))
    }

    async fn blast_email(
        &self,
        request: Request<BlastEmailRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        let subscribers = self
            .subscriber_service
            .list_subs_by_group(req.group, None, None)
            .await?
            .subscribers;

        let addresses = subscribers
            .into_iter()
            .map(|subscriber| subscriber.email)
            .collect::<Vec<String>>();

        self.email_service
            .blast_email(addresses, req.title, req.body)
            .await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Success blasting email!"),
        }))
    }

    async fn add_subscriber(
        &self,
        request: Request<AddSubscriberRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        self.subscriber_service
            .add_subscriber(req.email, req.group)
            .await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Successfully add subscriber!"),
        }))
    }

    async fn remove_subscriber(
        &self,
        request: Request<RemoveSubscriberRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        self.subscriber_service
            .remove_subscriber_from_group(req.email, req.group)
            .await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Successfully removed subscriber!"),
        }))
    }

    async fn add_group(
        &self,
        request: Request<AddGroupRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        self.group_service
            .add_group(req.name, req.description)
            .await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Successfully add group!"),
        }))
    }

    async fn remove_group(
        &self,
        request: Request<RemoveGroupRequest>,
    ) -> Result<Response<EmailResponse>, Status> {
        let req = request.into_inner();

        self.group_service.remove_group(req.name).await?;

        Ok(Response::new(EmailResponse {
            message: String::from("Successfully removed group!"),
        }))
    }

    async fn get_subscribers(
        &self,
        request: Request<GetSubscribersRequest>,
    ) -> Result<Response<SubscribersResponse>, Status> {
        let req = request.into_inner();

        let subscribers_response = self
            .subscriber_service
            .list_subs_by_group(req.group, Some(req.offset), Some(req.limit))
            .await?;

        Ok(Response::new(subscribers_response))
    }

    async fn get_subscriber_groups(
        &self,
        request: Request<GetSubscriberGroupsRequest>,
    ) -> Result<Response<GroupsResponse>, Status> {
        let req = request.into_inner();

        let group_response = self
            .group_service
            .list_groups_by_sub(req.email, Some(req.offset), Some(req.limit))
            .await?;

        Ok(Response::new(group_response))
    }
}
