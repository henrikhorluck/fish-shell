use crate::common::CancelChecker;
use crate::env::{EnvStack, EnvStackRef, EnvStackRefFFI, Environment};
use crate::job_group::JobGroup;
use once_cell::sync::Lazy;

use crate::ffi::parser_t;
use std::sync::{Arc, Mutex, RwLock};

/// A common helper which always returns false.
pub fn no_cancel() -> bool {
    false
}

// Default limits for expansion.
/// The default maximum number of items from expansion.
pub const EXPANSION_LIMIT_DEFAULT: usize = 512 * 1024;
/// A smaller limit for background operations like syntax highlighting.
pub const EXPANSION_LIMIT_BACKGROUND: usize = 512;

/// A operation_context_t is a simple property bag which wraps up data needed for highlighting,
/// expansion, completion, and more.
pub struct OperationContext<'a> {
    // The parser, if this is a foreground operation. If this is a background operation, this may be
    // nullptr.
    // XXX: It goes without saying we don't actually own the parser, this is extremely bogus
    pub parser: Option<Arc<Mutex<&'a mut parser_t>>>,

    // The set of variables.
    pub vars: Arc<dyn Environment>,

    vars_ffi: EnvStackRefFFI,

    // The limit in the number of expansions which should be produced.
    pub expansion_limit: usize,

    /// The job group of the parental job.
    /// This is used only when expanding command substitutions. If this is set, any jobs created
    /// by the command substitutions should use this tree.
    pub job_group: Option<Arc<RwLock<JobGroup>>>,

    // A function which may be used to poll for cancellation.
    pub cancel_checker: &'a CancelChecker,
}

// todo!
// static nullenv: Lazy<Arc<EnvNull>> = Lazy::new(|| Arc::new(EnvNull {}));
static nullenv: Lazy<EnvStackRef> = Lazy::new(|| Arc::new(EnvStack::new()));

impl<'a> OperationContext<'a> {
    // \return an "empty" context which contains no variables, no parser, and never cancels.
    pub fn empty() -> OperationContext<'static> {
        OperationContext::new(nullenv.clone(), EXPANSION_LIMIT_DEFAULT)
    }

    // \return an operation context that contains only global variables, no parser, and never
    // cancels.
    pub fn globals() -> OperationContext<'static> {
        OperationContext::new(EnvStack::globals().clone(), EXPANSION_LIMIT_DEFAULT)
    }

    /// Construct from a full set of properties.
    pub fn foreground(
        parser: &'a mut parser_t,
        cancel_checker: &'a CancelChecker,
        expansion_limit: usize,
    ) -> OperationContext<'a> {
        let vars = parser.get_vars();
        let vars_ffi = EnvStackRefFFI(Arc::clone(&vars));
        OperationContext {
            parser: Some(Arc::new(Mutex::new(parser))),
            vars,
            vars_ffi,
            expansion_limit,
            job_group: None,
            cancel_checker,
        }
    }

    /// Construct from vars alone.
    pub fn new(vars: EnvStackRef, expansion_limit: usize) -> Self {
        let vars_ffi = EnvStackRefFFI(Arc::clone(&vars));
        OperationContext {
            parser: None,
            vars,
            vars_ffi,
            expansion_limit,
            job_group: None,
            cancel_checker: &no_cancel,
        }
    }

    pub fn has_parser(&self) -> bool {
        self.parser.is_some()
    }

    // Invoke the cancel checker. \return if we should cancel.
    pub fn check_cancel(&self) -> bool {
        (self.cancel_checker)()
    }
}
