use std::time::Duration;

use diorama_supervisor::{
    Adapter, AdapterContext, AdapterError, AdapterFuture, ComponentKind, ComponentSpec,
    FailurePolicy, Registration, StopPhase,
};

use crate::service::DashboardService;

/// Canonical supervisor component identifier for the realtime node dashboard.
pub const COMPONENT_DASHBOARD: &str = "dashboard";

/// Default startup timeout for the node dashboard component.
pub const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(15);

/// Supervisor adapter for [`DashboardService`].
///
/// Uses [`FailurePolicy::Degrade`] so dashboard listener issues never take down
/// the core Kubernetes control plane.
#[derive(Clone, Debug)]
pub struct DashboardAdapter {
    service: DashboardService,
}

impl DashboardAdapter {
    /// Creates a new `DashboardAdapter` wrapping `service`.
    #[must_use]
    pub const fn new(service: DashboardService) -> Self {
        Self { service }
    }

    /// Constructs a supervisor [`Registration`] with [`FailurePolicy::Degrade`].
    pub fn registration(
        id: impl Into<String>,
        service: DashboardService,
        prerequisites: Vec<String>,
        timeout: Duration,
    ) -> Registration {
        let spec = ComponentSpec {
            id: id.into(),
            prerequisites,
            kind: ComponentKind::LongRunning,
            failure_policy: FailurePolicy::Degrade,
            startup_timeout: timeout,
        };
        Registration::new(spec, Self::new(service))
    }
}

impl Adapter for DashboardAdapter {
    fn run(self: Box<Self>, mut context: AdapterContext) -> AdapterFuture {
        Box::pin(async move {
            if !self.service.config().is_enabled() {
                if !context.ready() {
                    return Err(AdapterError {
                        code: "dashboard-readiness-rejected",
                    });
                }
                while context.changed().await == StopPhase::Running {}
                self.service.stop();
                return Ok(());
            }

            if let Err(err) = self.service.start().await {
                return Err(AdapterError {
                    code: err.diagnostic_code(),
                });
            }

            if !context.ready() {
                self.service.stop();
                return Err(AdapterError {
                    code: "dashboard-readiness-rejected",
                });
            }

            loop {
                match context.changed().await {
                    StopPhase::Running => {},
                    StopPhase::Graceful | StopPhase::Force => {
                        self.service.stop();
                        break;
                    },
                }
            }

            Ok(())
        })
    }
}
