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

### auto Trait Implementations

```rust
impl RefUnwindSafe for Terminal
impl Send for Terminal
impl Sync for Terminal
impl Unpin for Terminal
impl UnwindSafe for Terminal
```

### Blanket Implementations

```rust
impl<T> Any for T
where
    T: 'static + ?Sized,

impl<T> Borrow<T> for T
where
    T: ?Sized,

impl<T> BorrowMut<T> for T
where
    T: ?Sized,

impl<T> From<T> for T

impl<T, U> Into<U> for T
where
    U: From<T>,

impl<T, U> TryFrom<U> for T
where
    U: Into<T>,

impl<T, U> TryInto<U> for T
where
    U: TryFrom<T>,
```

