//! Example showing how to use the AWS KMS signer from the CLI with the default AWS config,
//! such as an active `aws login` SSO session.

use alloy_signer::Signer;
use alloy_signer_aws::{AwsSigner, aws_sdk_kms};
use aws_config::BehaviorVersion;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok();
	let key_id = std::env::var("AWS_KEY_ID").expect("AWS_KEY_ID not set in .env file");

	let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
	let client = aws_sdk_kms::Client::new(&config);
	let signer = AwsSigner::new(client, key_id, Some(1)).await?;

	let message = "Hello, world!";
	let signature = signer.sign_message(message.as_bytes()).await?;

	assert_eq!(signature.recover_address_from_msg(message)?, signer.address());

	Ok(())
}
