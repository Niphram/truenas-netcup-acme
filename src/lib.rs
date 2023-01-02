use anyhow::{anyhow, Context, Ok};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
enum ResponseStatus {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "success")]
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
enum ResponseData {
    #[serde(rename = "dnsrecords")]
    DNSRecords(Vec<DNSRecord>),
    #[serde(rename = "apisessionid")]
    APISessionId(String),
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
    client: Client,
}

const ENDPOINT: &str = "https://ccp.netcup.net/run/webservice/servers/endpoint.php?JSON";

impl NetcupAPIClient {
    pub fn login(
        customer_number: String,
        api_password: String,
        api_key: String,
    ) -> anyhow::Result<Self> {
        let client = reqwest::blocking::Client::new();

        let payload = json!({
            "action": "login",
            "param": {
                "apikey": &api_key,
                "apipassword": &api_password,
                "customernumber": &customer_number
            }
        });

        let response = client.post(ENDPOINT).body(payload.to_string()).send()?;
        let response: ResponseMessage = serde_json::from_str(&response.text()?)?;

        if let Some(ResponseData::APISessionId(session_id)) = response.response_data {
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

    pub fn logout(self) -> anyhow::Result<()> {
        let payload = json!({
            "action": "logout",
            "param": {
                "apikey": &self.api_key,
                "apisessionid": &self.session,
                "customernumber": &self.customer_number
            }
        });

        println!("Logout payload: {}", payload);

        let body = self
            .client
            .post(ENDPOINT)
            .body(payload.to_string())
            .send()?;

        println!("{}", body.text()?);

        Ok(())
    }

    pub fn list_records(&self, domain: &str) -> anyhow::Result<Vec<DNSRecord>> {
        let payload = json!({
            "action": "infoDnsRecords",
            "param": {
                "apikey": &self.api_key,
                "apisessionid": &self.session,
                "customernumber": &self.customer_number,
                "domainname": domain
            }
        });

        let body = self
            .client
            .post(ENDPOINT)
            .body(payload.to_string())
            .send()?;

        let response: ResponseMessage = serde_json::from_str(&body.text()?)?;

        if let Some(ResponseData::DNSRecords(records)) = response.response_data {
            Ok(records)
        } else {
            Err(anyhow!("No records were returned!"))
        }
    }

    pub fn find_txt_record_id(
        &self,
        domain: &str,
        hostname: &str,
        content: &str,
    ) -> anyhow::Result<String> {
        self.list_records(domain)?
            .iter()
            .find_map(|r| {
                let found =
                    r.hostname == hostname && r.record_type == "TXT" && r.destination == content;

                found.then_some(r.id.clone())
            })
            .context("Could not find record!")?
            .context("Record has no id!")
    }

    pub fn add_txt_record(
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

        let body = self
            .client
            .post(ENDPOINT)
            .body(payload.to_string())
            .send()?;

        let response: ResponseMessage = serde_json::from_str(&body.text()?)?;

        if let Some(ResponseData::DNSRecords(records)) = response.response_data {
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

    pub fn delete_record(
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

        let body = self
            .client
            .post(ENDPOINT)
            .body(payload.to_string())
            .send()?;

        let response: ResponseMessage = serde_json::from_str(&body.text()?)?;

        match response.status {
            ResponseStatus::Success => Ok(()),
            _ => Err(anyhow!("Could not delete record!")),
        }
    }
}

impl Drop for NetcupAPIClient {
    fn drop(&mut self) {
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
            .body(payload.to_string())
            .send()
            .expect("Send Logout");
    }
}
