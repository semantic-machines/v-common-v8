# Getting Started

## Installation

### Prerequisites

- Rust 1.70 or later
- libglib2.0-dev

**Install system dependencies:**
```bash
sudo apt install libglib2.0-dev
```

### Add Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
v-common-v8 = "0.1.132"
```

Or use the local path:

```toml
[dependencies]
v-common-v8 = { path = "../v-common-v8" }
```

## Basic Usage

### 1. Initialize V8

```rust
use v_common_v8::jsruntime::{v8_init, JsRuntime};

fn main() {
    // Initialize V8 platform (call once)
    v8_init();
    
    // Create runtime
    let mut runtime = JsRuntime::new();
}
```

### 2. Execute JavaScript

```rust
use v_common_v8::callback::init_context_with_callback;

let isolate = runtime.v8_isolate();
let mut scope = v8::HandleScope::new(isolate);
let context = init_context_with_callback(&mut scope);
let scope = &mut v8::ContextScope::new(&mut scope, context);

// Execute JavaScript
let code = r#"
    print("Hello from JavaScript!");
    
    var doc = get_individual("", "d:admin");
    if (doc) {
        print("Found:", doc["@"]);
    }
"#;

let source = v8::String::new(scope, code).unwrap();
let script = v8::Script::compile(scope, source, None).unwrap();
script.run(scope);
```

### 3. Load External Scripts

```rust
use v_common_v8::scripts_workplace::ScriptsWorkPlace;

let mut workplace = ScriptsWorkPlace::new(isolate);
workplace.load_ext_scripts("system_ticket");
```

## Simple Example

```rust
use v_common_v8::jsruntime::{v8_init, JsRuntime};
use v_common_v8::callback::init_context_with_callback;
use v_common_v8::v8;

fn main() {
    // Step 1: Initialize V8
    v8_init();
    
    // Step 2: Create runtime
    let mut runtime = JsRuntime::new();
    
    // Step 3: Get isolate and create scope
    let isolate = runtime.v8_isolate();
    let mut scope = v8::HandleScope::new(isolate);
    
    // Step 4: Initialize context with callbacks
    let context = init_context_with_callback(&mut scope);
    let scope = &mut v8::ContextScope::new(&mut scope, context);
    
    // Step 5: Execute JavaScript code
    let code = r#"
        // Use callback functions
        print("Starting script...");
        
        // Query for documents
        var result = query("", "'rdf:type'==='v-s:Document'", "", "", 10, 10, 0);
        print("Found documents:", result.count);
        
        // Process results
        for (var i = 0; i < result.result.length; i++) {
            var doc = get_individual("", result.result[i]);
            if (doc) {
                print("Document:", doc["@"]);
            }
        }
    "#;
    
    let source = v8::String::new(scope, code).unwrap();
    if let Some(script) = v8::Script::compile(scope, source, None) {
        script.run(scope);
    }
}
```

## Working with Individuals

### Create Individual

```rust
// In JavaScript
let code = r#"
    var newPerson = {
        "@": "d:new_person_" + Date.now(),
        "rdf:type": [
            {data: "v-s:Person", type: "Uri"}
        ],
        "v-s:firstName": [
            {data: "John", type: "String"}
        ],
        "v-s:lastName": [
            {data: "Doe", type: "String"}
        ],
        "v-s:birthday": [
            {data: "1990-01-15T00:00:00Z", type: "Datetime"}
        ]
    };
    
    var rc = put_individual("", newPerson);
    if (rc === 0) {
        print("Person created successfully");
    } else {
        print("Error:", rc);
    }
"#;
```

### Update Individual

```rust
let code = r#"
    // Load existing individual
    var person = get_individual("", "d:person_123");
    
    if (person) {
        // Add new property
        add_to_individual("", {
            "@": person["@"],
            "v-s:email": [
                {data: "john@example.com", type: "String"}
            ]
        });
        
        // Replace property
        set_in_individual("", {
            "@": person["@"],
            "v-s:lastName": [
                {data: "Smith", type: "String"}
            ]
        });
    }
"#;
```

### Query Individuals

```rust
let code = r#"
    // Search for persons
    var result = query(
        "",                              // ticket (empty for system)
        "'rdf:type'==='v-s:Person'",    // query
        "'v-s:lastName' asc",           // sort
        "",                              // databases
        100,                             // top
        100,                             // limit
        0                                // from
    );
    
    print("Total found:", result.count);
    
    // Process results
    var persons = get_individuals("", result.result);
    for (var i = 0; i < persons.length; i++) {
        if (persons[i]) {
            var name = persons[i]["v-s:firstName"][0].data + " " +
                      persons[i]["v-s:lastName"][0].data;
            print("Person:", name);
        }
    }
"#;
```

## Error Handling

```rust
let code = r#"
    try {
        var doc = get_individual("", "d:nonexistent");
        if (!doc) {
            print("Document not found");
        } else {
            // Process document
            var rc = put_individual("", doc);
            if (rc !== 0) {
                print("Update failed with code:", rc);
            }
        }
    } catch (e) {
        print("Error occurred:", e);
        log_trace("Stack: " + e.stack);
    }
"#;
```

## Configuration

### Script Locations

Set custom script location:

```
Module property: scripts_location = /path/to/scripts
```

Default locations:
- `./public/js/common/`
- `./public/js/server/`
- `./public/modules/**/server/`
- `./public/modules/**/common/`

### Loading Order

Create `.seq` file to control load order:

**Example: `./public/js/server/.seq`**
```
jquery.js
lodash.js
$modules
utils.js
main.js
```

## Best Practices

### 1. Initialize Once

```rust
use std::sync::Once;

static INIT: Once = Once::new();

fn ensure_v8_initialized() {
    INIT.call_once(|| {
        v8_init();
    });
}
```

### 2. Check Return Codes

```javascript
var rc = put_individual("", individual);
if (rc !== 0) {
    log_trace("Error code: " + rc);
}
```

### 3. Validate Input

```javascript
function updatePerson(id, email) {
    var person = get_individual("", id);
    if (!person) {
        print("Person not found:", id);
        return;
    }
    
    if (person["rdf:type"][0].data !== "v-s:Person") {
        print("Not a person:", id);
        return;
    }
    
    // Safe to update
    add_to_individual("", {
        "@": id,
        "v-s:email": [{data: email, type: "String"}]
    });
}
```

### 4. Use Transactions

All operations are automatically buffered in a transaction:

```javascript
// These are buffered
put_individual("", doc1);
put_individual("", doc2);
put_individual("", doc3);

// All committed together at script end
```

### 5. Handle Authorization

```javascript
var rights = get_rights("", "d:document_123", "d:user_456");

if (rights["v-s:canUpdate"] && rights["v-s:canUpdate"][0].data) {
    // User can update
    put_individual("", updatedDoc);
} else {
    print("Access denied");
}
```

## Next Steps

- Read [Callback Functions](callbacks.md) for complete API
- Study [Transaction System](transactions.md) for data operations
- Check [Script Execution](scripts.md) for advanced usage
- Review [Architecture](architecture.md) for system design

## Troubleshooting

### V8 Initialization Error

**Problem:** "V8 initialization failed"

**Solution:** Ensure V8 is initialized before any operations:
```rust
v8_init();  // Must be first
```

### Compilation Error

**Problem:** "Script compilation failed"

**Solution:** Check JavaScript syntax, common issues:
- Missing semicolons in certain contexts
- Undefined variables
- Incorrect callback function names

### Authorization Error

**Problem:** "Access denied" or empty results

**Solution:** Check ticket validity:
```javascript
var ticket = get_env_str_var("$ticket");
print("Using ticket:", ticket);
```

### Performance Issues

**Problem:** Slow script execution

**Solutions:**
- Batch operations where possible
- Use `get_individuals()` instead of multiple `get_individual()` calls
- Add filters to queries to reduce results
- Consider script execution order

