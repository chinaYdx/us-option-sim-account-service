use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use x_com_lib::{time_utils, Status};

use crate::x_com::source_api::{
    SimAccountAuditAction, SimAccountAuditEvent, SimAccountInfo, SimAccountRiskLimits,
    SimAccountStatus, SimAccountTradingEngine,
};

type XResult<T> = x_com_lib::x_core::Result<T>;
static EVENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct SimAccountStore {
    path: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

#[derive(Clone, Debug)]
pub struct AccountPatch {
    pub display_name: Option<String>,
    pub strategy_task_id: Option<String>,
    pub run_id: Option<String>,
    pub status: Option<i32>,
    pub risk_limits: Option<Box<SimAccountRiskLimits>>,
    pub actor: String,
    pub reason: String,
}

impl SimAccountStore {
    pub fn unopened() -> Self {
        let conn = Connection::open_in_memory().expect("open in-memory sim account placeholder");
        SimAccountStore {
            path: PathBuf::new(),
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn open(path: PathBuf) -> XResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(status_from_error)?;
        }
        let conn = Connection::open(&path).map_err(status_from_error)?;
        let store = SimAccountStore {
            path,
            conn: Arc::new(Mutex::new(conn)),
        };
        store.init_schema()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn create_account(
        &self,
        account: &SimAccountInfo,
        action: SimAccountAuditAction,
        actor: &str,
        reason: &str,
    ) -> XResult<()> {
        let mut conn = self.conn.lock().map_err(status_from_error)?;
        let tx = conn.transaction().map_err(status_from_error)?;
        insert_account_tx(&tx, account)?;
        upsert_risk_limits_tx(
            &tx,
            &account.account_id,
            &account.risk_limits,
            account.updated_at,
        )?;
        insert_audit_tx(
            &tx,
            &new_event_id(account.updated_at),
            &account.account_id,
            action,
            actor,
            reason,
            0,
            status_value(account.status),
            "",
            &account.strategy_task_id,
            "",
            &account.run_id,
            true,
            account.updated_at,
        )?;
        tx.commit().map_err(status_from_error)
    }

    pub fn get_account(&self, account_id: &str) -> XResult<Option<Box<SimAccountInfo>>> {
        let conn = self.conn.lock().map_err(status_from_error)?;
        get_account_conn(&conn, account_id)
    }

    pub fn list_accounts(
        &self,
        status: Option<i32>,
        strategy_task_id: &str,
        run_id: &str,
        query: &str,
        include_archived: bool,
        limit: i32,
    ) -> XResult<Vec<Box<SimAccountInfo>>> {
        let conn = self.conn.lock().map_err(status_from_error)?;
        let mut sql = "SELECT account_id FROM sim_account WHERE 1 = 1".to_owned();
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(status) = status {
            sql.push_str(" AND status = ?");
            args.push(Box::new(status));
        } else if !include_archived {
            sql.push_str(" AND status <> ?");
            args.push(Box::new(SimAccountStatus::Archived as i32));
        }
        if !strategy_task_id.is_empty() {
            sql.push_str(" AND strategy_task_id = ?");
            args.push(Box::new(strategy_task_id.to_owned()));
        }
        if !run_id.is_empty() {
            sql.push_str(" AND run_id = ?");
            args.push(Box::new(run_id.to_owned()));
        }
        if !query.is_empty() {
            sql.push_str(" AND (account_id LIKE ? OR display_name LIKE ?)");
            let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
            args.push(Box::new(pattern.clone()));
            args.push(Box::new(pattern));
        }
        sql.push_str(" ORDER BY updated_at DESC, account_id ASC LIMIT ?");
        args.push(Box::new(normalize_limit(limit)));
        let arg_refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|arg| arg.as_ref()).collect();
        let mut stmt = conn.prepare(&sql).map_err(status_from_error)?;
        let ids = stmt
            .query_map(arg_refs.as_slice(), |row| row.get::<_, String>(0))
            .map_err(status_from_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(status_from_error)?;
        ids.into_iter()
            .map(|id| {
                get_account_conn(&conn, &id)?.ok_or_else(|| {
                    Status::error(format!("account disappeared while listing: {id}"))
                })
            })
            .collect()
    }

    pub fn update_account(
        &self,
        account_id: &str,
        patch: AccountPatch,
    ) -> XResult<Option<Box<SimAccountInfo>>> {
        let mut conn = self.conn.lock().map_err(status_from_error)?;
        let tx = conn.transaction().map_err(status_from_error)?;
        let Some(mut account) = get_account_tx(&tx, account_id)? else {
            return Ok(None);
        };
        let old_status = status_value(account.status);
        let old_strategy_task_id = account.strategy_task_id.clone();
        let old_run_id = account.run_id.clone();
        let mut action = SimAccountAuditAction::Updated;
        let mut risk_limits_changed = false;
        let now = time_utils::cur_timestamp();

        if let Some(display_name) = patch.display_name {
            account.display_name = display_name;
        }
        if let Some(strategy_task_id) = patch.strategy_task_id {
            account.strategy_task_id = strategy_task_id;
            account.run_id = patch.run_id.unwrap_or_default();
            action = SimAccountAuditAction::BindingChanged;
        }
        if let Some(status) = patch.status {
            account.status = status_from_i32(status);
            action = SimAccountAuditAction::StatusChanged;
        }
        if let Some(risk_limits) = patch.risk_limits {
            account.risk_limits = risk_limits;
            risk_limits_changed = true;
            action = SimAccountAuditAction::RiskLimitsChanged;
        }
        account.updated_at = now;
        update_account_tx(&tx, &account)?;
        if risk_limits_changed {
            upsert_risk_limits_tx(&tx, account_id, &account.risk_limits, now)?;
        }
        insert_audit_tx(
            &tx,
            &new_event_id(now),
            account_id,
            action,
            &patch.actor,
            &patch.reason,
            old_status,
            status_value(account.status),
            &old_strategy_task_id,
            &account.strategy_task_id,
            &old_run_id,
            &account.run_id,
            risk_limits_changed,
            now,
        )?;
        tx.commit().map_err(status_from_error)?;
        Ok(Some(account))
    }

    pub fn list_audit_events(
        &self,
        account_id: &str,
        start_time: i64,
        end_time: i64,
        limit: i32,
    ) -> XResult<Vec<Box<SimAccountAuditEvent>>> {
        let conn = self.conn.lock().map_err(status_from_error)?;
        let mut sql = "SELECT event_id, account_id, action, actor, reason, old_status, new_status, old_strategy_task_id, new_strategy_task_id, old_run_id, new_run_id, risk_limits_changed, event_at FROM sim_account_audit_event WHERE 1 = 1".to_owned();
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if !account_id.is_empty() {
            sql.push_str(" AND account_id = ?");
            args.push(Box::new(account_id.to_owned()));
        }
        if start_time > 0 {
            sql.push_str(" AND event_at >= ?");
            args.push(Box::new(start_time));
        }
        if end_time > 0 {
            sql.push_str(" AND event_at <= ?");
            args.push(Box::new(end_time));
        }
        sql.push_str(" ORDER BY event_at DESC, event_id DESC LIMIT ?");
        args.push(Box::new(normalize_limit(limit)));
        let arg_refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|arg| arg.as_ref()).collect();
        let mut stmt = conn.prepare(&sql).map_err(status_from_error)?;
        let events = stmt
            .query_map(arg_refs.as_slice(), audit_from_row)
            .map_err(status_from_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(status_from_error)?;
        Ok(events)
    }

    pub fn counts(&self) -> XResult<(i64, i64)> {
        let conn = self.conn.lock().map_err(status_from_error)?;
        let accounts = count_table(&conn, "sim_account")?;
        let events = count_table(&conn, "sim_account_audit_event")?;
        Ok((accounts, events))
    }

    fn init_schema(&self) -> XResult<()> {
        let conn = self.conn.lock().map_err(status_from_error)?;
        conn.execute_batch(
            "
CREATE TABLE IF NOT EXISTS sim_account (
  account_id TEXT PRIMARY KEY NOT NULL,
  display_name TEXT NOT NULL,
  initial_cash REAL NOT NULL,
  currency TEXT NOT NULL,
  status INTEGER NOT NULL,
  strategy_task_id TEXT NOT NULL,
  run_id TEXT NOT NULL,
  created_by TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS sim_account_risk_limits (
  account_id TEXT PRIMARY KEY NOT NULL,
  max_single_order_notional REAL,
  max_daily_notional REAL,
  max_open_order_count INTEGER,
  max_contract_quantity INTEGER,
  max_quote_age_ms INTEGER,
  max_spread_pct REAL,
  max_abs_spread REAL,
  allow_opening_trades INTEGER,
  allow_naked_short_options INTEGER,
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS sim_account_audit_event (
  event_id TEXT PRIMARY KEY NOT NULL,
  account_id TEXT NOT NULL,
  action INTEGER NOT NULL,
  actor TEXT NOT NULL,
  reason TEXT NOT NULL,
  old_status INTEGER NOT NULL,
  new_status INTEGER NOT NULL,
  old_strategy_task_id TEXT NOT NULL,
  new_strategy_task_id TEXT NOT NULL,
  old_run_id TEXT NOT NULL,
  new_run_id TEXT NOT NULL,
  risk_limits_changed INTEGER NOT NULL,
  event_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sim_account_task_run ON sim_account(strategy_task_id, run_id);
CREATE INDEX IF NOT EXISTS idx_sim_account_status ON sim_account(status);
CREATE INDEX IF NOT EXISTS idx_sim_account_audit_account_time ON sim_account_audit_event(account_id, event_at);
",
        )
        .map_err(status_from_error)?;
        ensure_column(
            &conn,
            "sim_account",
            "trading_engine",
            "INTEGER NOT NULL DEFAULT 1",
        )?;
        Ok(())
    }
}

fn insert_account_tx(tx: &Transaction<'_>, account: &SimAccountInfo) -> XResult<()> {
    tx.execute(
        "INSERT INTO sim_account(account_id, display_name, initial_cash, currency, status, strategy_task_id, run_id, created_by, created_at, updated_at, trading_engine) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            account.account_id,
            account.display_name,
            account.initial_cash,
            account.currency,
            status_value(account.status),
            account.strategy_task_id,
            account.run_id,
            account.created_by,
            account.created_at,
            account.updated_at,
            trading_engine_value(account.trading_engine)
        ],
    )
    .map(|_| ())
    .map_err(status_from_error)
}

fn update_account_tx(tx: &Transaction<'_>, account: &SimAccountInfo) -> XResult<()> {
    tx.execute(
        "UPDATE sim_account SET display_name = ?, status = ?, strategy_task_id = ?, run_id = ?, updated_at = ? WHERE account_id = ?",
        params![
            account.display_name,
            status_value(account.status),
            account.strategy_task_id,
            account.run_id,
            account.updated_at,
            account.account_id
        ],
    )
    .map(|_| ())
    .map_err(status_from_error)
}

fn upsert_risk_limits_tx(
    tx: &Transaction<'_>,
    account_id: &str,
    risk: &SimAccountRiskLimits,
    updated_at: i64,
) -> XResult<()> {
    tx.execute(
        "INSERT INTO sim_account_risk_limits(account_id, max_single_order_notional, max_daily_notional, max_open_order_count, max_contract_quantity, max_quote_age_ms, max_spread_pct, max_abs_spread, allow_opening_trades, allow_naked_short_options, updated_at)
         VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(account_id) DO UPDATE SET
           max_single_order_notional = excluded.max_single_order_notional,
           max_daily_notional = excluded.max_daily_notional,
           max_open_order_count = excluded.max_open_order_count,
           max_contract_quantity = excluded.max_contract_quantity,
           max_quote_age_ms = excluded.max_quote_age_ms,
           max_spread_pct = excluded.max_spread_pct,
           max_abs_spread = excluded.max_abs_spread,
           allow_opening_trades = excluded.allow_opening_trades,
           allow_naked_short_options = excluded.allow_naked_short_options,
           updated_at = excluded.updated_at",
        params![
            account_id,
            risk.max_single_order_notional,
            risk.max_daily_notional,
            risk.max_open_order_count,
            risk.max_contract_quantity,
            risk.max_quote_age_ms,
            risk.max_spread_pct,
            risk.max_abs_spread,
            option_bool_to_i64(risk.allow_opening_trades),
            option_bool_to_i64(risk.allow_naked_short_options),
            updated_at
        ],
    )
    .map(|_| ())
    .map_err(status_from_error)
}

#[allow(clippy::too_many_arguments)]
fn insert_audit_tx(
    tx: &Transaction<'_>,
    event_id: &str,
    account_id: &str,
    action: SimAccountAuditAction,
    actor: &str,
    reason: &str,
    old_status: i32,
    new_status: i32,
    old_strategy_task_id: &str,
    new_strategy_task_id: &str,
    old_run_id: &str,
    new_run_id: &str,
    risk_limits_changed: bool,
    event_at: i64,
) -> XResult<()> {
    tx.execute(
        "INSERT INTO sim_account_audit_event(event_id, account_id, action, actor, reason, old_status, new_status, old_strategy_task_id, new_strategy_task_id, old_run_id, new_run_id, risk_limits_changed, event_at) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            event_id,
            account_id,
            action as i32,
            actor,
            reason,
            old_status,
            new_status,
            old_strategy_task_id,
            new_strategy_task_id,
            old_run_id,
            new_run_id,
            if risk_limits_changed { 1 } else { 0 },
            event_at
        ],
    )
    .map(|_| ())
    .map_err(status_from_error)
}

fn get_account_conn(conn: &Connection, account_id: &str) -> XResult<Option<Box<SimAccountInfo>>> {
    let mut stmt = conn
        .prepare("SELECT account_id, display_name, initial_cash, currency, status, strategy_task_id, run_id, created_by, created_at, updated_at, trading_engine FROM sim_account WHERE account_id = ?")
        .map_err(status_from_error)?;
    let account = stmt
        .query_row(params![account_id], account_from_row)
        .optional()
        .map_err(status_from_error)?;
    let Some(mut account) = account else {
        return Ok(None);
    };
    account.risk_limits = get_risk_limits_conn(conn, account_id)?;
    Ok(Some(account))
}

fn get_account_tx(tx: &Transaction<'_>, account_id: &str) -> XResult<Option<Box<SimAccountInfo>>> {
    let mut stmt = tx
        .prepare("SELECT account_id, display_name, initial_cash, currency, status, strategy_task_id, run_id, created_by, created_at, updated_at, trading_engine FROM sim_account WHERE account_id = ?")
        .map_err(status_from_error)?;
    let account = stmt
        .query_row(params![account_id], account_from_row)
        .optional()
        .map_err(status_from_error)?;
    let Some(mut account) = account else {
        return Ok(None);
    };
    account.risk_limits = get_risk_limits_tx(tx, account_id)?;
    Ok(Some(account))
}

fn get_risk_limits_conn(conn: &Connection, account_id: &str) -> XResult<Box<SimAccountRiskLimits>> {
    let mut stmt = conn.prepare("SELECT max_single_order_notional, max_daily_notional, max_open_order_count, max_contract_quantity, max_quote_age_ms, max_spread_pct, max_abs_spread, allow_opening_trades, allow_naked_short_options FROM sim_account_risk_limits WHERE account_id = ?").map_err(status_from_error)?;
    stmt.query_row(params![account_id], risk_from_row)
        .optional()
        .map(|value| value.unwrap_or_else(|| Box::new(SimAccountRiskLimits::default())))
        .map_err(status_from_error)
}

fn get_risk_limits_tx(
    tx: &Transaction<'_>,
    account_id: &str,
) -> XResult<Box<SimAccountRiskLimits>> {
    let mut stmt = tx.prepare("SELECT max_single_order_notional, max_daily_notional, max_open_order_count, max_contract_quantity, max_quote_age_ms, max_spread_pct, max_abs_spread, allow_opening_trades, allow_naked_short_options FROM sim_account_risk_limits WHERE account_id = ?").map_err(status_from_error)?;
    stmt.query_row(params![account_id], risk_from_row)
        .optional()
        .map(|value| value.unwrap_or_else(|| Box::new(SimAccountRiskLimits::default())))
        .map_err(status_from_error)
}

fn account_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Box<SimAccountInfo>> {
    let mut account = Box::new(SimAccountInfo::default());
    account.account_id = row.get(0)?;
    account.display_name = row.get(1)?;
    account.initial_cash = row.get(2)?;
    account.currency = row.get(3)?;
    account.status = status_from_i32(row.get::<_, i32>(4)?);
    account.strategy_task_id = row.get(5)?;
    account.run_id = row.get(6)?;
    account.created_by = row.get(7)?;
    account.created_at = row.get(8)?;
    account.updated_at = row.get(9)?;
    account.trading_engine = trading_engine_from_i32(row.get::<_, i32>(10)?);
    Ok(account)
}

fn risk_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Box<SimAccountRiskLimits>> {
    let mut risk = Box::new(SimAccountRiskLimits::default());
    risk.max_single_order_notional = row.get(0)?;
    risk.max_daily_notional = row.get(1)?;
    risk.max_open_order_count = row.get(2)?;
    risk.max_contract_quantity = row.get(3)?;
    risk.max_quote_age_ms = row.get(4)?;
    risk.max_spread_pct = row.get(5)?;
    risk.max_abs_spread = row.get(6)?;
    risk.allow_opening_trades = option_i64_to_bool(row.get(7)?);
    risk.allow_naked_short_options = option_i64_to_bool(row.get(8)?);
    Ok(risk)
}

fn audit_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Box<SimAccountAuditEvent>> {
    let mut event = Box::new(SimAccountAuditEvent::default());
    event.event_id = row.get(0)?;
    event.account_id = row.get(1)?;
    event.action = audit_action_from_i32(row.get::<_, i32>(2)?);
    event.actor = row.get(3)?;
    event.reason = row.get(4)?;
    event.old_status = status_from_i32(row.get::<_, i32>(5)?);
    event.new_status = status_from_i32(row.get::<_, i32>(6)?);
    event.old_strategy_task_id = row.get(7)?;
    event.new_strategy_task_id = row.get(8)?;
    event.old_run_id = row.get(9)?;
    event.new_run_id = row.get(10)?;
    event.risk_limits_changed = row.get::<_, i32>(11)? != 0;
    event.event_at = row.get(12)?;
    Ok(event)
}

fn count_table(conn: &Connection, table: &str) -> XResult<i64> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    conn.query_row(&sql, [], |row| row.get(0))
        .map_err(status_from_error)
}

pub fn status_from_i32(value: i32) -> Option<SimAccountStatus> {
    match value {
        1 => Some(SimAccountStatus::Active),
        2 => Some(SimAccountStatus::Paused),
        3 => Some(SimAccountStatus::Archived),
        _ => Some(SimAccountStatus::Unknown),
    }
}

pub fn status_value(value: Option<SimAccountStatus>) -> i32 {
    value.unwrap_or(SimAccountStatus::Unknown) as i32
}

pub fn trading_engine_from_i32(value: i32) -> Option<SimAccountTradingEngine> {
    match value {
        1 => Some(SimAccountTradingEngine::DqteaSim),
        2 => Some(SimAccountTradingEngine::MoomooSimulate),
        _ => Some(SimAccountTradingEngine::Unknown),
    }
}

pub fn trading_engine_value(value: Option<SimAccountTradingEngine>) -> i32 {
    value.unwrap_or(SimAccountTradingEngine::DqteaSim) as i32
}

fn audit_action_from_i32(value: i32) -> Option<SimAccountAuditAction> {
    match value {
        1 => Some(SimAccountAuditAction::Created),
        2 => Some(SimAccountAuditAction::Updated),
        3 => Some(SimAccountAuditAction::StatusChanged),
        4 => Some(SimAccountAuditAction::BindingChanged),
        5 => Some(SimAccountAuditAction::RiskLimitsChanged),
        _ => Some(SimAccountAuditAction::Unknown),
    }
}

fn option_bool_to_i64(value: Option<bool>) -> Option<i64> {
    value.map(|flag| if flag { 1 } else { 0 })
}

fn option_i64_to_bool(value: Option<i64>) -> Option<bool> {
    value.map(|raw| raw != 0)
}

fn normalize_limit(limit: i32) -> i32 {
    if limit <= 0 {
        100
    } else {
        limit.min(1000)
    }
}

fn new_event_id(now: i64) -> String {
    let seq = EVENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("sim-account-event-{now}-{seq}")
}

fn status_from_error(error: impl std::fmt::Display) -> Status {
    Status::error(error.to_string())
}

fn ensure_column(conn: &Connection, table: &str, column: &str, column_sql: &str) -> XResult<()> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(status_from_error)?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(status_from_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(status_from_error)?;
    if columns.iter().any(|name| name == column) {
        return Ok(());
    }
    conn.execute_batch(&format!(
        "ALTER TABLE {table} ADD COLUMN {column} {column_sql};"
    ))
    .map_err(status_from_error)
}
