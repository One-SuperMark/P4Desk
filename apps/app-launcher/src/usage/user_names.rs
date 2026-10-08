//! Bounded, optional user-profile enrichment for the single monitor worker.
//! Retain only the displayed name and a configuration digest, never emails,
//! complete notes, authentication keys, or management API response bodies.
use super::api::{array, request, Error, Transport};
use super::{Config, Data};
use serde_json::Value;
use sha2::{Digest, Sha256};

const MAX_NAMES: usize = 200;
const PAGE_SIZE: usize = 25;
const MAX_PAGES: usize = MAX_NAMES / PAGE_SIZE;
const TTL_MS: u64 = 5 * 60 * 1_000;

/// Match the desktop client's remark/name priority without using an email as
/// a display-name substitute. Limit by Unicode characters, not UTF-8 bytes.
pub fn clean_name(value: &Value) -> Option<String> {
    clean_text(value.get("notes").and_then(Value::as_str))
        .or_else(|| clean_text(value.get("username").and_then(Value::as_str)))
}

fn clean_text(value: Option<&str>) -> Option<String> {
    let cleaned: String = value?
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(80)
        .collect();
    let cleaned = cleaned.trim();
    (!cleaned.is_empty()).then(|| cleaned.to_owned())
}

struct CachedName {
    id: i64,
    name: Option<String>,
    checked_ms: u64,
}

impl CachedName {
    fn fresh(&self, now_ms: u64) -> bool {
        now_ms >= self.checked_ms && now_ms - self.checked_ms < TTL_MS
    }
}

/// One cache per serial worker. Expired names remain available on a transient
/// request failure; successful profiles with no name explicitly clear them.
#[derive(Default)]
pub struct UserNameCache {
    identity: Option<[u8; 32]>,
    names: Vec<CachedName>,
}

impl UserNameCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lookup(&self, id: i64) -> Option<&str> {
        self.names
            .iter()
            .find(|entry| entry.id == id)
            .and_then(|entry| entry.name.as_deref())
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn clear(&mut self) {
        self.identity = None;
        self.names.clear();
    }

    fn store(&mut self, id: i64, name: Option<String>, checked_ms: u64) {
        if let Some(entry) = self.names.iter_mut().find(|entry| entry.id == id) {
            entry.name = name;
            entry.checked_ms = checked_ms;
        } else if self.names.len() < MAX_NAMES {
            self.names.push(CachedName {
                id,
                name,
                checked_ms,
            });
        }
    }

    /// Enrich only the current ranked IDs. The optional profile request never
    /// removes valid usage metrics. Partial successful pages remain applied if
    /// a later page fails, so callers may report a single bounded warning.
    pub fn enrich(
        &mut self,
        transport: &mut impl Transport,
        config: &Config,
        data: &mut Data,
        now_mono_ms: u64,
    ) -> Result<(), Error> {
        if data.users.len() > MAX_NAMES {
            return Err(Error::TooLarge);
        }
        let mut ranked_ids = Vec::with_capacity(data.users.len());
        for user in &data.users {
            if user.id <= 0 || ranked_ids.contains(&user.id) {
                return Err(Error::Format);
            }
            ranked_ids.push(user.id);
        }
        let mut digest = Sha256::new();
        digest.update(config.site.as_bytes());
        digest.update([0]);
        digest.update(config.key.as_bytes());
        let identity: [u8; 32] = digest.finalize().into();
        if self.identity != Some(identity) {
            self.names.clear();
            self.identity = Some(identity);
        }
        self.names.retain(|entry| ranked_ids.contains(&entry.id));

        let mut pending = Vec::with_capacity(data.users.len());
        for user in &mut data.users {
            if let Some(name) = clean_text(user.name.as_deref()) {
                // Fresh inline profiles need no second management request.
                user.name = Some(name.clone());
                self.store(user.id, Some(name), now_mono_ms);
                continue;
            }
            user.name = None;
            if let Some(entry) = self.names.iter().find(|entry| entry.id == user.id) {
                user.name = entry.name.clone();
                if entry.fresh(now_mono_ms) {
                    continue;
                }
            }
            pending.push(user.id);
        }
        if pending.is_empty() {
            return Ok(());
        }

        let mut seen = Vec::with_capacity(MAX_NAMES);
        let mut expected_total = None;
        for page in 1..=MAX_PAGES {
            let value = request(
                transport,
                config,
                &format!("/admin/users?page={page}&page_size={PAGE_SIZE}"),
                None,
            )?;
            let total = value
                .get("total")
                .and_then(Value::as_u64)
                .ok_or(Error::Format)?;
            if expected_total.is_some_and(|expected| expected != total) {
                return Err(Error::Format);
            }
            expected_total = Some(total);
            let rows = array(&value, "items", PAGE_SIZE)?;
            let expected_count = total
                .saturating_sub(seen.len() as u64)
                .min(PAGE_SIZE as u64) as usize;
            if rows.len() != expected_count {
                return Err(Error::Format);
            }

            // Validate the complete page before clearing any cached name.
            let mut parsed = Vec::with_capacity(rows.len());
            for row in rows {
                let id = row
                    .get("id")
                    .and_then(Value::as_i64)
                    .filter(|id| *id > 0)
                    .ok_or(Error::Format)?;
                if seen.contains(&id)
                    || parsed.iter().any(|(previous_id, _)| *previous_id == id)
                    || ["notes", "username"].iter().any(|field| {
                        row.get(*field)
                            .is_some_and(|value| !value.is_null() && !value.is_string())
                    })
                {
                    return Err(Error::Format);
                }
                parsed.push((id, clean_name(row)));
            }
            for (id, name) in parsed {
                seen.push(id);
                if pending.contains(&id) {
                    self.store(id, name.clone(), now_mono_ms);
                    if let Some(user) = data.users.iter_mut().find(|user| user.id == id) {
                        user.name = name;
                    }
                    pending.retain(|pending_id| *pending_id != id);
                }
            }
            if pending.is_empty() {
                return Ok(());
            }
            if seen.len() as u64 == total {
                // A complete management list also proves absent IDs have no
                // current profile. Cache that result to avoid repeated scans.
                for id in pending {
                    self.store(id, None, now_mono_ms);
                    if let Some(user) = data.users.iter_mut().find(|user| user.id == id) {
                        user.name = None;
                    }
                }
                return Ok(());
            }
        }
        Err(Error::TooLarge)
    }
}
