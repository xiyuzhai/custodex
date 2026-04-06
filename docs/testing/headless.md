# Headless Testing

## Goal

A test binary that exercises the full app lifecycle without egui. Same `AppState`, same method calls, same bot logic. The only difference is rendering — replaced by stdout/assertions.

## What it tests

- App startup: directories created, config loaded, saved instances restored
- Wizard flow: open → select template → configure → launch
- Instance persistence: create instance, save, reload, verify
- Bot lifecycle: start, receive events, stop
- Log filtering: set filter, verify entries match

## What it can't test

- egui rendering correctness
- Mouse/keyboard interaction
- Window management

## How to run

```
cargo run --bin headless-test
```

## Principle

The fake and real versions have minimal difference. If a bug exists in `AppState`, the headless test catches it. This is the primary debugging tool — not the GUI.
