#[allow(dead_code, unused_imports, unused_mut, unused_variables)]
mod x_com {
    pub mod import_api {}

    pub mod source_api {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/x_com/source_api.rs"
        ));
    }
}

use std::{env, sync::Arc};

use anyhow::{anyhow, bail, Result};
use semver::Version;
use x_com::source_api::{
    CreateSimAccountRequest, ListSimAccountAuditEventsRequest, ListSimAccountsRequest,
    SimAccountAuditAction, SimAccountAuditEventList, SimAccountIdRequest, SimAccountInfo,
    SimAccountList, SimAccountServiceHealth, SimAccountStatus, UpdateSimAccountRequest,
};
use x_com_lib::{x_core::serial_request, CodedInputStream, RequestMessage, Status, TargetKey};
use x_common_lib::{base::id_generator::make_node_id, protocol::protocol_dxc::ProtocolDXCReader};
use xport_lib::{
    airport::channel::SELF_CHANNEL, airport::channel_manager::BuildMessageChannelResult,
    application::Application, service::outer_service::OuterService,
};

const DXC_NAME: &str = "UsOptionSimAccountService";
const DXC_VERSION: &str = "0.0.1";
const CREATE_API: &str = "CreateSimAccount";
const GET_API: &str = "GetSimAccount";
const LIST_API: &str = "ListSimAccounts";
const UPDATE_API: &str = "UpdateSimAccount";
const AUDIT_API: &str = "ListSimAccountAuditEvents";
const HEALTH_API: &str = "GetSimAccountServiceHealth";
const DEFAULT_XRPC_PORT: u16 = 28710;
const DEFAULT_CLIENT_XRPC_PORT: u16 = 0;
const CLIENT_PRIVATE_KEY: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const ACCOUNT_ID: &str = "runtime-acct-001";

#[derive(Debug)]
struct Args {
    xrpc_port: u16,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error = {}", toml_string(&err.to_string()));
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = parse_args(env::args().skip(1))?;
    let outer_service = boot_runtime_client().await?;

    let created = invoke_once::<CreateSimAccountRequest, SimAccountInfo>(
        &outer_service,
        args.xrpc_port,
        CREATE_API,
        &create_request(),
    )
    .await?;
    assert_created(&created)?;

    let fetched = invoke_once::<SimAccountIdRequest, SimAccountInfo>(
        &outer_service,
        args.xrpc_port,
        GET_API,
        &account_request(),
    )
    .await?;
    assert_created(&fetched)?;

    let listed = invoke_once::<ListSimAccountsRequest, SimAccountList>(
        &outer_service,
        args.xrpc_port,
        LIST_API,
        &list_request(),
    )
    .await?;
    if listed.accounts.len() != 1 || listed.accounts[0].account_id != ACCOUNT_ID {
        bail!("list response did not contain expected account");
    }

    let updated = invoke_once::<UpdateSimAccountRequest, SimAccountInfo>(
        &outer_service,
        args.xrpc_port,
        UPDATE_API,
        &update_request(),
    )
    .await?;
    if updated.status != Some(SimAccountStatus::Paused) {
        bail!("updated account status mismatch: {:?}", updated.status);
    }
    if updated.strategy_task_id != "runtime-task-002" || updated.run_id != "runtime-run-002" {
        bail!(
            "updated binding mismatch: {}/{}",
            updated.strategy_task_id,
            updated.run_id
        );
    }

    let audit = invoke_once::<ListSimAccountAuditEventsRequest, SimAccountAuditEventList>(
        &outer_service,
        args.xrpc_port,
        AUDIT_API,
        &audit_request(),
    )
    .await?;
    if audit.events.len() != 2 {
        bail!("unexpected audit event count: {}", audit.events.len());
    }
    if !audit
        .events
        .iter()
        .any(|event| event.action == Some(SimAccountAuditAction::StatusChanged))
    {
        bail!("audit events did not include status change");
    }

    let health =
        invoke_no_param_once::<SimAccountServiceHealth>(&outer_service, args.xrpc_port, HEALTH_API)
            .await?;
    if health.account_count != 1 || health.audit_event_count != 2 {
        bail!(
            "unexpected health counts: accounts={} audit={}",
            health.account_count,
            health.audit_event_count
        );
    }

    println!("runtime_typed_invoke_ok = true");
    println!("create_sim_account_ok = true");
    println!("get_sim_account_ok = true");
    println!("list_sim_accounts_ok = true");
    println!("update_sim_account_ok = true");
    println!("list_sim_account_audit_events_ok = true");
    println!("get_sim_account_service_health_ok = true");
    println!("account_count_matches = true");
    println!("audit_event_count_matches = true");
    println!("account_count = {}", health.account_count);
    println!("audit_event_count = {}", health.audit_event_count);
    println!("PASS SimAccount runtime typed XRPC smoke");
    Ok(())
}

fn parse_args<I>(mut args: I) -> Result<Args>
where
    I: Iterator<Item = String>,
{
    let mut xrpc_port = DEFAULT_XRPC_PORT;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "--xrpc-port" => xrpc_port = parse_port("--xrpc-port", &next_arg(&mut args, &arg)?)?,
            _ if arg.starts_with("--xrpc-port=") => {
                xrpc_port = parse_port("--xrpc-port", &arg["--xrpc-port=".len()..])?;
            }
            _ => bail!("unknown argument: {arg}"),
        }
    }
    Ok(Args { xrpc_port })
}

fn print_usage() {
    println!("usage = \"cargo run --bin sim_account_runtime_xrpc_smoke -- --xrpc-port 28710\"");
}

fn next_arg<I>(args: &mut I, label: &str) -> Result<String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| anyhow!("{label} requires a value"))
}

fn parse_port(label: &str, value: &str) -> Result<u16> {
    let port: u16 = value
        .parse()
        .map_err(|_| anyhow!("{label} must be a TCP port"))?;
    if port == 0 {
        bail!("{label} cannot be 0");
    }
    Ok(port)
}

fn create_request() -> Box<CreateSimAccountRequest> {
    let mut request = Box::new(CreateSimAccountRequest::default());
    request.account_id = ACCOUNT_ID.to_owned();
    request.display_name = "Runtime account".to_owned();
    request.initial_cash = 250_000.0;
    request.currency = "usd".to_owned();
    request.strategy_task_id = "runtime-task-001".to_owned();
    request.run_id = "runtime-run-001".to_owned();
    request.created_by = "runtime-smoke".to_owned();
    request.risk_limits.max_single_order_notional = Some(12_500.0);
    request.risk_limits.max_open_order_count = Some(8);
    request.risk_limits.allow_opening_trades = Some(true);
    request
}

fn account_request() -> Box<SimAccountIdRequest> {
    let mut request = Box::new(SimAccountIdRequest::default());
    request.account_id = ACCOUNT_ID.to_owned();
    request
}

fn list_request() -> Box<ListSimAccountsRequest> {
    let mut request = Box::new(ListSimAccountsRequest::default());
    request.strategy_task_id = "runtime-task-001".to_owned();
    request.limit = 10;
    request
}

fn update_request() -> Box<UpdateSimAccountRequest> {
    let mut request = Box::new(UpdateSimAccountRequest::default());
    request.account_id = ACCOUNT_ID.to_owned();
    request.update_binding = true;
    request.strategy_task_id = "runtime-task-002".to_owned();
    request.run_id = "runtime-run-002".to_owned();
    request.update_status = true;
    request.status = Some(SimAccountStatus::Paused);
    request.actor = "runtime-risk".to_owned();
    request.reason = "runtime smoke".to_owned();
    request
}

fn audit_request() -> Box<ListSimAccountAuditEventsRequest> {
    let mut request = Box::new(ListSimAccountAuditEventsRequest::default());
    request.account_id = ACCOUNT_ID.to_owned();
    request.limit = 10;
    request
}

fn assert_created(account: &SimAccountInfo) -> Result<()> {
    if account.account_id != ACCOUNT_ID {
        bail!("account_id mismatch: {}", account.account_id);
    }
    if account.currency != "USD" {
        bail!("currency mismatch: {}", account.currency);
    }
    if account.status != Some(SimAccountStatus::Active) {
        bail!("account status mismatch: {:?}", account.status);
    }
    if account.initial_cash != 250_000.0 {
        bail!("initial_cash mismatch: {}", account.initial_cash);
    }
    if account.risk_limits.max_open_order_count != Some(8) {
        bail!(
            "risk max_open_order_count mismatch: {:?}",
            account.risk_limits.max_open_order_count
        );
    }
    Ok(())
}

async fn invoke_once<Req, Resp>(
    outer_service: &Arc<Box<OuterService>>,
    xrpc_port: u16,
    api: &str,
    request: &Box<Req>,
) -> Result<Box<Resp>>
where
    Req: RequestMessage,
    Resp: RequestMessage + Default + 'static,
{
    let request_buffer = serial_request(request);
    let conn_id = make_node_id("127.0.0.1", xrpc_port);
    let channel_id = build_remote_message_channel(conn_id).await?;
    let response_buffer = outer_service
        .send_message(build_target_key(api), channel_id, request_buffer)
        .await
        .map_err(status_to_anyhow)?;
    parse_runtime_response(&response_buffer)
}

async fn invoke_no_param_once<Resp>(
    outer_service: &Arc<Box<OuterService>>,
    xrpc_port: u16,
    api: &str,
) -> Result<Box<Resp>>
where
    Resp: RequestMessage + Default + 'static,
{
    let conn_id = make_node_id("127.0.0.1", xrpc_port);
    let channel_id = build_remote_message_channel(conn_id).await?;
    let response_buffer = outer_service
        .send_message(build_target_key(api), channel_id, Vec::new())
        .await
        .map_err(status_to_anyhow)?;
    parse_runtime_response(&response_buffer)
}

async fn build_remote_message_channel(conn_id: i64) -> Result<i64> {
    let channel_manager = Application::get_airport().get_channel_manager();
    let build_result = channel_manager
        .build_message_channel(conn_id)
        .await
        .ok_or_else(|| anyhow!("connect runtime xrpc node failed"))?;
    let channel_id = match build_result {
        BuildMessageChannelResult::Existing(channel_id) => channel_id,
        BuildMessageChannelResult::Created(channel_id) => {
            if channel_id != SELF_CHANNEL {
                channel_manager
                    .handshake_message_channel(conn_id, channel_id)
                    .await
                    .map_err(status_to_anyhow)?;
            }
            channel_id
        }
    };
    Ok(channel_id)
}

fn build_target_key(api: &str) -> TargetKey {
    let mut target = TargetKey::default();
    target.dxc_name = DXC_NAME.to_owned();
    target.dxc_version = DXC_VERSION.to_owned();
    target.api = api.to_owned();
    target
}

fn parse_runtime_response<Resp>(response_buffer: &[u8]) -> Result<Box<Resp>>
where
    Resp: RequestMessage + Default + 'static,
{
    let reader = ProtocolDXCReader::new(response_buffer);
    let body = reader.msg_body();
    let mut is = CodedInputStream::from_bytes(body);
    let mut status = Status::default();
    status
        .parse_from_input_stream_with_tag_and_len(&mut is)
        .map_err(status_to_anyhow)?;
    if status.is_erorr() {
        return Err(status_to_anyhow(status));
    }
    if is.pos() >= body.len() as u64 {
        bail!("runtime response body did not contain typed response");
    }
    let mut response = Box::<Resp>::default();
    response
        .parse_from_input_stream_with_tag_and_len(&mut is)
        .map_err(status_to_anyhow)?;
    Ok(response)
}

async fn boot_runtime_client() -> Result<Arc<Box<OuterService>>> {
    xport_lib::application::init_runtime();
    let config_text = format!(
        "[xport]\n\
         xrpc-listen = false\n\
         xrpc-ip = \"127.0.0.1\"\n\
         xrpc-port = \"{}\"\n\
         default-dxc = []\n\
         msg-timeout = 10\n\
         log-level = \"info\"\n\
         log-writer = [\"console\"]\n",
        DEFAULT_CLIENT_XRPC_PORT
    );
    Application::init_with_config(CLIENT_PRIVATE_KEY.to_owned(), config_text)
        .await
        .map_err(status_to_anyhow)?;
    let outer_service = Arc::new(OuterService::new(
        "SimAccountRuntimeSmokeClient".to_owned(),
        Version::new(0, 0, 1),
        Box::new(|_| {}),
    ));
    Application::get_airport()
        .get_dxc_manager()
        .add_outer_service(outer_service.clone())
        .await
        .map_err(status_to_anyhow)?;
    outer_service.start();
    Ok(outer_service)
}

fn status_to_anyhow(status: Status) -> anyhow::Error {
    anyhow!("xport status {}: {}", status.err_code, status.err_msg)
}

fn toml_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}
