# Script Execution

## Overview

The script execution system loads and runs JavaScript files within the V8 environment, providing integration with the Veda platform.

## Script Discovery

### Location Priority

Scripts are loaded from locations in this order:

1. **Custom Location** (if configured)
   ```
   Module property: "scripts_location"
   ```

2. **Default Locations**
   ```
   ./public/js/common/
   ./public/js/server/
   ```

3. **Module Directories**
   ```
   ./public/modules/**/server/
   ./public/modules/**/common/
   ```

### Loading Sequence

Scripts can be loaded in specific order using `.seq` files:

**Example .seq file:**
```
core.js
utils.js
$modules
app.js
```

- Regular filenames load specific files
- `$modules` placeholder loads all module scripts
- One filename per line
- Order matters for dependencies

## Script Information

### ScriptInfo Structure

```rust
pub struct ScriptInfo<'a, T> {
    pub id: String,                              // Script identifier
    pub str_script: String,                      // Source code
    pub compiled_script: Option<v8::Script>,     // Compiled script
    pub dependency: HashVec<String>,             // Dependencies
    pub context: T,                              // Context (filter rules)
}
```

### Script Context

For trigger scripts:

```rust
pub struct ScriptInfoContext {
    pub trigger_by_type: HashVec<String>,       // Trigger on types
    pub prevent_by_type: HashVec<String>,       // Skip on types
    pub trigger_by_uid: HashVec<String>,        // Trigger on specific IDs
    pub run_at: String,                         // Execution phase
    pub execute_if: String,                     // Condition expression
    pub disallow_changing_source: bool,         // Source change flag
    pub is_unsafe: bool,                        // Safety flag
}
```

## Scripts Workplace

### Initialization

```rust
let workplace = ScriptsWorkPlace::new(&mut isolate);
workplace.load_ext_scripts(sys_ticket);
```

### Loading Process

1. **Collect Script Paths**
   - Read .seq files or scan directories
   - Collect all .js files
   - Sort by dependencies

2. **Initialize Context**
   - Create V8 context with callbacks
   - Set up global variables
   - Register system functions

3. **Compile and Execute**
   - Load file contents
   - Compile JavaScript
   - Execute in context
   - Handle errors

## Script Compilation

### Compilation Process

```rust
pub fn compile_script(&mut self, js_name: &str, scope: &mut HandleScope<'a>) {
    let source = str_2_v8(scope, &self.str_script);
    let name = v8::String::new(scope, js_name).unwrap();
    let origin = script_origin(scope, name);
    
    let mut tc_scope = v8::TryCatch::new(scope);
    match v8::Script::compile(&mut tc_scope, source, Some(&origin)) {
        Some(script) => self.compiled_script = Some(script),
        None => {
            // Log compilation error
            let exc = tc_scope.exception().unwrap();
            error!("Compilation failed: {}", exc);
        }
    }
}
```

### Error Handling

Compilation errors include:
- Syntax errors
- Reference errors
- Type errors

Errors are logged but don't stop other scripts from loading.

## Script Execution

### Execution Context

Each script executes with access to:
- All registered callback functions
- Global context variables
- Previous script results (shared scope)

### Execution Example

```javascript
// Script has access to callbacks
var doc = get_individual("", "d:example");

if (doc) {
    // Modify document
    doc["v-s:lastModified"] = [{
        data: new Date().toISOString(),
        type: "Datetime"
    }];
    
    // Save changes
    put_individual("", doc);
}
```

## Filter System

### Trigger Filters

Control when scripts execute:

**By Type:**
```rust
script.context.trigger_by_type = vec!["v-s:Person", "v-s:Organization"];
```

**By UID:**
```rust
script.context.trigger_by_uid = vec!["d:specific_doc"];
```

**Prevent:**
```rust
script.context.prevent_by_type = vec!["v-s:File"];
```

### Filter Logic

```rust
pub fn is_filter_pass(
    script: &ScriptInfo<ScriptInfoContext>,
    individual_id: &str,
    indv_types: &[String],
    onto: &mut Onto
) -> bool
```

**Rules:**
1. If any `prevent_by_type` matches → skip
2. If no triggers defined → execute
3. If `trigger_by_uid` matches → execute
4. If `trigger_by_type` matches → execute
5. Check ontology hierarchy for matches

## Dependency Management

### Dependency Tracking

```rust
pub struct HashVec<String> {
    pub hash: HashSet<String>,  // Fast lookup
    pub vec: Vec<String>,       // Ordered list
}
```

### Execution Order

Scripts execute in dependency order:

```
dependency1.js → dependency2.js → main.js
```

The system automatically:
- Detects dependencies
- Sorts scripts
- Ensures correct load order

## Session Data

### Shared Variables

Scripts share data via global variables:

```rust
pub struct CallbackSharedData {
    pub g_key2indv: HashMap<String, Individual>,  // Shared individuals
    pub g_key2attr: HashMap<String, String>,      // Shared attributes
}
```

**Access from JavaScript:**
```javascript
var ticket = get_env_str_var("$ticket");
var parentDoc = get_env_str_var("$parent_document_id");
```

### Setting Session Data

```rust
session_data.g_key2attr.insert("$ticket".to_owned(), sys_ticket.to_owned());
```

## File Operations

### Collecting JS Files

```rust
pub fn collect_js_files(in_path: &str, res: &mut Vec<String>)
```

- Recursively scans directories
- Filters .js extensions
- Returns sorted list

### Collecting Module Directories

```rust
pub fn collect_module_dirs(in_path: &str, res: &mut Vec<String>)
```

- Finds /server/ and /common/ directories
- Used for modular script loading

## Best Practices

1. **Structure Scripts**
   - Use .seq files for complex dependencies
   - Keep scripts focused and modular
   - Separate common and server logic

2. **Handle Errors**
   ```javascript
   try {
       var doc = get_individual("", id);
       // ... process ...
   } catch (e) {
       log_trace("Error: " + e);
   }
   ```

3. **Use Filters Wisely**
   - Set specific type triggers
   - Use prevent rules to skip unnecessary work
   - Consider ontology hierarchy

4. **Manage Dependencies**
   - Declare dependencies explicitly
   - Don't assume load order without .seq
   - Test with fresh context

5. **Session Variables**
   - Use $ prefix for session variables
   - Clean up after script execution
   - Document expected variables

