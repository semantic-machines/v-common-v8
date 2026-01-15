# Architecture Overview

## System Architecture

`v-common-v8` is structured around several key modules that work together to provide JavaScript execution within the Veda platform.

## Module Structure

```
src/
├── lib.rs              - Library entry point and module exports
├── jsruntime.rs        - V8 runtime initialization and management
├── callback.rs         - JavaScript callback functions
├── common.rs           - Utility functions and data converters
├── scripts_workplace.rs - Script loading and execution environment
└── session_cache.rs    - Session data and transaction management
```

## Core Components

### 1. JsRuntime (`jsruntime.rs`)

Manages V8 isolate lifecycle:
- Initializes V8 platform and isolates
- Creates execution contexts
- Handles isolate state management

**Key Functions:**
- `v8_init()` - Initialize V8 platform
- `JsRuntime::new()` - Create new runtime instance
- `setup_isolate()` - Configure isolate settings

### 2. Callback System (`callback.rs`)

Provides JavaScript-accessible functions:
- Individual CRUD operations
- Search queries
- Authorization checks
- Logging and debugging

**Main Callbacks:**
- `get_individual` - Fetch individual by ID
- `put_individual` - Create/update individual
- `query` - Execute search queries
- `get_rights` - Check user permissions

### 3. Common Utilities (`common.rs`)

Data conversion and utilities:
- V8 ↔ Rust type conversions
- Individual ↔ V8 object mapping
- Script information structures
- File system helpers

### 4. Scripts Workplace (`scripts_workplace.rs`)

Script execution environment:
- Load JavaScript files from filesystem
- Compile and execute scripts
- Manage script dependencies
- Initialize execution context

### 5. Session Cache (`session_cache.rs`)

Transaction and session management:
- Transaction buffering
- Shared data between executions
- Commit operations to storage

## Data Flow

```
JavaScript Script
      ↓
   Callback Function
      ↓
   Transaction Buffer
      ↓
   Storage Commit
```

## Integration Points

### With Veda Platform:
- **Authorization** - LMDB-based authorization context
- **Search** - Full-text search client integration
- **Storage** - Backend storage operations
- **Ontology** - Type system and class hierarchy

### Thread Safety:
- Uses `Mutex` for shared state (transactions, authorization, search client)
- `lazy_static` for global state initialization
- V8 isolates are single-threaded

## Execution Model

1. **Initialization Phase**
   - V8 platform initialization (once)
   - Create isolate and context
   - Register callback functions
   
2. **Script Loading Phase**
   - Discover JavaScript files
   - Load and compile scripts
   - Execute in defined order
   
3. **Execution Phase**
   - Scripts call callback functions
   - Operations buffered in transaction
   - Transaction committed on completion

## Memory Management

- V8 handles JavaScript heap automatically
- Rust objects managed by ownership system
- Global state protected by Mutex
- No manual memory management needed

