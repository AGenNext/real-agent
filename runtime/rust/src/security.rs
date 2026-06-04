//! Security as a core concern (not an adapter): who is acting, and the security
//! posture that gates whether they may. Vendor-neutral, dependency-free.

use crate::memory::Classification;

/// The authenticated actor on whose behalf an operation is requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub provider: String,
    pub subject: String,
}

impl Principal {
    pub fn new(provider: &str, subject: &str) -> Self {
        Principal { provider: provider.to_string(), subject: subject.to_string() }
    }

    /// An unauthenticated caller.
    pub fn anonymous() -> Self {
        Principal { provider: String::new(), subject: String::new() }
    }

    pub fn is_anonymous(&self) -> bool {
        self.provider.is_empty() && self.subject.is_empty()
    }
}

/// An agent's security posture. Defaults are open (nothing required) so an agent
/// only tightens as configured — but governance still applies deny-by-default at
/// the authority layer.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Security {
    pub authentication_required: bool,
    pub authorization_required: bool,
    pub allowed_identity_providers: Vec<String>,
    pub data_classifications: Vec<Classification>,
}

impl Security {
    /// Evaluate a principal against this posture. `Ok(())` means allowed.
    pub fn check(&self, principal: &Principal) -> Result<(), &'static str> {
        if self.authentication_required && principal.is_anonymous() {
            return Err("authentication required");
        }
        if self.authorization_required
            && !self.allowed_identity_providers.iter().any(|p| p == &principal.provider)
        {
            return Err("identity provider not authorized");
        }
        Ok(())
    }
}
