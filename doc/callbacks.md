# JavaScript Callback Functions

This document describes the JavaScript callback functions exposed to scripts running in the V8 environment.

## Individual Operations

### get_individual(ticket, id)

Retrieve an individual by ID.

**Parameters:**
- `ticket` (string) - Authentication ticket (can be empty to use transaction ticket)
- `id` (string) - Individual ID

**Returns:** Individual object or undefined if not found

**Example:**
```javascript
var person = get_individual("", "d:person_123");
if (person) {
    print(person["@"]);  // prints ID
}
```

**Special IDs:**
- IDs starting with `$` - retrieved from shared variables
- Regular IDs - retrieved from transaction buffer or storage

---

### get_individuals(ticket, ids)

Retrieve multiple individuals at once.

**Parameters:**
- `ticket` (string) - Authentication ticket
- `ids` (array) - Array of individual IDs

**Returns:** Array of individual objects (nulls for not found)

**Example:**
```javascript
var persons = get_individuals("", ["d:person_1", "d:person_2"]);
```

---

### put_individual(ticket, individual)

Create or update an individual.

**Parameters:**
- `ticket` (string) - Authentication ticket (empty for transaction ticket)
- `individual` (object) - Individual object to save

**Returns:** Result code (integer)

**Example:**
```javascript
var newPerson = {
    "@": "d:new_person",
    "rdf:type": [{data: "v-s:Person", type: "Uri"}],
    "v-s:name": [{data: "John Doe", type: "String"}]
};
var rc = put_individual("", newPerson);
```

---

### remove_individual(ticket, id)

Remove an individual.

**Parameters:**
- `ticket` (string) - Authentication ticket
- `id` (string) - Individual ID to remove

**Returns:** Result code (integer)

**Example:**
```javascript
var rc = remove_individual("", "d:person_123");
```

---

### add_to_individual(ticket, individual)

Add properties to existing individual.

**Parameters:**
- `ticket` (string) - Authentication ticket
- `individual` (object) - Individual with properties to add

**Returns:** Result code (integer)

**Example:**
```javascript
var update = {
    "@": "d:person_123",
    "v-s:email": [{data: "new@email.com", type: "String"}]
};
add_to_individual("", update);
```

---

### set_in_individual(ticket, individual)

Set (replace) properties in individual.

**Parameters:**
- `ticket` (string) - Authentication ticket
- `individual` (object) - Individual with properties to set

**Returns:** Result code (integer)

---

### remove_from_individual(ticket, individual)

Remove specific properties from individual.

**Parameters:**
- `ticket` (string) - Authentication ticket
- `individual` (object) - Individual with properties to remove

**Returns:** Result code (integer)

## Search Operations

### query(ticket, query, sort, databases, top, limit, from)

Execute search query.

**Parameters:**
- `ticket` (string) - Authentication ticket (empty for transaction ticket)
- `query` (string) - Search query string
- `sort` (string, optional) - Sort specification
- `databases` (string, optional) - Database filter
- `top` (number, optional) - Maximum results (default: 100000)
- `limit` (number, optional) - Result limit (default: 100000)
- `from` (number, optional) - Starting offset (default: 0)

**Returns:** Query result object with:
- `count` - Total count
- `result` - Array of individual IDs
- `estimated` - Estimated count
- `processed` - Processed count
- `total_time` - Total query time
- `query_time` - Query execution time
- `authorize_time` - Authorization time
- `result_code` - Result code

**Example:**
```javascript
var result = query("", "'rdf:type' === 'v-s:Person'", "", "", 100, 100, 0);
print("Found:", result.count);
for (var i = 0; i < result.result.length; i++) {
    print(result.result[i]);
}
```

## Authorization

### get_rights(ticket, id, user_id)

Get user rights for individual.

**Parameters:**
- `ticket` (string) - Authentication ticket
- `id` (string) - Individual ID
- `user_id` (string, optional) - User ID (uses current user if not specified)

**Returns:** Permission statement object with boolean flags:
- `v-s:canRead`
- `v-s:canCreate`
- `v-s:canUpdate`
- `v-s:canDelete`

**Example:**
```javascript
var rights = get_rights("", "d:document_123", "d:user_456");
if (rights["v-s:canUpdate"]) {
    // User can update
}
```

## Environment Variables

### get_env_str_var(name)

Get string environment variable.

**Parameters:**
- `name` (string) - Variable name

**Returns:** String value or undefined

**Available Variables:**
- `$ticket` - Current ticket
- `$parent_script_id` - Parent script ID
- `$parent_document_id` - Parent document ID
- `$super_classes` - Super classes array

**Example:**
```javascript
var ticket = get_env_str_var("$ticket");
var parentId = get_env_str_var("$parent_document_id");
```

---

### get_env_num_var(name)

Get numeric environment variable.

**Parameters:**
- `name` (string) - Variable name

**Returns:** Number value or undefined

## Logging

### print(...args)

Log multiple arguments to info level.

**Example:**
```javascript
print("Processing", individual["@"], "at", Date.now());
```

---

### log_trace(message)

Log trace message to info level.

**Example:**
```javascript
log_trace("Script execution started");
```

## Data Types

Individual properties are objects with:
- `data` - Property value
- `type` - Data type: "String", "Uri", "Integer", "Decimal", "Datetime", "Boolean"
- `lang` - Language code (for String type, optional)

**Type Examples:**
```javascript
// String
{data: "Hello", type: "String", lang: "EN"}

// Integer
{data: 42, type: "Integer"}

// Decimal
{data: "3.14159", type: "Decimal"}

// Datetime
{data: "2024-01-15T10:30:00Z", type: "Datetime"}

// Boolean
{data: true, type: "Boolean"}

// Uri
{data: "d:person_123", type: "Uri"}
```

