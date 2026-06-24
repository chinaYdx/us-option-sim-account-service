use std::sync::atomic::{AtomicU64, Ordering};

use x_com_lib::{status_err, time_utils, x_core, x_core::xrpc::Context};

use crate::{
    config::SimAccountConfig,
    store::{status_value, AccountPatch, SimAccountStore},
    x_com::source_api::*,
};

static ACCOUNT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct UsOptionSimAccountService {
    config: SimAccountConfig,
    store: SimAccountStore,
}

impl UsOptionSimAccountService {
    pub fn new() -> UsOptionSimAccountService {
        let config = SimAccountConfig {
            data_path: String::default(),
        };
        let store = SimAccountStore::unopened();
        UsOptionSimAccountService { config, store }
    }

    pub async fn on_init(&mut self) -> x_core::Result<()> {
        self.config = SimAccountConfig::load();
        self.store = SimAccountStore::open(self.config.db_path())?;
        Ok(())
    }

    pub async fn on_finalize(&self) {}

    #[cfg(test)]
    async fn init_for_test(&mut self, data_path: &str) -> x_core::Result<()> {
        self.config = SimAccountConfig {
            data_path: data_path.to_owned(),
        };
        self.store = SimAccountStore::open(self.config.db_path())?;
        Ok(())
    }

    // CreateSimAccount
    pub async fn create_sim_account(
        &self,
        _ctx: Context,
        param: Box<CreateSimAccountRequest>,
    ) -> x_core::Result<Box<SimAccountInfo>> {
        let now = time_utils::cur_timestamp();
        let account_id = normalize_optional_account_id(&param.account_id, now)?;
        let currency = normalize_currency(&param.currency);
        if param.initial_cash <= 0.0 || !param.initial_cash.is_finite() {
            return status_err!("initial_cash must be positive finite number");
        }
        if self.store.get_account(&account_id)?.is_some() {
            return status_err!("sim account already exists: {}", account_id);
        }

        let mut account = Box::new(SimAccountInfo::default());
        account.account_id = account_id.clone();
        account.display_name = if param.display_name.trim().is_empty() {
            account_id
        } else {
            param.display_name.trim().to_owned()
        };
        account.initial_cash = param.initial_cash;
        account.currency = currency;
        account.status = Some(SimAccountStatus::Active);
        account.strategy_task_id = param.strategy_task_id.trim().to_owned();
        account.run_id = param.run_id.trim().to_owned();
        account.created_by = default_string(param.created_by.trim(), "system");
        account.created_at = now;
        account.updated_at = now;
        account.risk_limits = param.risk_limits;

        self.store.create_account(
            &account,
            SimAccountAuditAction::Created,
            &account.created_by,
            "created",
        )?;
        Ok(account)
    }

    // GetSimAccount
    pub async fn get_sim_account(
        &self,
        _ctx: Context,
        param: Box<SimAccountIdRequest>,
    ) -> x_core::Result<Box<SimAccountInfo>> {
        let account_id = require_account_id(&param.account_id)?;
        let Some(account) = self.store.get_account(account_id)? else {
            return status_err!("sim account not found: {}", account_id);
        };
        Ok(account)
    }

    // ListSimAccounts
    pub async fn list_sim_accounts(
        &self,
        _ctx: Context,
        param: Box<ListSimAccountsRequest>,
    ) -> x_core::Result<Box<SimAccountList>> {
        let status = match param.status {
            Some(SimAccountStatus::Unknown) | None => None,
            Some(status) => Some(status as i32),
        };
        let accounts = self.store.list_accounts(
            status,
            param.strategy_task_id.trim(),
            param.run_id.trim(),
            param.query.trim(),
            param.include_archived,
            param.limit,
        )?;
        let mut resp = Box::new(SimAccountList::default());
        resp.accounts = accounts;
        Ok(resp)
    }

    // UpdateSimAccount
    pub async fn update_sim_account(
        &self,
        _ctx: Context,
        param: Box<UpdateSimAccountRequest>,
    ) -> x_core::Result<Box<SimAccountInfo>> {
        let account_id = require_account_id(&param.account_id)?;
        let status = if param.update_status {
            let status = status_value(param.status);
            if status == SimAccountStatus::Unknown as i32 {
                return status_err!("status update cannot use Unknown status");
            }
            Some(status)
        } else {
            None
        };
        let patch = AccountPatch {
            display_name: if param.update_display_name {
                Some(param.display_name.trim().to_owned())
            } else {
                None
            },
            strategy_task_id: if param.update_binding {
                Some(param.strategy_task_id.trim().to_owned())
            } else {
                None
            },
            run_id: if param.update_binding {
                Some(param.run_id.trim().to_owned())
            } else {
                None
            },
            status,
            risk_limits: if param.update_risk_limits {
                Some(param.risk_limits)
            } else {
                None
            },
            actor: default_string(param.actor.trim(), "system"),
            reason: param.reason.trim().to_owned(),
        };
        let Some(account) = self.store.update_account(account_id, patch)? else {
            return status_err!("sim account not found: {}", account_id);
        };
        Ok(account)
    }

    // ListSimAccountAuditEvents
    pub async fn list_sim_account_audit_events(
        &self,
        _ctx: Context,
        param: Box<ListSimAccountAuditEventsRequest>,
    ) -> x_core::Result<Box<SimAccountAuditEventList>> {
        let events = self.store.list_audit_events(
            param.account_id.trim(),
            param.start_time,
            param.end_time,
            param.limit,
        )?;
        let mut resp = Box::new(SimAccountAuditEventList::default());
        resp.events = events;
        Ok(resp)
    }

    // GetSimAccountServiceHealth
    pub async fn get_sim_account_service_health(
        &self,
        _ctx: Context,
    ) -> x_core::Result<Box<SimAccountServiceHealth>> {
        let (account_count, audit_event_count) = self.store.counts()?;
        let mut resp = Box::new(SimAccountServiceHealth::default());
        resp.sqlite_path = self.store.path().display().to_string();
        resp.account_count = account_count;
        resp.audit_event_count = audit_event_count;
        resp.updated_at = time_utils::cur_timestamp();
        Ok(resp)
    }
}

fn normalize_optional_account_id(value: &str, now: i64) -> x_core::Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        let seq = ACCOUNT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        return Ok(format!("us-option-sim-{now}-{seq}"));
    }
    validate_id("account_id", trimmed)?;
    Ok(trimmed.to_owned())
}

fn require_account_id(value: &str) -> x_core::Result<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return status_err!("account_id is required");
    }
    validate_id("account_id", trimmed)?;
    Ok(trimmed)
}

fn validate_id(label: &str, value: &str) -> x_core::Result<()> {
    if value == "." || value == ".." {
        return status_err!("{} must not be {}", label, value);
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return status_err!(
            "{} must contain only ASCII letters, digits, '_' or '-'",
            label
        );
    }
    Ok(())
}

fn normalize_currency(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "USD".to_owned()
    } else {
        trimmed.to_ascii_uppercase()
    }
}

fn default_string(value: &str, default_value: &str) -> String {
    if value.is_empty() {
        default_value.to_owned()
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use x_com_lib::{x_core::xrpc::Context, ServiceKey};

    use super::*;

    static TEMP_ROOT_COUNTER: AtomicU64 = AtomicU64::new(1);

    fn temp_root() -> String {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let seq = TEMP_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("us_option_sim_account_{stamp}_{seq}"));
        fs::create_dir_all(&root).expect("create temp dir");
        root.to_string_lossy().to_string()
    }

    fn test_ctx() -> Context {
        Context {
            sender_service_key: Box::new(ServiceKey::default()),
            channel_id: 0,
            conn_id: 0,
            request_id: 0,
            from_addr: None,
        }
    }

    fn create_request(account_id: &str) -> Box<CreateSimAccountRequest> {
        let mut req = Box::new(CreateSimAccountRequest::default());
        req.account_id = account_id.to_owned();
        req.display_name = "SPY live sim".to_owned();
        req.initial_cash = 100_000.0;
        req.currency = "usd".to_owned();
        req.strategy_task_id = "task-001".to_owned();
        req.run_id = "run-001".to_owned();
        req.created_by = "tester".to_owned();
        req.risk_limits.max_single_order_notional = Some(10_000.0);
        req.risk_limits.max_open_order_count = Some(5);
        req.risk_limits.allow_opening_trades = Some(true);
        req
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_get_list_update_audit_and_restart_smoke() {
        let root = temp_root();
        let mut service = UsOptionSimAccountService::new();
        service.init_for_test(&root).await.expect("init");

        let created = service
            .create_sim_account(test_ctx(), create_request("acct-001"))
            .await
            .expect("create");
        assert_eq!(created.account_id, "acct-001");
        assert_eq!(created.currency, "USD");
        assert_eq!(created.initial_cash, 100_000.0);
        assert_eq!(created.risk_limits.max_open_order_count, Some(5));

        let mut get = Box::new(SimAccountIdRequest::default());
        get.account_id = "acct-001".to_owned();
        let fetched = service.get_sim_account(test_ctx(), get).await.expect("get");
        assert_eq!(fetched.strategy_task_id, "task-001");

        let mut list_req = Box::new(ListSimAccountsRequest::default());
        list_req.strategy_task_id = "task-001".to_owned();
        let list = service
            .list_sim_accounts(test_ctx(), list_req)
            .await
            .expect("list");
        assert_eq!(list.accounts.len(), 1);

        let mut update = Box::new(UpdateSimAccountRequest::default());
        update.account_id = "acct-001".to_owned();
        update.update_binding = true;
        update.strategy_task_id = "task-002".to_owned();
        update.run_id = "run-002".to_owned();
        update.update_status = true;
        update.status = Some(SimAccountStatus::Paused);
        update.update_risk_limits = true;
        update.risk_limits.max_contract_quantity = Some(20);
        update.actor = "risk".to_owned();
        update.reason = "rebalance limits".to_owned();
        let updated = service
            .update_sim_account(test_ctx(), update)
            .await
            .expect("update");
        assert_eq!(updated.strategy_task_id, "task-002");
        assert_eq!(updated.run_id, "run-002");
        assert_eq!(
            updated.status,
            Some(SimAccountStatus::Paused)
        );
        assert_eq!(updated.risk_limits.max_contract_quantity, Some(20));

        let mut audit_req = Box::new(ListSimAccountAuditEventsRequest::default());
        audit_req.account_id = "acct-001".to_owned();
        let audit = service
            .list_sim_account_audit_events(test_ctx(), audit_req)
            .await
            .expect("audit");
        assert_eq!(audit.events.len(), 2);
        assert!(audit.events.iter().any(|event| event.action
            == Some(SimAccountAuditAction::RiskLimitsChanged)));

        let mut restarted = UsOptionSimAccountService::new();
        restarted.init_for_test(&root).await.expect("restart");
        let mut get = Box::new(SimAccountIdRequest::default());
        get.account_id = "acct-001".to_owned();
        let loaded = restarted
            .get_sim_account(test_ctx(), get)
            .await
            .expect("get after restart");
        assert_eq!(loaded.strategy_task_id, "task-002");
        assert_eq!(loaded.risk_limits.max_contract_quantity, Some(20));

        let health = restarted
            .get_sim_account_service_health(test_ctx())
            .await
            .expect("health");
        assert_eq!(health.account_count, 1);
        assert_eq!(health.audit_event_count, 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_rejects_duplicate_and_bad_runtime_fields() {
        let root = temp_root();
        let mut service = UsOptionSimAccountService::new();
        service.init_for_test(&root).await.expect("init");
        service
            .create_sim_account(test_ctx(), create_request("acct-dup"))
            .await
            .expect("create");

        let dup = service
            .create_sim_account(test_ctx(), create_request("acct-dup"))
            .await
            .expect_err("duplicate must fail");
        assert!(format!("{dup:?}").contains("already exists"));

        let mut bad = create_request("bad acct");
        bad.initial_cash = 10.0;
        let err = service
            .create_sim_account(test_ctx(), bad)
            .await
            .expect_err("bad id must fail");
        assert!(format!("{err:?}").contains("account_id"));

        let mut bad_initial = create_request("acct-bad-initial");
        bad_initial.initial_cash = 0.0;
        let err = service
            .create_sim_account(test_ctx(), bad_initial)
            .await
            .expect_err("bad initial_cash must fail");
        assert!(format!("{err:?}").contains("initial_cash"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn rejects_missing_not_found_and_unknown_status_updates() {
        let root = temp_root();
        let mut service = UsOptionSimAccountService::new();
        service.init_for_test(&root).await.expect("init");

        let err = service
            .get_sim_account(test_ctx(), Box::new(SimAccountIdRequest::default()))
            .await
            .expect_err("empty account id must fail");
        assert!(format!("{err:?}").contains("account_id"));

        let mut missing_get = Box::new(SimAccountIdRequest::default());
        missing_get.account_id = "missing-account".to_owned();
        let err = service
            .get_sim_account(test_ctx(), missing_get)
            .await
            .expect_err("missing get must fail");
        assert!(format!("{err:?}").contains("not found"));

        let mut missing_update = Box::new(UpdateSimAccountRequest::default());
        missing_update.account_id = "missing-account".to_owned();
        missing_update.update_display_name = true;
        missing_update.display_name = "missing".to_owned();
        let err = service
            .update_sim_account(test_ctx(), missing_update)
            .await
            .expect_err("missing update must fail");
        assert!(format!("{err:?}").contains("not found"));

        service
            .create_sim_account(test_ctx(), create_request("acct-status"))
            .await
            .expect("create");
        let mut bad_status = Box::new(UpdateSimAccountRequest::default());
        bad_status.account_id = "acct-status".to_owned();
        bad_status.update_status = true;
        bad_status.status = Some(SimAccountStatus::Unknown);
        let err = service
            .update_sim_account(test_ctx(), bad_status)
            .await
            .expect_err("unknown status update must fail");
        assert!(format!("{err:?}").contains("Unknown status"));
    }
}
