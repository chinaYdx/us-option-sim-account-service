#![allow(unused_imports)]
#![allow(static_mut_refs)]

use crate::service::UsOptionSimAccountService;
use core::slice;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;
use x_com_lib::status_err;
use x_com_lib::x_core;
use x_com_lib::x_core::gen_id;
use x_com_lib::x_core::parse_request_param;
use x_com_lib::x_core::response_empty_msg;
use x_com_lib::x_core::response_msg;
use x_com_lib::x_core::set_request_id;
use x_com_lib::x_core::xrpc;
use x_com_lib::x_core::{
    get_runtime, init_runtime, take_runtime, take_stream_runtime, wait_task_empty,
};
use x_com_lib::CodedInputStream;
use x_com_lib::ProtocolDXCReader;
use x_com_lib::Status;
pub static mut SERVICE: Option<Box<UsOptionSimAccountService>> = None;
static SERVICE_STATE: AtomicU8 = AtomicU8::new(0);
const SERVICE_STATE_INITIALIZING: u8 = 0;
const SERVICE_STATE_RUNNING: u8 = 1;
const SERVICE_STATE_FINALIZING: u8 = 2;
const SERVICE_STATE_STOPPED: u8 = 3;
fn set_service_state(state: u8) {
    SERVICE_STATE.store(state, Ordering::Release);
}
fn is_service_running() -> bool {
    SERVICE_STATE.load(Ordering::Acquire) == SERVICE_STATE_RUNNING
}
pub fn get_service() -> Option<&'static UsOptionSimAccountService> {
    unsafe { SERVICE.as_ref().map(|service| &**service) }
}

#[unsafe(no_mangle)]
pub extern "C" fn init(service_id: i64, config: *const u8, config_len: u32, log_level: i32) {
    set_service_state(SERVICE_STATE_INITIALIZING);
    init_runtime();
    let runtime = get_runtime();
    // 初始化日志
    runtime.block_on(async {
        let request_id = gen_id();
        set_request_id(request_id);
        let config_str = unsafe {
            let buffer = slice::from_raw_parts(config as *mut u8, config_len as usize);
            std::str::from_utf8_unchecked(buffer)
        };
        x_core::init_app(
            service_id,
            "UsOptionSimAccountService",
            &config_str,
            log_level,
        );
        let service_ins = Box::new(UsOptionSimAccountService::new());
        unsafe {
            SERVICE = Some(service_ins);
            let ret = SERVICE.as_mut().unwrap().on_init().await;
            if ret.is_err() {
                set_service_state(SERVICE_STATE_STOPPED);
                response_empty_msg(0, &ret);
                return;
            }
        }
        set_service_state(SERVICE_STATE_RUNNING);
        set_request_id(0);
        let ok = Ok(());
        response_empty_msg(0, &ok);
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn finalize() {
    set_service_state(SERVICE_STATE_FINALIZING);
    let runtime = get_runtime();
    runtime.block_on(async {
        let request_id = gen_id();
        set_request_id(request_id);
        let _ = wait_task_empty(Duration::from_secs(3)).await;
        unsafe {
            if let Some(service) = SERVICE.as_mut() {
                service.on_finalize().await;
            }
        }
        set_request_id(0);
    });
    let runtime = take_runtime();
    let stream_runtime = take_stream_runtime();
    stream_runtime.shutdown_timeout(Duration::from_secs(1));
    runtime.shutdown_timeout(Duration::from_secs(1));
    unsafe {
        SERVICE.take();
    }
    set_service_state(SERVICE_STATE_STOPPED);
}
#[unsafe(no_mangle)]
pub extern "C" fn dispatch_message(buffer: *const u8, buffer_len: u32) {
    let vec_buffer = unsafe { slice::from_raw_parts(buffer as *mut u8, buffer_len as usize) };

    let _dxc_msg_reader = ProtocolDXCReader::new(vec_buffer);
    let msg_header = _dxc_msg_reader.header();
    let msg_body = _dxc_msg_reader.msg_body();
    let _ctx = xrpc::Context {
        sender_service_key: msg_header.sender_key,
        channel_id: msg_header.channel_id,
        conn_id: msg_header.conn_id,
        request_id: msg_header.request_id,
        from_addr: msg_header.from_address,
    };
    if !is_service_running() {
        let err_status = status_err!("服务卸载中，暂不接收请求");
        response_empty_msg(msg_header.request_id, &err_status);
        return;
    }
    let Some(receiver_key) = msg_header.receiver_key else {
        let err_status = status_err!("数据格式出错！");
        response_empty_msg(msg_header.request_id, &err_status);
        return;
    };
    let mut _input_stream = CodedInputStream::from_bytes(msg_body);
    match receiver_key.api.as_str() {
        "CreateSimAccount" => {
            let param = parse_request_param(&mut _input_stream);
            x_core::spawn(async move {
                let Some(service) = get_service() else {
                    let err_status = status_err!("服务已卸载");
                    response_empty_msg(msg_header.request_id, &err_status);
                    return;
                };
                let result = service.create_sim_account(_ctx, param).await;
                response_msg(msg_header.request_id, &result);
            });
        }
        "GetSimAccount" => {
            let param = parse_request_param(&mut _input_stream);
            x_core::spawn(async move {
                let Some(service) = get_service() else {
                    let err_status = status_err!("服务已卸载");
                    response_empty_msg(msg_header.request_id, &err_status);
                    return;
                };
                let result = service.get_sim_account(_ctx, param).await;
                response_msg(msg_header.request_id, &result);
            });
        }
        "ListSimAccounts" => {
            let param = parse_request_param(&mut _input_stream);
            x_core::spawn(async move {
                let Some(service) = get_service() else {
                    let err_status = status_err!("服务已卸载");
                    response_empty_msg(msg_header.request_id, &err_status);
                    return;
                };
                let result = service.list_sim_accounts(_ctx, param).await;
                response_msg(msg_header.request_id, &result);
            });
        }
        "UpdateSimAccount" => {
            let param = parse_request_param(&mut _input_stream);
            x_core::spawn(async move {
                let Some(service) = get_service() else {
                    let err_status = status_err!("服务已卸载");
                    response_empty_msg(msg_header.request_id, &err_status);
                    return;
                };
                let result = service.update_sim_account(_ctx, param).await;
                response_msg(msg_header.request_id, &result);
            });
        }
        "ListSimAccountAuditEvents" => {
            let param = parse_request_param(&mut _input_stream);
            x_core::spawn(async move {
                let Some(service) = get_service() else {
                    let err_status = status_err!("服务已卸载");
                    response_empty_msg(msg_header.request_id, &err_status);
                    return;
                };
                let result = service.list_sim_account_audit_events(_ctx, param).await;
                response_msg(msg_header.request_id, &result);
            });
        }
        "GetSimAccountServiceHealth" => {
            x_core::spawn(async move {
                let Some(service) = get_service() else {
                    let err_status = status_err!("服务已卸载");
                    response_empty_msg(msg_header.request_id, &err_status);
                    return;
                };
                let result = service.get_sim_account_service_health(_ctx).await;
                response_msg(msg_header.request_id, &result);
            });
        }
        _ => {
            let err_status = status_err!("{} 不存在！", receiver_key.api);
            response_empty_msg(msg_header.request_id, &err_status);
        }
    }
}
