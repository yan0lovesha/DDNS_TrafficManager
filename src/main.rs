use clap::Parser;
use ureq;
use serde::{Deserialize, Serialize};
use public_ip::addr;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct EndpointProperties {
    endpointLocation: String,
    target: String,
    endpoint_status: String,
}

#[derive(Serialize, Deserialize)]
struct EndpointRequest<'a> {
    properties: EndpointProperties,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Parser)]
struct Args {
    /// The tenant id that host Traffic Manager Profile instance.
    #[clap(long)]
    tenant_id: String,

    /// The Azure subscription id that hosts your resources.
    #[clap(long)]
    subscription_id: String,

    /// The resource group that contains your Traffic Manager Profile instance. 
    #[clap(long)]
    resource_group: String,

    /// The name of the Traffic Manager Profile isntance.
    #[clap(long)]
    traffic_manager_name: String,

    /// The name of Traffic Manager Profile endpoint.
    #[clap(long)]
    endpoint_name: String,

    /// The location of Traffic Manager Profile endpoint.
    #[clap(long)]
    endpoint_location: String,

    /// The client id of the Entra Application that has permission to edit the Traffic Manager Profile instance.
    #[clap(long)]
    client_id: String,

    /// The secret of the Entra Application.
    #[clap(long)] // , default_value_t="asdf"
    client_secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/token",
        args.tenant_id
    );

    let token_request = [
        ("grant_type", "client_credentials"),
        ("client_id", &args.client_id),
        ("client_secret", &args.client_secret),
        ("resource", "https://management.azure.com/"),
    ];

    let token_response: TokenResponse = ureq::post(&token_url)
        .send_form(&token_request)?
        .into_json()?;

    let access_token = token_response.access_token;
    
    let public_ip = addr().await.ok_or("Failed to get public IP address")?;
    println!("Public IP: {:?}", public_ip);

    let endpoint_url = format!(
        "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/trafficManagerProfiles/{}/externalEndpoints/{}?api-version=2022-04-01",
        args.subscription_id, args.resource_group, args.traffic_manager_name, args.endpoint_name
    );

    let endpoint_request = EndpointRequest {
        properties: EndpointProperties {
            endpointLocation: args.endpoint_location,
            target: public_ip.to_string(),
            endpoint_status: "Enabled".to_string(),
        },
        name: Some(&args.endpoint_name),
    };

    let response = ureq::put(&endpoint_url)
        .set("Authorization", &format!("Bearer {}", access_token))
        .send_json(serde_json::to_value(&endpoint_request)?)?;

    if response.status() % 200 == 1 {
        println!("Endpoint updated successfully.");
    } else {
        println!("Failed to update endpoint. Status Code {:?} \r\n\r\n {:?}", response.status(), response.into_string()?);
    }

    Ok(())
}