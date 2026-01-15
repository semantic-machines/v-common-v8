# v-common-v8 Documentation

## Overview

`v-common-v8` is a common library component for the Veda platform that provides V8 JavaScript engine integration for executing scripts within the Veda environment.

## Table of Contents

- [Architecture Overview](architecture.md)
- [API Reference](api.md)
- [Callback Functions](callbacks.md)
- [Transaction System](transactions.md)
- [Script Execution](scripts.md)
- [Getting Started](getting_started.md)

## Purpose

This library provides:
- V8 JavaScript runtime initialization and management
- Callback functions for JavaScript scripts to interact with Veda backend
- Transaction management for data operations
- Script loading and execution environment
- Integration with Veda's authorization and search systems

## Key Features

1. **V8 Runtime Management** - Initialize and manage V8 isolates and contexts
2. **JavaScript Callbacks** - Expose Rust functions to JavaScript for data operations
3. **Transaction System** - Manage transactional operations on individuals
4. **Script Execution** - Load and execute external JavaScript files
5. **Session Management** - Handle shared data between script executions

## Dependencies

- `v8` (version 0.84) - V8 JavaScript engine bindings
- `v_common` - Common Veda platform functionality
- `v-individual-model` - Individual data model
- Additional dependencies for JSON, async operations, and utilities

## Quick Links

- [Callback Functions Documentation](callbacks.md) - Learn about available JavaScript callbacks
- [Transaction System](transactions.md) - Understand how to work with transactions
- [API Reference](api.md) - Complete API documentation

