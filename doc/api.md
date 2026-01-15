# API Reference

## Rust Public API

### Module: jsruntime

#### v8_init()

Initialize V8 platform.

```rust
pub fn v8_init()
```

**Usage:**
```rust
use v_common_v8::jsruntime::v8_init;
v8_init();  // Call once at application startup
```

**Notes:**
- Must be called before any V8 operations
- Safe to call multiple times (uses `Once`)
- Initializes V8 platform and flags

---

#### JsRuntime

V8 runtime manager.

```rust
pub struct JsRuntime {
    v8_isolate: Option<v8::OwnedIsolate>,
}
```

**Methods:**

##### new()

Create new runtime instance.

```rust
pub fn new() -> Self
```

**Example:**
```rust
let mut runtime = JsRuntime::new();
```

##### v8_isolate()

Get mutable reference to V8 isolate.

```rust
pub fn v8_isolate(&mut self) -> &mut v8::OwnedIsolate
```

##### global_context()

Get global context handle.

```rust
pub fn global_context(&mut self) -> v8::Global<v8::Context>
```

---

### Module: callback

#### init_context_with_callback()

Initialize V8 context with callback functions.

```rust
pub fn init_context_with_callback<'a>(
    scope: &mut HandleScope<'a, ()>
) -> Local<'a, Context>
```

**Returns:** V8 context with all callbacks registered

**Registered Callbacks:**
- `print`
- `get_individual`
- `get_individuals`
- `put_individual`
- `remove_individual`
- `add_to_individual`
- `set_in_individual`
- `remove_from_individual`
- `query`
- `get_rights`
- `get_env_str_var`
- `get_env_num_var`
- `log_trace`

**Example:**
```rust
let mut scope = v8::HandleScope::new(isolate);
let context = init_context_with_callback(&mut scope);
```

---

### Module: common

#### str_2_v8()

Convert Rust string to V8 string.

```rust
pub fn str_2_v8<'sc>(
    scope: &mut HandleScope<'sc, ()>,
    s: &str
) -> v8::Local<'sc, v8::String>
```

---

#### v8_2_str()

Convert V8 value to Rust string.

```rust
pub fn v8_2_str<'sc>(
    scope: &mut HandleScope<'sc>,
    s: &v8::Local<'sc, v8::Value>
) -> String
```

---

#### individual2v8obj()

Convert Individual to V8 object.

```rust
pub fn individual2v8obj<'a>(
    scope: &mut HandleScope<'a>,
    src: &mut Individual
) -> v8::Local<'a, v8::Object>
```

**Example:**
```rust
let mut indv = Individual::default();
indv.set_id("d:test");
let v8_obj = individual2v8obj(&mut scope, &mut indv);
```

**Object Structure:**
```javascript
{
    "@": "d:test",
    "rdf:type": [{data: "...", type: "Uri"}],
    // ... other properties
}
```

---

#### v8obj2individual()

Convert V8 object to Individual.

```rust
pub fn v8obj2individual<'a>(
    scope: &mut HandleScope<'a>,
    v8_obj: v8::Local<'a, v8::Object>
) -> Individual
```

---

#### query_result2v8obj()

Convert query result to V8 object.

```rust
pub fn query_result2v8obj<'a>(
    scope: &mut HandleScope<'a>,
    src: &QueryResult
) -> v8::Local<'a, v8::Object>
```

**Result Object:**
```javascript
{
    count: 42,
    result: ["d:id1", "d:id2", ...],
    estimated: 42,
    processed: 42,
    total_time: 150,
    query_time: 100,
    authorize_time: 50,
    result_code: 0
}
```

---

#### collect_js_files()

Collect JavaScript files from path.

```rust
pub fn collect_js_files(in_path: &str, res: &mut Vec<String>)
```

**Example:**
```rust
let mut files = vec![];
collect_js_files("./scripts", &mut files);
```

---

#### collect_module_dirs()

Collect module directories.

```rust
pub fn collect_module_dirs(in_path: &str, res: &mut Vec<String>)
```

**Finds:**
- Directories containing `/server/`
- Directories containing `/common/`

---

#### is_filter_pass()

Check if script should execute for individual.

```rust
pub fn is_filter_pass(
    script: &ScriptInfo<ScriptInfoContext>,
    individual_id: &str,
    indv_types: &[String],
    onto: &mut Onto
) -> bool
```

---

### Module: scripts_workplace

#### ScriptsWorkPlace

Script execution environment.

```rust
pub struct ScriptsWorkPlace<'a, T> {
    pub scripts: HashMap<String, ScriptInfo<'a, T>>,
    pub scripts_order: Vec<String>,
    pub backend: Backend,
    pub scope: HandleScope<'a, ()>,
    pub context: Local<'a, Context>,
}
```

**Methods:**

##### new()

Create new scripts workplace.

```rust
pub fn new(isolate: &'a mut Isolate) -> Self
```

##### load_ext_scripts()

Load external scripts.

```rust
pub fn load_ext_scripts(&mut self, sys_ticket: &str)
```

**Example:**
```rust
let mut workplace = ScriptsWorkPlace::new(&mut isolate);
workplace.load_ext_scripts("system_ticket");
```

##### add_to_order()

Add script to execution order.

```rust
pub fn add_to_order(&mut self, scr_inf: &ScriptInfo<T>)
```

---

#### ScriptInfo

Script information and state.

```rust
pub struct ScriptInfo<'a, T> {
    pub id: String,
    pub str_script: String,
    pub compiled_script: Option<v8::Local<'a, v8::Script>>,
    pub dependency: HashVec<String>,
    pub context: T,
}
```

**Methods:**

##### new_with_src()

Create new script info.

```rust
pub fn new_with_src(id: &str, src: &str) -> Self
```

##### compile_script()

Compile JavaScript source.

```rust
pub fn compile_script(&mut self, js_name: &str, scope: &mut HandleScope<'a>)
```

---

### Module: session_cache

#### Transaction

Transaction manager.

```rust
pub struct Transaction {
    pub sys_ticket: String,
    pub id: i64,
    pub event_id: String,
    pub queue: Vec<TransactionItem>,
    pub src: String,
}
```

**Methods:**

##### add_to_transaction()

Add operation to transaction.

```rust
pub fn add_to_transaction(
    &mut self,
    cmd: IndvOp,
    new_indv: Individual,
    ticket_id: String,
    user_id: String
) -> ResultCode
```

##### get_indv()

Get individual from transaction buffer.

```rust
pub fn get_indv(&mut self, id: &str) -> Option<&mut Individual>
```

---

#### commit()

Commit transaction to storage.

```rust
pub fn commit(
    tnx: &Transaction,
    api_client: &mut MStorageClient
) -> ResultCode
```

**Example:**
```rust
let rc = commit(&tnx, &mut client);
if rc != ResultCode::Ok {
    error!("Commit failed: {:?}", rc);
}
```

---

#### CallbackSharedData

Shared data between callbacks.

```rust
pub struct CallbackSharedData {
    pub g_key2indv: HashMap<String, Individual>,
    pub g_key2attr: HashMap<String, String>,
}
```

**Methods:**

##### set_g_parent_script_id_etc()

Set parent script information.

```rust
pub fn set_g_parent_script_id_etc(&mut self, event_id: &str)
```

##### set_g_super_classes()

Set super classes array.

```rust
pub fn set_g_super_classes(&mut self, indv_types: &[String], onto: &Onto)
```

---

## Global State

### G_VARS

Global variables shared across scripts.

```rust
pub static ref G_VARS: Mutex<RefCell<CallbackSharedData>>
```

**Access:**
```rust
let mut sh_g_vars = G_VARS.lock().unwrap();
let g_vars = sh_g_vars.get_mut();
// ... use g_vars ...
drop(sh_g_vars);
```

---

### G_TRANSACTION

Global transaction state.

```rust
pub static ref G_TRANSACTION: Mutex<RefCell<Transaction>>
```

---

## Data Types

### HashVec

Hybrid hash-set and vector.

```rust
pub struct HashVec<String> {
    pub hash: HashSet<String>,
    pub vec: Vec<String>,
}
```

**Methods:**

##### new()

Create from vector.

```rust
pub fn new(src: Vec<String>) -> Self
```

---

### ScriptInfoContext

Script execution context.

```rust
pub struct ScriptInfoContext {
    pub trigger_by_type: HashVec<String>,
    pub prevent_by_type: HashVec<String>,
    pub trigger_by_uid: HashVec<String>,
    pub run_at: String,
    pub execute_if: String,
    pub disallow_changing_source: bool,
    pub is_unsafe: bool,
}
```

---

## Re-exports

The library re-exports:

```rust
pub use v8;
pub use v_common;
```

These can be used as:
```rust
use v_common_v8::v8;
use v_common_v8::v_common;
```

