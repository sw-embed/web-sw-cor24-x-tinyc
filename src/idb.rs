//! Minimal IndexedDB wrapper for storing binary blobs that exceed
//! localStorage limits (e.g. SD card .img uploads).
//!
//! Single object store `blobs` keyed by string, values stored as
//! Uint8Array. Opens the DB on demand; the schema is created on first
//! open via the `upgradeneeded` event.

use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    IdbDatabase, IdbObjectStore, IdbOpenDbRequest, IdbRequest, IdbTransactionMode,
};

const DB_NAME: &str = "web-sw-cor24-x-tinyc";
const DB_VERSION: u32 = 1;
const STORE_NAME: &str = "blobs";

async fn open_db() -> Result<IdbDatabase, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let factory = window
        .indexed_db()?
        .ok_or_else(|| JsValue::from_str("no IndexedDB"))?;
    let open_req: IdbOpenDbRequest = factory.open_with_u32(DB_NAME, DB_VERSION)?;

    // Wire the upgrade handler before awaiting the open promise.
    // `onupgradeneeded` only fires on first open (or version bump), so
    // create_object_store unconditionally is safe.
    let upgrade = Closure::<dyn FnMut(web_sys::IdbVersionChangeEvent)>::new(
        |e: web_sys::IdbVersionChangeEvent| {
            let req: IdbOpenDbRequest = e.target().unwrap().dyn_into().unwrap();
            let db: IdbDatabase = req.result().unwrap().dyn_into().unwrap();
            let _ = db.create_object_store(STORE_NAME);
        },
    );
    open_req.set_onupgradeneeded(Some(upgrade.as_ref().unchecked_ref()));
    let result = request_to_future(&open_req).await?;
    drop(upgrade);
    result.dyn_into::<IdbDatabase>()
}

/// Convert an `IdbRequest` to a future that resolves to its `.result` on
/// success or rejects with the request's error on failure.
fn request_to_future(req: &IdbRequest) -> JsFuture {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let req_clone = req.clone();
        let resolve_clone = resolve.clone();
        let onsuccess = Closure::once_into_js(move || {
            let result = req_clone.result().unwrap_or(JsValue::UNDEFINED);
            let _ = resolve_clone.call1(&JsValue::NULL, &result);
        });
        req.set_onsuccess(Some(onsuccess.unchecked_ref()));

        let req_clone = req.clone();
        let onerror = Closure::once_into_js(move || {
            let err = req_clone.error().ok().flatten().map_or(
                JsValue::from_str("idb request failed"),
                JsValue::from,
            );
            let _ = reject.call1(&JsValue::NULL, &err);
        });
        req.set_onerror(Some(onerror.unchecked_ref()));
    });
    JsFuture::from(promise)
}

fn store(db: &IdbDatabase, mode: IdbTransactionMode) -> Result<IdbObjectStore, JsValue> {
    db.transaction_with_str_and_mode(STORE_NAME, mode)?
        .object_store(STORE_NAME)
}

/// Read a binary blob by key. Returns `None` if absent.
pub async fn get(key: &str) -> Option<Vec<u8>> {
    let db = open_db().await.ok()?;
    let store = store(&db, IdbTransactionMode::Readonly).ok()?;
    let req = store.get(&JsValue::from_str(key)).ok()?;
    let result = request_to_future(&req).await.ok()?;
    if result.is_undefined() || result.is_null() {
        return None;
    }
    let arr: Uint8Array = result.dyn_into().ok()?;
    Some(arr.to_vec())
}

/// Write a binary blob under `key`. Overwrites any existing value.
pub async fn put(key: &str, bytes: &[u8]) -> Result<(), JsValue> {
    let db = open_db().await?;
    let store = store(&db, IdbTransactionMode::Readwrite)?;
    let arr = Uint8Array::new_with_length(bytes.len() as u32);
    arr.copy_from(bytes);
    let req = store.put_with_key(&arr, &JsValue::from_str(key))?;
    request_to_future(&req).await?;
    Ok(())
}

/// Remove the value at `key`. No-op if absent.
pub async fn remove(key: &str) -> Result<(), JsValue> {
    let db = open_db().await?;
    let store = store(&db, IdbTransactionMode::Readwrite)?;
    let req = store.delete(&JsValue::from_str(key))?;
    request_to_future(&req).await?;
    Ok(())
}
