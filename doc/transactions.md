# Transaction System

## Overview

The transaction system buffers all data operations during script execution and commits them as a batch at the end. This ensures data consistency and allows operations to reference uncommitted changes.

## Transaction Structure

```rust
pub struct Transaction {
    pub sys_ticket: String,        // System ticket for operations
    pub id: i64,                   // Transaction ID
    pub event_id: String,          // Event identifier
    buff: HashMap<String, usize>,  // ID to queue index mapping
    pub queue: Vec<TransactionItem>, // Queued operations
    pub src: String,               // Source identifier
}
```

## Transaction Flow

```
JavaScript Operation
        ↓
  add_to_transaction()
        ↓
    Buffer in Queue
        ↓
    Apply Commands
        ↓
   commit() to Storage
```

## Operation Types

### IndvOp Enum

- **Put** - Create or completely replace individual
- **Remove** - Delete individual
- **AddTo** - Add properties to existing individual
- **SetIn** - Set (replace) specific properties
- **RemoveFrom** - Remove specific properties

## Operation Processing

### 1. Put Operation

Directly adds the individual to transaction queue.

```javascript
put_individual("", {
    "@": "d:new_doc",
    "rdf:type": [{data: "v-s:Document", type: "Uri"}]
});
```

### 2. Remove Operation

Adds remove operation to queue.

```javascript
remove_individual("", "d:doc_to_delete");
```

### 3. AddTo/SetIn/RemoveFrom Operations

These operations:
1. Load current individual (from buffer or storage)
2. Apply the operation to current state
3. Convert to Put operation
4. Add to queue

**Example - AddTo:**
```javascript
// Adds email without removing existing properties
add_to_individual("", {
    "@": "d:person_123",
    "v-s:email": [{data: "new@email.com", type: "String"}]
});
```

**Example - SetIn:**
```javascript
// Replaces email property completely
set_in_individual("", {
    "@": "d:person_123",
    "v-s:email": [{data: "updated@email.com", type: "String"}]
});
```

**Example - RemoveFrom:**
```javascript
// Removes specific email value
remove_from_individual("", {
    "@": "d:person_123",
    "v-s:email": [{data: "old@email.com", type: "String"}]
});
```

## Buffering Behavior

The transaction maintains a buffer (`HashMap<String, usize>`) that:
- Maps individual IDs to their position in the queue
- Allows subsequent operations to reference uncommitted changes
- Enables reading your own writes within the transaction

**Example:**
```javascript
// Create new individual
put_individual("", {
    "@": "d:new_person",
    "v-s:firstName": [{data: "John", type: "String"}]
});

// Read it immediately (from transaction buffer)
var person = get_individual("", "d:new_person");

// Update it (applies to buffered version)
add_to_individual("", {
    "@": "d:new_person",
    "v-s:lastName": [{data: "Doe", type: "String"}]
});
```

## Result Codes

Operations return `ResultCode`:
- **Ok** (0) - Success
- **NotFound** - Individual not found
- **UnprocessableEntity** - Failed to parse data
- Other error codes for various failure conditions

## Commit Process

When `commit()` is called:

1. Iterate through all queued operations
2. Skip invalid operations (empty IDs, error states)
3. For each valid operation:
   - Create update options with event_id and src
   - Call storage API client
   - Check result code
   - Abort on first error

**Commit Options:**
```rust
UpdateOptions {
    event_id: Some(&tnx.event_id),
    src: Some(&tnx.src),
    assigned_subsystems: Some(ALL_MODULES),
    addr: None,
}
```

## Error Handling

### During add_to_transaction:
- Returns error code immediately
- Operation not added to queue
- Transaction continues

### During commit:
- First error stops commit process
- Returns error code
- Already committed operations remain
- No automatic rollback

## Best Practices

1. **Check Result Codes**
   ```javascript
   var rc = put_individual("", individual);
   if (rc !== 0) {
       print("Error:", rc);
   }
   ```

2. **Use Correct Operation Type**
   - Use `Put` for complete replacement
   - Use `AddTo` for appending properties
   - Use `SetIn` for replacing specific properties

3. **Validate Before Operations**
   ```javascript
   var existing = get_individual("", id);
   if (existing) {
       // Safe to update
   }
   ```

4. **Handle Event IDs**
   - Set meaningful event_id for tracking
   - Use format: `{doc_id}+{script_id};{timestamp}`

## Thread Safety

- Global transaction accessed via `Mutex<RefCell<Transaction>>`
- Lock acquired for each operation
- Released immediately after
- No long-held locks

## Transaction Lifecycle

```rust
// 1. Initialize
let mut tnx = Transaction::default();
tnx.sys_ticket = ticket;
tnx.event_id = event_id;
tnx.src = source;

// 2. Operations executed (via callbacks)
// ... JavaScript calls add operations ...

// 3. Commit
let rc = commit(&tnx, &mut api_client);

// 4. Check result
if rc != ResultCode::Ok {
    error!("Commit failed: {:?}", rc);
}
```

