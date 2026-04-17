//! Example showing how to use the AWS KMS signer from app (server).

use alloy_signer::Signer;
use alloy_signer_aws::{AwsSigner, aws_sdk_kms};
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok();
	let key_id = std::env::var("AWS_KEY_ID").expect("AWS_KEY_ID not set in .env file");
	let access_key_id =
		std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID not set in .env file");
	let secret_access_key =
		std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY not set in .env file");
	let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-west-2".to_string());

	let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "env");
	let config = aws_config::defaults(BehaviorVersion::latest())
		.region(Region::new(region))
		.credentials_provider(credentials)
		.load()
		.await;
	let client = aws_sdk_kms::Client::new(&config);
	let signer = AwsSigner::new(client, key_id, Some(1)).await?;

	let message = "Hello, world!";
	let signature = signer.sign_message(message.as_bytes()).await?;

	assert_eq!(signature.recover_address_from_msg(message)?, signer.address());

	Ok(())
}
