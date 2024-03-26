# pseudoterminal

## `pseudoterminal::Terminal`

`STRUCTURE`

```rust
pub struct Terminal {
    pub termin: Option<TerminalIn>,
    pub termout: Option<TerminalOut>,
    /* private fields */
}
```

### implementation

```rust
impl Terminal
pub fn get_term_size(&mut self) -> Result<TerminalSize>
pub fn set_term_size(&mut self, new_size: TerminalSize) -> Result<()>
pub fn close(self) -> Result<()>
```

