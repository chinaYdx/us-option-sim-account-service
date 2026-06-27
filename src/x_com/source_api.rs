#[allow(unused_imports)]
use super::import_api::*;
#[allow(unused_imports)]
use x_com_lib::{
    self, extract_wire_type_from_tag, pb_error_to_status,
    x_core::{self, serial_request},
    CodedInputStream, CodedOutputStream, RequestMessage, Status,
};

#[derive(Clone)]
pub struct UpdateSimAccountRequest {
    pub account_id: String,
    pub display_name: String,
    pub update_display_name: bool,
    pub strategy_task_id: String,
    pub run_id: String,
    pub update_binding: bool,
    pub status: Option<SimAccountStatus>,
    pub update_status: bool,
    pub risk_limits: Box<SimAccountRiskLimits>,
    pub update_risk_limits: bool,
    pub actor: String,
    pub reason: String,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimAccountStatus {
    Unknown = 0,
    Active = 1,
    Paused = 2,
    Archived = 3,
}

#[derive(Clone)]
pub struct ListSimAccountsRequest {
    pub status: Option<SimAccountStatus>,
    pub strategy_task_id: String,
    pub run_id: String,
    pub query: String,
    pub include_archived: bool,
    pub limit: i32,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimAccountTradingEngine {
    Unknown = 0,
    DqteaSim = 1,
    MoomooSimulate = 2,
}

#[derive(Clone)]
pub struct SimAccountRiskLimits {
    pub max_single_order_notional: Option<f64>,
    pub max_daily_notional: Option<f64>,
    pub max_open_order_count: Option<i32>,
    pub max_contract_quantity: Option<i32>,
    pub max_quote_age_ms: Option<i64>,
    pub max_spread_pct: Option<f64>,
    pub max_abs_spread: Option<f64>,
    pub allow_opening_trades: Option<bool>,
    pub allow_naked_short_options: Option<bool>,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone)]
pub struct SimAccountList {
    pub accounts: Vec<Box<SimAccountInfo>>,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone)]
pub struct SimAccountServiceHealth {
    pub sqlite_path: String,
    pub account_count: i64,
    pub audit_event_count: i64,
    pub updated_at: i64,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone)]
pub struct ListSimAccountAuditEventsRequest {
    pub account_id: String,
    pub start_time: i64,
    pub end_time: i64,
    pub limit: i32,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone)]
pub struct SimAccountAuditEvent {
    pub event_id: String,
    pub account_id: String,
    pub action: Option<SimAccountAuditAction>,
    pub actor: String,
    pub reason: String,
    pub old_status: Option<SimAccountStatus>,
    pub new_status: Option<SimAccountStatus>,
    pub old_strategy_task_id: String,
    pub new_strategy_task_id: String,
    pub old_run_id: String,
    pub new_run_id: String,
    pub risk_limits_changed: bool,
    pub event_at: i64,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone)]
pub struct CreateSimAccountRequest {
    pub account_id: String,
    pub display_name: String,
    pub initial_cash: f64,
    pub currency: String,
    pub strategy_task_id: String,
    pub run_id: String,
    pub created_by: String,
    pub risk_limits: Box<SimAccountRiskLimits>,
    pub trading_engine: Option<SimAccountTradingEngine>,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone)]
pub struct SimAccountAuditEventList {
    pub events: Vec<Box<SimAccountAuditEvent>>,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone)]
pub struct SimAccountIdRequest {
    pub account_id: String,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone)]
pub struct SimAccountInfo {
    pub account_id: String,
    pub display_name: String,
    pub initial_cash: f64,
    pub currency: String,
    pub status: Option<SimAccountStatus>,
    pub strategy_task_id: String,
    pub run_id: String,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub risk_limits: Box<SimAccountRiskLimits>,
    pub trading_engine: Option<SimAccountTradingEngine>,
    pub(crate) cached_size: x_com_lib::rt::CachedSize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimAccountAuditAction {
    Unknown = 0,
    Created = 1,
    Updated = 2,
    StatusChanged = 3,
    BindingChanged = 4,
    RiskLimitsChanged = 5,
}

impl std::fmt::Debug for SimAccountAuditEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimAccountAuditEvent")
            .field("event_id", &self.event_id)
            .field("account_id", &self.account_id)
            .field("action", &self.action)
            .field("actor", &self.actor)
            .field("reason", &self.reason)
            .field("old_status", &self.old_status)
            .field("new_status", &self.new_status)
            .field("old_strategy_task_id", &self.old_strategy_task_id)
            .field("new_strategy_task_id", &self.new_strategy_task_id)
            .field("old_run_id", &self.old_run_id)
            .field("new_run_id", &self.new_run_id)
            .field("risk_limits_changed", &self.risk_limits_changed)
            .field("event_at", &self.event_at)
            .finish()
    }
}

impl Default for SimAccountAuditEvent {
    fn default() -> Self {
        SimAccountAuditEvent {
            event_id: String::default(),
            account_id: String::default(),
            action: SimAccountAuditAction::from_i32(0),
            actor: String::default(),
            reason: String::default(),
            old_status: SimAccountStatus::from_i32(0),
            new_status: SimAccountStatus::from_i32(0),
            old_strategy_task_id: String::default(),
            new_strategy_task_id: String::default(),
            old_run_id: String::default(),
            new_run_id: String::default(),
            risk_limits_changed: false,
            event_at: 0,
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for SimAccountAuditEvent {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if !self.event_id.is_empty() {
            my_size += x_com_lib::rt::string_size(1, &self.event_id);
        }
        if !self.account_id.is_empty() {
            my_size += x_com_lib::rt::string_size(2, &self.account_id);
        }
        if let Some(value) = self.action.as_ref() {
            my_size += x_com_lib::rt::int32_size(3, *value as i32);
        }
        if !self.actor.is_empty() {
            my_size += x_com_lib::rt::string_size(4, &self.actor);
        }
        if !self.reason.is_empty() {
            my_size += x_com_lib::rt::string_size(5, &self.reason);
        }
        if let Some(value) = self.old_status.as_ref() {
            my_size += x_com_lib::rt::int32_size(6, *value as i32);
        }
        if let Some(value) = self.new_status.as_ref() {
            my_size += x_com_lib::rt::int32_size(7, *value as i32);
        }
        if !self.old_strategy_task_id.is_empty() {
            my_size += x_com_lib::rt::string_size(8, &self.old_strategy_task_id);
        }
        if !self.new_strategy_task_id.is_empty() {
            my_size += x_com_lib::rt::string_size(9, &self.new_strategy_task_id);
        }
        if !self.old_run_id.is_empty() {
            my_size += x_com_lib::rt::string_size(10, &self.old_run_id);
        }
        if !self.new_run_id.is_empty() {
            my_size += x_com_lib::rt::string_size(11, &self.new_run_id);
        }
        if self.risk_limits_changed != false {
            my_size += 2;
        }
        if self.event_at != 0 {
            my_size += x_com_lib::rt::int64_size(13, self.event_at);
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        if !self.event_id.is_empty() {
            os.write_string(1, &self.event_id).unwrap();
        }
        if !self.account_id.is_empty() {
            os.write_string(2, &self.account_id).unwrap();
        }
        if let Some(value) = self.action.as_ref() {
            os.write_enum(3, *value as i32).unwrap();
        }
        if !self.actor.is_empty() {
            os.write_string(4, &self.actor).unwrap();
        }
        if !self.reason.is_empty() {
            os.write_string(5, &self.reason).unwrap();
        }
        if let Some(value) = self.old_status.as_ref() {
            os.write_enum(6, *value as i32).unwrap();
        }
        if let Some(value) = self.new_status.as_ref() {
            os.write_enum(7, *value as i32).unwrap();
        }
        if !self.old_strategy_task_id.is_empty() {
            os.write_string(8, &self.old_strategy_task_id).unwrap();
        }
        if !self.new_strategy_task_id.is_empty() {
            os.write_string(9, &self.new_strategy_task_id).unwrap();
        }
        if !self.old_run_id.is_empty() {
            os.write_string(10, &self.old_run_id).unwrap();
        }
        if !self.new_run_id.is_empty() {
            os.write_string(11, &self.new_run_id).unwrap();
        }
        if self.risk_limits_changed != false {
            os.write_bool(12, self.risk_limits_changed).unwrap();
        }
        if self.event_at != 0 {
            os.write_int64(13, self.event_at).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                10 => {
                    self.event_id = is.read_string().map_err(pb_error_to_status)?;
                }
                18 => {
                    self.account_id = is.read_string().map_err(pb_error_to_status)?;
                }
                24 => {
                    let value = SimAccountAuditAction::from_i32(
                        is.read_int32().map_err(pb_error_to_status)?,
                    );
                    self.action = value;
                }
                34 => {
                    self.actor = is.read_string().map_err(pb_error_to_status)?;
                }
                42 => {
                    self.reason = is.read_string().map_err(pb_error_to_status)?;
                }
                48 => {
                    let value =
                        SimAccountStatus::from_i32(is.read_int32().map_err(pb_error_to_status)?);
                    self.old_status = value;
                }
                56 => {
                    let value =
                        SimAccountStatus::from_i32(is.read_int32().map_err(pb_error_to_status)?);
                    self.new_status = value;
                }
                66 => {
                    self.old_strategy_task_id = is.read_string().map_err(pb_error_to_status)?;
                }
                74 => {
                    self.new_strategy_task_id = is.read_string().map_err(pb_error_to_status)?;
                }
                82 => {
                    self.old_run_id = is.read_string().map_err(pb_error_to_status)?;
                }
                90 => {
                    self.new_run_id = is.read_string().map_err(pb_error_to_status)?;
                }
                96 => {
                    self.risk_limits_changed = is.read_bool().map_err(pb_error_to_status)?;
                }
                104 => {
                    self.event_at = is.read_int64().map_err(pb_error_to_status)?;
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for UpdateSimAccountRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateSimAccountRequest")
            .field("account_id", &self.account_id)
            .field("display_name", &self.display_name)
            .field("update_display_name", &self.update_display_name)
            .field("strategy_task_id", &self.strategy_task_id)
            .field("run_id", &self.run_id)
            .field("update_binding", &self.update_binding)
            .field("status", &self.status)
            .field("update_status", &self.update_status)
            .field("risk_limits", &self.risk_limits)
            .field("update_risk_limits", &self.update_risk_limits)
            .field("actor", &self.actor)
            .field("reason", &self.reason)
            .finish()
    }
}

impl Default for UpdateSimAccountRequest {
    fn default() -> Self {
        UpdateSimAccountRequest {
            account_id: String::default(),
            display_name: String::default(),
            update_display_name: false,
            strategy_task_id: String::default(),
            run_id: String::default(),
            update_binding: false,
            status: SimAccountStatus::from_i32(0),
            update_status: false,
            risk_limits: Box::new(SimAccountRiskLimits::default()),
            update_risk_limits: false,
            actor: String::default(),
            reason: String::default(),
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for UpdateSimAccountRequest {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if !self.account_id.is_empty() {
            my_size += x_com_lib::rt::string_size(1, &self.account_id);
        }
        if !self.display_name.is_empty() {
            my_size += x_com_lib::rt::string_size(2, &self.display_name);
        }
        if self.update_display_name != false {
            my_size += 2;
        }
        if !self.strategy_task_id.is_empty() {
            my_size += x_com_lib::rt::string_size(4, &self.strategy_task_id);
        }
        if !self.run_id.is_empty() {
            my_size += x_com_lib::rt::string_size(5, &self.run_id);
        }
        if self.update_binding != false {
            my_size += 2;
        }
        if let Some(value) = self.status.as_ref() {
            my_size += x_com_lib::rt::int32_size(7, *value as i32);
        }
        if self.update_status != false {
            my_size += 2;
        }
        {
            let len = self.risk_limits.compute_size();
            my_size += 1 + x_com_lib::rt::compute_raw_varint64_size(len) + len;
        }
        if self.update_risk_limits != false {
            my_size += 2;
        }
        if !self.actor.is_empty() {
            my_size += x_com_lib::rt::string_size(11, &self.actor);
        }
        if !self.reason.is_empty() {
            my_size += x_com_lib::rt::string_size(12, &self.reason);
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        if !self.account_id.is_empty() {
            os.write_string(1, &self.account_id).unwrap();
        }
        if !self.display_name.is_empty() {
            os.write_string(2, &self.display_name).unwrap();
        }
        if self.update_display_name != false {
            os.write_bool(3, self.update_display_name).unwrap();
        }
        if !self.strategy_task_id.is_empty() {
            os.write_string(4, &self.strategy_task_id).unwrap();
        }
        if !self.run_id.is_empty() {
            os.write_string(5, &self.run_id).unwrap();
        }
        if self.update_binding != false {
            os.write_bool(6, self.update_binding).unwrap();
        }
        if let Some(value) = self.status.as_ref() {
            os.write_enum(7, *value as i32).unwrap();
        }
        if self.update_status != false {
            os.write_bool(8, self.update_status).unwrap();
        }
        {
            os.write_tag(9, x_com_lib::rt::WireType::LengthDelimited)
                .unwrap();
            os.write_raw_varint32(self.risk_limits.cached_size.get() as u32)
                .unwrap();
            self.risk_limits.serial_with_output_stream(os).unwrap();
        }
        if self.update_risk_limits != false {
            os.write_bool(10, self.update_risk_limits).unwrap();
        }
        if !self.actor.is_empty() {
            os.write_string(11, &self.actor).unwrap();
        }
        if !self.reason.is_empty() {
            os.write_string(12, &self.reason).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                10 => {
                    self.account_id = is.read_string().map_err(pb_error_to_status)?;
                }
                18 => {
                    self.display_name = is.read_string().map_err(pb_error_to_status)?;
                }
                24 => {
                    self.update_display_name = is.read_bool().map_err(pb_error_to_status)?;
                }
                34 => {
                    self.strategy_task_id = is.read_string().map_err(pb_error_to_status)?;
                }
                42 => {
                    self.run_id = is.read_string().map_err(pb_error_to_status)?;
                }
                48 => {
                    self.update_binding = is.read_bool().map_err(pb_error_to_status)?;
                }
                56 => {
                    let value =
                        SimAccountStatus::from_i32(is.read_int32().map_err(pb_error_to_status)?);
                    self.status = value;
                }
                64 => {
                    self.update_status = is.read_bool().map_err(pb_error_to_status)?;
                }
                74 => {
                    let len = is.read_raw_varint64();
                    let len = len.map_err(pb_error_to_status)?;
                    let old_limit = is.push_limit(len);
                    let old_limit = old_limit.unwrap();
                    self.risk_limits.parse_from_input_stream(is)?;
                    is.pop_limit(old_limit);
                }
                80 => {
                    self.update_risk_limits = is.read_bool().map_err(pb_error_to_status)?;
                }
                90 => {
                    self.actor = is.read_string().map_err(pb_error_to_status)?;
                }
                98 => {
                    self.reason = is.read_string().map_err(pb_error_to_status)?;
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for CreateSimAccountRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateSimAccountRequest")
            .field("account_id", &self.account_id)
            .field("display_name", &self.display_name)
            .field("initial_cash", &self.initial_cash)
            .field("currency", &self.currency)
            .field("strategy_task_id", &self.strategy_task_id)
            .field("run_id", &self.run_id)
            .field("created_by", &self.created_by)
            .field("risk_limits", &self.risk_limits)
            .field("trading_engine", &self.trading_engine)
            .finish()
    }
}

impl Default for CreateSimAccountRequest {
    fn default() -> Self {
        CreateSimAccountRequest {
            account_id: String::default(),
            display_name: String::default(),
            initial_cash: 0.0,
            currency: String::default(),
            strategy_task_id: String::default(),
            run_id: String::default(),
            created_by: String::default(),
            risk_limits: Box::new(SimAccountRiskLimits::default()),
            trading_engine: SimAccountTradingEngine::from_i32(0),
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for CreateSimAccountRequest {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if !self.account_id.is_empty() {
            my_size += x_com_lib::rt::string_size(1, &self.account_id);
        }
        if !self.display_name.is_empty() {
            my_size += x_com_lib::rt::string_size(2, &self.display_name);
        }
        if self.initial_cash != 0.0 {
            my_size += 9;
        }
        if !self.currency.is_empty() {
            my_size += x_com_lib::rt::string_size(4, &self.currency);
        }
        if !self.strategy_task_id.is_empty() {
            my_size += x_com_lib::rt::string_size(5, &self.strategy_task_id);
        }
        if !self.run_id.is_empty() {
            my_size += x_com_lib::rt::string_size(6, &self.run_id);
        }
        if !self.created_by.is_empty() {
            my_size += x_com_lib::rt::string_size(7, &self.created_by);
        }
        {
            let len = self.risk_limits.compute_size();
            my_size += 1 + x_com_lib::rt::compute_raw_varint64_size(len) + len;
        }
        if let Some(value) = self.trading_engine.as_ref() {
            my_size += x_com_lib::rt::int32_size(9, *value as i32);
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        if !self.account_id.is_empty() {
            os.write_string(1, &self.account_id).unwrap();
        }
        if !self.display_name.is_empty() {
            os.write_string(2, &self.display_name).unwrap();
        }
        if self.initial_cash != 0.0 {
            os.write_double(3, self.initial_cash).unwrap();
        }
        if !self.currency.is_empty() {
            os.write_string(4, &self.currency).unwrap();
        }
        if !self.strategy_task_id.is_empty() {
            os.write_string(5, &self.strategy_task_id).unwrap();
        }
        if !self.run_id.is_empty() {
            os.write_string(6, &self.run_id).unwrap();
        }
        if !self.created_by.is_empty() {
            os.write_string(7, &self.created_by).unwrap();
        }
        {
            os.write_tag(8, x_com_lib::rt::WireType::LengthDelimited)
                .unwrap();
            os.write_raw_varint32(self.risk_limits.cached_size.get() as u32)
                .unwrap();
            self.risk_limits.serial_with_output_stream(os).unwrap();
        }
        if let Some(value) = self.trading_engine.as_ref() {
            os.write_enum(9, *value as i32).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                10 => {
                    self.account_id = is.read_string().map_err(pb_error_to_status)?;
                }
                18 => {
                    self.display_name = is.read_string().map_err(pb_error_to_status)?;
                }
                25 => {
                    self.initial_cash = is.read_double().map_err(pb_error_to_status)?;
                }
                34 => {
                    self.currency = is.read_string().map_err(pb_error_to_status)?;
                }
                42 => {
                    self.strategy_task_id = is.read_string().map_err(pb_error_to_status)?;
                }
                50 => {
                    self.run_id = is.read_string().map_err(pb_error_to_status)?;
                }
                58 => {
                    self.created_by = is.read_string().map_err(pb_error_to_status)?;
                }
                66 => {
                    let len = is.read_raw_varint64();
                    let len = len.map_err(pb_error_to_status)?;
                    let old_limit = is.push_limit(len);
                    let old_limit = old_limit.unwrap();
                    self.risk_limits.parse_from_input_stream(is)?;
                    is.pop_limit(old_limit);
                }
                72 => {
                    let value = SimAccountTradingEngine::from_i32(
                        is.read_int32().map_err(pb_error_to_status)?,
                    );
                    self.trading_engine = value;
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for SimAccountRiskLimits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimAccountRiskLimits")
            .field("max_single_order_notional", &self.max_single_order_notional)
            .field("max_daily_notional", &self.max_daily_notional)
            .field("max_open_order_count", &self.max_open_order_count)
            .field("max_contract_quantity", &self.max_contract_quantity)
            .field("max_quote_age_ms", &self.max_quote_age_ms)
            .field("max_spread_pct", &self.max_spread_pct)
            .field("max_abs_spread", &self.max_abs_spread)
            .field("allow_opening_trades", &self.allow_opening_trades)
            .field("allow_naked_short_options", &self.allow_naked_short_options)
            .finish()
    }
}

impl Default for SimAccountRiskLimits {
    fn default() -> Self {
        SimAccountRiskLimits {
            max_single_order_notional: None,
            max_daily_notional: None,
            max_open_order_count: None,
            max_contract_quantity: None,
            max_quote_age_ms: None,
            max_spread_pct: None,
            max_abs_spread: None,
            allow_opening_trades: None,
            allow_naked_short_options: None,
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for SimAccountRiskLimits {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if let Some(_v) = self.max_single_order_notional {
            my_size += 9;
        }
        if let Some(_v) = self.max_daily_notional {
            my_size += 9;
        }
        if let Some(v) = self.max_open_order_count {
            my_size += x_com_lib::rt::int32_size(3, v);
        }
        if let Some(v) = self.max_contract_quantity {
            my_size += x_com_lib::rt::int32_size(4, v);
        }
        if let Some(v) = self.max_quote_age_ms {
            my_size += x_com_lib::rt::int64_size(5, v);
        }
        if let Some(_v) = self.max_spread_pct {
            my_size += 9;
        }
        if let Some(_v) = self.max_abs_spread {
            my_size += 9;
        }
        if let Some(_v) = self.allow_opening_trades {
            my_size += 2;
        }
        if let Some(_v) = self.allow_naked_short_options {
            my_size += 2;
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        if let Some(v) = self.max_single_order_notional {
            os.write_double(1, v).unwrap();
        }
        if let Some(v) = self.max_daily_notional {
            os.write_double(2, v).unwrap();
        }
        if let Some(v) = self.max_open_order_count {
            os.write_int32(3, v).unwrap();
        }
        if let Some(v) = self.max_contract_quantity {
            os.write_int32(4, v).unwrap();
        }
        if let Some(v) = self.max_quote_age_ms {
            os.write_int64(5, v).unwrap();
        }
        if let Some(v) = self.max_spread_pct {
            os.write_double(6, v).unwrap();
        }
        if let Some(v) = self.max_abs_spread {
            os.write_double(7, v).unwrap();
        }
        if let Some(v) = self.allow_opening_trades {
            os.write_bool(8, v).unwrap();
        }
        if let Some(v) = self.allow_naked_short_options {
            os.write_bool(9, v).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                9 => {
                    self.max_single_order_notional =
                        Some(is.read_double().map_err(pb_error_to_status)?);
                }
                17 => {
                    self.max_daily_notional = Some(is.read_double().map_err(pb_error_to_status)?);
                }
                24 => {
                    self.max_open_order_count = Some(is.read_int32().map_err(pb_error_to_status)?);
                }
                32 => {
                    self.max_contract_quantity = Some(is.read_int32().map_err(pb_error_to_status)?);
                }
                40 => {
                    self.max_quote_age_ms = Some(is.read_int64().map_err(pb_error_to_status)?);
                }
                49 => {
                    self.max_spread_pct = Some(is.read_double().map_err(pb_error_to_status)?);
                }
                57 => {
                    self.max_abs_spread = Some(is.read_double().map_err(pb_error_to_status)?);
                }
                64 => {
                    self.allow_opening_trades = Some(is.read_bool().map_err(pb_error_to_status)?);
                }
                72 => {
                    self.allow_naked_short_options =
                        Some(is.read_bool().map_err(pb_error_to_status)?);
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for SimAccountList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimAccountList")
            .field("accounts", &self.accounts)
            .finish()
    }
}

impl Default for SimAccountList {
    fn default() -> Self {
        SimAccountList {
            accounts: Vec::new(),
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for SimAccountList {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        for value in &self.accounts {
            let len = value.compute_size();
            my_size += 1 + x_com_lib::rt::compute_raw_varint64_size(len) + len;
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        for v in &self.accounts {
            os.write_tag(1, x_com_lib::rt::WireType::LengthDelimited)
                .unwrap();
            os.write_raw_varint32(v.cached_size.get() as u32).unwrap();
            v.serial_with_output_stream(os).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                10 => {
                    let len = is.read_raw_varint64();
                    let len = len.map_err(pb_error_to_status)?;
                    let old_limit = is.push_limit(len);
                    let old_limit = old_limit.unwrap();
                    let mut value = Box::new(SimAccountInfo::default());
                    value.parse_from_input_stream(is)?;
                    self.accounts.push(value);
                    is.pop_limit(old_limit);
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for ListSimAccountAuditEventsRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListSimAccountAuditEventsRequest")
            .field("account_id", &self.account_id)
            .field("start_time", &self.start_time)
            .field("end_time", &self.end_time)
            .field("limit", &self.limit)
            .finish()
    }
}

impl Default for ListSimAccountAuditEventsRequest {
    fn default() -> Self {
        ListSimAccountAuditEventsRequest {
            account_id: String::default(),
            start_time: 0,
            end_time: 0,
            limit: 0,
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for ListSimAccountAuditEventsRequest {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if !self.account_id.is_empty() {
            my_size += x_com_lib::rt::string_size(1, &self.account_id);
        }
        if self.start_time != 0 {
            my_size += x_com_lib::rt::int64_size(2, self.start_time);
        }
        if self.end_time != 0 {
            my_size += x_com_lib::rt::int64_size(3, self.end_time);
        }
        if self.limit != 0 {
            my_size += x_com_lib::rt::int32_size(4, self.limit);
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        if !self.account_id.is_empty() {
            os.write_string(1, &self.account_id).unwrap();
        }
        if self.start_time != 0 {
            os.write_int64(2, self.start_time).unwrap();
        }
        if self.end_time != 0 {
            os.write_int64(3, self.end_time).unwrap();
        }
        if self.limit != 0 {
            os.write_int32(4, self.limit).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                10 => {
                    self.account_id = is.read_string().map_err(pb_error_to_status)?;
                }
                16 => {
                    self.start_time = is.read_int64().map_err(pb_error_to_status)?;
                }
                24 => {
                    self.end_time = is.read_int64().map_err(pb_error_to_status)?;
                }
                32 => {
                    self.limit = is.read_int32().map_err(pb_error_to_status)?;
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for SimAccountAuditEventList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimAccountAuditEventList")
            .field("events", &self.events)
            .finish()
    }
}

impl Default for SimAccountAuditEventList {
    fn default() -> Self {
        SimAccountAuditEventList {
            events: Vec::new(),
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for SimAccountAuditEventList {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        for value in &self.events {
            let len = value.compute_size();
            my_size += 1 + x_com_lib::rt::compute_raw_varint64_size(len) + len;
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        for v in &self.events {
            os.write_tag(1, x_com_lib::rt::WireType::LengthDelimited)
                .unwrap();
            os.write_raw_varint32(v.cached_size.get() as u32).unwrap();
            v.serial_with_output_stream(os).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                10 => {
                    let len = is.read_raw_varint64();
                    let len = len.map_err(pb_error_to_status)?;
                    let old_limit = is.push_limit(len);
                    let old_limit = old_limit.unwrap();
                    let mut value = Box::new(SimAccountAuditEvent::default());
                    value.parse_from_input_stream(is)?;
                    self.events.push(value);
                    is.pop_limit(old_limit);
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for SimAccountIdRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimAccountIdRequest")
            .field("account_id", &self.account_id)
            .finish()
    }
}

impl Default for SimAccountIdRequest {
    fn default() -> Self {
        SimAccountIdRequest {
            account_id: String::default(),
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for SimAccountIdRequest {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if !self.account_id.is_empty() {
            my_size += x_com_lib::rt::string_size(1, &self.account_id);
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        if !self.account_id.is_empty() {
            os.write_string(1, &self.account_id).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                10 => {
                    self.account_id = is.read_string().map_err(pb_error_to_status)?;
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for SimAccountInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimAccountInfo")
            .field("account_id", &self.account_id)
            .field("display_name", &self.display_name)
            .field("initial_cash", &self.initial_cash)
            .field("currency", &self.currency)
            .field("status", &self.status)
            .field("strategy_task_id", &self.strategy_task_id)
            .field("run_id", &self.run_id)
            .field("created_by", &self.created_by)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .field("risk_limits", &self.risk_limits)
            .field("trading_engine", &self.trading_engine)
            .finish()
    }
}

impl Default for SimAccountInfo {
    fn default() -> Self {
        SimAccountInfo {
            account_id: String::default(),
            display_name: String::default(),
            initial_cash: 0.0,
            currency: String::default(),
            status: SimAccountStatus::from_i32(0),
            strategy_task_id: String::default(),
            run_id: String::default(),
            created_by: String::default(),
            created_at: 0,
            updated_at: 0,
            risk_limits: Box::new(SimAccountRiskLimits::default()),
            trading_engine: SimAccountTradingEngine::from_i32(0),
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for SimAccountInfo {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if !self.account_id.is_empty() {
            my_size += x_com_lib::rt::string_size(1, &self.account_id);
        }
        if !self.display_name.is_empty() {
            my_size += x_com_lib::rt::string_size(2, &self.display_name);
        }
        if self.initial_cash != 0.0 {
            my_size += 9;
        }
        if !self.currency.is_empty() {
            my_size += x_com_lib::rt::string_size(4, &self.currency);
        }
        if let Some(value) = self.status.as_ref() {
            my_size += x_com_lib::rt::int32_size(5, *value as i32);
        }
        if !self.strategy_task_id.is_empty() {
            my_size += x_com_lib::rt::string_size(6, &self.strategy_task_id);
        }
        if !self.run_id.is_empty() {
            my_size += x_com_lib::rt::string_size(7, &self.run_id);
        }
        if !self.created_by.is_empty() {
            my_size += x_com_lib::rt::string_size(8, &self.created_by);
        }
        if self.created_at != 0 {
            my_size += x_com_lib::rt::int64_size(9, self.created_at);
        }
        if self.updated_at != 0 {
            my_size += x_com_lib::rt::int64_size(10, self.updated_at);
        }
        {
            let len = self.risk_limits.compute_size();
            my_size += 1 + x_com_lib::rt::compute_raw_varint64_size(len) + len;
        }
        if let Some(value) = self.trading_engine.as_ref() {
            my_size += x_com_lib::rt::int32_size(12, *value as i32);
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        if !self.account_id.is_empty() {
            os.write_string(1, &self.account_id).unwrap();
        }
        if !self.display_name.is_empty() {
            os.write_string(2, &self.display_name).unwrap();
        }
        if self.initial_cash != 0.0 {
            os.write_double(3, self.initial_cash).unwrap();
        }
        if !self.currency.is_empty() {
            os.write_string(4, &self.currency).unwrap();
        }
        if let Some(value) = self.status.as_ref() {
            os.write_enum(5, *value as i32).unwrap();
        }
        if !self.strategy_task_id.is_empty() {
            os.write_string(6, &self.strategy_task_id).unwrap();
        }
        if !self.run_id.is_empty() {
            os.write_string(7, &self.run_id).unwrap();
        }
        if !self.created_by.is_empty() {
            os.write_string(8, &self.created_by).unwrap();
        }
        if self.created_at != 0 {
            os.write_int64(9, self.created_at).unwrap();
        }
        if self.updated_at != 0 {
            os.write_int64(10, self.updated_at).unwrap();
        }
        {
            os.write_tag(11, x_com_lib::rt::WireType::LengthDelimited)
                .unwrap();
            os.write_raw_varint32(self.risk_limits.cached_size.get() as u32)
                .unwrap();
            self.risk_limits.serial_with_output_stream(os).unwrap();
        }
        if let Some(value) = self.trading_engine.as_ref() {
            os.write_enum(12, *value as i32).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                10 => {
                    self.account_id = is.read_string().map_err(pb_error_to_status)?;
                }
                18 => {
                    self.display_name = is.read_string().map_err(pb_error_to_status)?;
                }
                25 => {
                    self.initial_cash = is.read_double().map_err(pb_error_to_status)?;
                }
                34 => {
                    self.currency = is.read_string().map_err(pb_error_to_status)?;
                }
                40 => {
                    let value =
                        SimAccountStatus::from_i32(is.read_int32().map_err(pb_error_to_status)?);
                    self.status = value;
                }
                50 => {
                    self.strategy_task_id = is.read_string().map_err(pb_error_to_status)?;
                }
                58 => {
                    self.run_id = is.read_string().map_err(pb_error_to_status)?;
                }
                66 => {
                    self.created_by = is.read_string().map_err(pb_error_to_status)?;
                }
                72 => {
                    self.created_at = is.read_int64().map_err(pb_error_to_status)?;
                }
                80 => {
                    self.updated_at = is.read_int64().map_err(pb_error_to_status)?;
                }
                90 => {
                    let len = is.read_raw_varint64();
                    let len = len.map_err(pb_error_to_status)?;
                    let old_limit = is.push_limit(len);
                    let old_limit = old_limit.unwrap();
                    self.risk_limits.parse_from_input_stream(is)?;
                    is.pop_limit(old_limit);
                }
                96 => {
                    let value = SimAccountTradingEngine::from_i32(
                        is.read_int32().map_err(pb_error_to_status)?,
                    );
                    self.trading_engine = value;
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for ListSimAccountsRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListSimAccountsRequest")
            .field("status", &self.status)
            .field("strategy_task_id", &self.strategy_task_id)
            .field("run_id", &self.run_id)
            .field("query", &self.query)
            .field("include_archived", &self.include_archived)
            .field("limit", &self.limit)
            .finish()
    }
}

impl Default for ListSimAccountsRequest {
    fn default() -> Self {
        ListSimAccountsRequest {
            status: SimAccountStatus::from_i32(0),
            strategy_task_id: String::default(),
            run_id: String::default(),
            query: String::default(),
            include_archived: false,
            limit: 0,
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for ListSimAccountsRequest {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if let Some(value) = self.status.as_ref() {
            my_size += x_com_lib::rt::int32_size(1, *value as i32);
        }
        if !self.strategy_task_id.is_empty() {
            my_size += x_com_lib::rt::string_size(2, &self.strategy_task_id);
        }
        if !self.run_id.is_empty() {
            my_size += x_com_lib::rt::string_size(3, &self.run_id);
        }
        if !self.query.is_empty() {
            my_size += x_com_lib::rt::string_size(4, &self.query);
        }
        if self.include_archived != false {
            my_size += 2;
        }
        if self.limit != 0 {
            my_size += x_com_lib::rt::int32_size(6, self.limit);
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        if let Some(value) = self.status.as_ref() {
            os.write_enum(1, *value as i32).unwrap();
        }
        if !self.strategy_task_id.is_empty() {
            os.write_string(2, &self.strategy_task_id).unwrap();
        }
        if !self.run_id.is_empty() {
            os.write_string(3, &self.run_id).unwrap();
        }
        if !self.query.is_empty() {
            os.write_string(4, &self.query).unwrap();
        }
        if self.include_archived != false {
            os.write_bool(5, self.include_archived).unwrap();
        }
        if self.limit != 0 {
            os.write_int32(6, self.limit).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                8 => {
                    let value =
                        SimAccountStatus::from_i32(is.read_int32().map_err(pb_error_to_status)?);
                    self.status = value;
                }
                18 => {
                    self.strategy_task_id = is.read_string().map_err(pb_error_to_status)?;
                }
                26 => {
                    self.run_id = is.read_string().map_err(pb_error_to_status)?;
                }
                34 => {
                    self.query = is.read_string().map_err(pb_error_to_status)?;
                }
                40 => {
                    self.include_archived = is.read_bool().map_err(pb_error_to_status)?;
                }
                48 => {
                    self.limit = is.read_int32().map_err(pb_error_to_status)?;
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for SimAccountServiceHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimAccountServiceHealth")
            .field("sqlite_path", &self.sqlite_path)
            .field("account_count", &self.account_count)
            .field("audit_event_count", &self.audit_event_count)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl Default for SimAccountServiceHealth {
    fn default() -> Self {
        SimAccountServiceHealth {
            sqlite_path: String::default(),
            account_count: 0,
            audit_event_count: 0,
            updated_at: 0,
            cached_size: x_com_lib::rt::CachedSize::new(),
        }
    }
}

impl RequestMessage for SimAccountServiceHealth {
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if !self.sqlite_path.is_empty() {
            my_size += x_com_lib::rt::string_size(1, &self.sqlite_path);
        }
        if self.account_count != 0 {
            my_size += x_com_lib::rt::int64_size(2, self.account_count);
        }
        if self.audit_event_count != 0 {
            my_size += x_com_lib::rt::int64_size(3, self.audit_event_count);
        }
        if self.updated_at != 0 {
            my_size += x_com_lib::rt::int64_size(4, self.updated_at);
        }
        self.cached_size.set(my_size as u32);
        my_size
    }

    fn serial_with_output_stream(
        &self,
        os: &mut x_com_lib::CodedOutputStream<'_>,
    ) -> Result<(), Status> {
        if !self.sqlite_path.is_empty() {
            os.write_string(1, &self.sqlite_path).unwrap();
        }
        if self.account_count != 0 {
            os.write_int64(2, self.account_count).unwrap();
        }
        if self.audit_event_count != 0 {
            os.write_int64(3, self.audit_event_count).unwrap();
        }
        if self.updated_at != 0 {
            os.write_int64(4, self.updated_at).unwrap();
        }
        Ok(())
    }

    fn parse_from_input_stream(
        &mut self,
        is: &mut x_com_lib::CodedInputStream<'_>,
    ) -> Result<(), Status> {
        while let Some(tag) = is.read_raw_tag_or_eof().unwrap() {
            match tag {
                10 => {
                    self.sqlite_path = is.read_string().map_err(pb_error_to_status)?;
                }
                16 => {
                    self.account_count = is.read_int64().map_err(pb_error_to_status)?;
                }
                24 => {
                    self.audit_event_count = is.read_int64().map_err(pb_error_to_status)?;
                }
                32 => {
                    self.updated_at = is.read_int64().map_err(pb_error_to_status)?;
                }
                _ => {
                    let wire_type = extract_wire_type_from_tag(tag);
                    if wire_type.is_none() {
                        return Err(Status::error("消息格式出错".into()));
                    }
                    let result = is.skip_field(wire_type.unwrap());
                    if let Err(err) = result {
                        return Err(Status::error(err.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl SimAccountTradingEngine {
    pub fn from_i32(value: i32) -> Option<SimAccountTradingEngine> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::DqteaSim),
            2 => Some(Self::MoomooSimulate),
            _ => None,
        }
    }
}

impl SimAccountAuditAction {
    pub fn from_i32(value: i32) -> Option<SimAccountAuditAction> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::Created),
            2 => Some(Self::Updated),
            3 => Some(Self::StatusChanged),
            4 => Some(Self::BindingChanged),
            5 => Some(Self::RiskLimitsChanged),
            _ => None,
        }
    }
}

impl SimAccountStatus {
    pub fn from_i32(value: i32) -> Option<SimAccountStatus> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::Active),
            2 => Some(Self::Paused),
            3 => Some(Self::Archived),
            _ => None,
        }
    }
}
