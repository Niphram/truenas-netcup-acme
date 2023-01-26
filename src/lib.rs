use anyhow::{anyhow, Context, Ok};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ResponseStatus {
    Error,
    Started,
    Pending,
    Warning,
    Success,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseMessage {
    #[serde(rename = "serverrequestid")]
    server_request_id: String,
    #[serde(rename = "clientrequestid")]
    client_request_id: Option<String>,
    action: String,
    status: ResponseStatus,
    #[serde(rename = "statuscode")]
    status_code: u32,
    #[serde(rename = "shortmessage")]
    short_message: String,
    #[serde(rename = "longmessage")]
    long_message: Option<String>,
    #[serde(rename = "responsedata")]
    response_data: Option<ResponseData>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ResponseData {
    DNSRecords(Vec<DNSRecord>),
    APISessionId(String),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DNSRecord {
    id: Option<String>,
    hostname: String,
    #[serde(rename = "type")]
    record_type: String,
    priority: Option<String>,
    destination: String,
    deleterecord: Option<bool>,
    state: Option<String>,
}

pub struct NetcupAPIClient {
    session: String,
    customer_number: String,
    api_key: String,
    client: surf::Client,
}

const ENDPOINT: &str = "https://ccp.netcup.net/run/webservice/servers/endpoint.php?JSON";

impl NetcupAPIClient {
    pub async fn login(
        customer_number: String,
        api_password: String,
        api_key: String,
    ) -> anyhow::Result<Self> {
        let client = surf::Client::new();

        let payload = json!({
            "action": "login",
            "param": {
                "apikey": &api_key,
                "apipassword": &api_password,
                "customernumber": &customer_number
            }
        });

        let mut response = client
            .post(ENDPOINT)
            .body_json(&payload)
            .map_err(|err| anyhow!(err))?
            .await
            .map_err(|err| anyhow!(err))?;

        let message: ResponseMessage = response.body_json().await.map_err(|err| anyhow!(err))?;

        if let Some(ResponseData::APISessionId(session_id)) = message.response_data {
            Ok(Self {
                session: session_id,
                customer_number,
                api_key,
                client,
            })
        } else {
            Err(anyhow!("Could not login!"))
        }
    }

    pub async fn logout(self) -> anyhow::Result<()> {
        let payload = json!({
            "action": "logout",
            "param": {
                "apikey": &self.api_key,
                "apisessionid": &self.session,
                "customernumber": &self.customer_number
            }
        });

        self.client
            .post(ENDPOINT)
            .body_json(&payload)
            .map_err(|err| anyhow!(err))?
            .await
            .map_err(|err| anyhow!(err))?;

        Ok(())
    }

    pub async fn list_records(&self, domain: &str) -> anyhow::Result<Vec<DNSRecord>> {
        let payload = json!({
            "action": "infoDnsRecords",
            "param": {
                "apikey": &self.api_key,
                "apisessionid": &self.session,
                "customernumber": &self.customer_number,
                "domainname": domain
            }
        });

        let mut response = self
            .client
            .post(ENDPOINT)
            .body_json(&payload)
            .map_err(|err| anyhow!(err))?
            .await
            .map_err(|err| anyhow!(err))?;

        let message: ResponseMessage = response.body_json().await.map_err(|err| anyhow!(err))?;

        if let Some(ResponseData::DNSRecords(records)) = message.response_data {
            Ok(records)
        } else {
            Err(anyhow!("No records were returned!"))
        }
    }

    pub async fn find_txt_record_id(
        &self,
        domain: &str,
        hostname: &str,
        content: &str,
    ) -> anyhow::Result<String> {
        self.list_records(domain)
            .await?
            .iter()
            .find_map(|r| {
                let found =
                    r.hostname == hostname && r.record_type == "TXT" && r.destination == content;

                found.then_some(r.id.clone())
            })
            .context("Could not find record!")?
            .context("Record has no id!")
    }

    pub async fn add_txt_record(
        &self,
        domain: &str,
        hostname: &str,
        content: &str,
    ) -> anyhow::Result<()> {
        let payload = json!({
            "action": "updateDnsRecords",
            "param": {
                "apikey": &self.api_key,
                "apisessionid": &self.session,
                "customernumber": &self.customer_number,
                "domainname": domain,
                "dnsrecordset": {
                    "dnsrecords": [
                        DNSRecord {
                            id: None,
                            hostname: hostname.into(),
                            record_type: "TXT".into(),
                            priority: None,
                            destination: content.into(),
                            deleterecord: None,
                            state: None
                        }
                    ]
                }
            }
        });

        let mut response = self
            .client
            .post(ENDPOINT)
            .body_json(&payload)
            .map_err(|err| anyhow!(err))?
            .await
            .map_err(|err| anyhow!(err))?;

        let message: ResponseMessage = response.body_json().await.map_err(|err| anyhow!(err))?;

        if let Some(ResponseData::DNSRecords(records)) = message.response_data {
            records
                .iter()
                .find(|record| {
                    record.hostname == hostname
                        && record.record_type == "TXT"
                        && record.destination == content
                })
                .context("Could not find updated record!")?;

            Ok(())
        } else {
            Err(anyhow!("Could not update records!"))
        }
    }

    pub async fn delete_record(
        &self,
        id: &str,
        domain: &str,
        hostname: &str,
        content: &str,
    ) -> anyhow::Result<()> {
        let payload = json!({
            "action": "updateDnsRecords",
            "param": {
                "apikey": &self.api_key,
                "apisessionid": &self.session,
                "customernumber": &self.customer_number,
                "domainname": domain,
                "dnsrecordset": {
                    "dnsrecords": [
                        DNSRecord {
                            id: Some(id.into()),
                            hostname: hostname.into(),
                            record_type: "TXT".into(),
                            priority: None,
                            destination: content.into(),
                            deleterecord: Some(true),
                            state: None
                        }
                    ]
                }
            }
        });

        let mut response = self
            .client
            .post(ENDPOINT)
            .body_json(&payload)
            .map_err(|err| anyhow!(err))?
            .await
            .map_err(|err| anyhow!(err))?;

        let response_data: ResponseMessage =
            response.body_json().await.map_err(|err| anyhow!(err))?;

        match response_data.status {
            ResponseStatus::Success => Ok(()),
            _ => Err(anyhow!("Could not delete record!")),
        }
    }
}
