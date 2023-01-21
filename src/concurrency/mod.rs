mod context;
mod manager;

/// Utility methods to track the relationship between different lock types.
pub enum LockType {
    S,   // shared lock
    X,   // exclusive lock
    IS,  // intention shared lock
    IX,  // intention exclusive lock
    SIX, // shared intention exclusive lock
    NL,  // no lock held
}
